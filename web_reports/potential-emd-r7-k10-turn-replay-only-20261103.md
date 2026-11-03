# Test-only replay-only turn R7/K10 — остановлен до теста

Дата: 2026-11-03.

## Граница проверки

Подготовлен test-only `Wasserstein1dStepper` и отдельный ignored-test
`test_only_ranks7_turn_k10_replays_verified_step_28_only`. Он ограничен
28 шагами, проверяет каждый сохранённый objective строгим `assert_eq!`,
проверяет историческую раннюю остановку и финальную немутирующую диагностику,
затем записывает и загружает только checkpoint-28. Переход к 29–50 в этом
тесте отсутствует; flop и production-таблицы не создаются.

Малая проверка `test_only_ranks7_replay_limit_excludes_continuation` прошла.

## Единственная попытка запуска

Команда:

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-replay-only.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-20261103 -TimeoutSeconds 300
```

Параметры: `ranks=7`, `K=10`, seed `202610050003`, два Rayon-потока,
river budget `10`, turn limit `28`, timeout `300 с`, `continuation=false`.
HEAD на момент попытки: `76f246207145631474ba710cbcaf8d08be8896ac`; рабочее
дерево было dirty из-за ещё не зафиксированных test-only правок.

До измерения `cargo test --release --no-run` успешно собрал release
test-бинарник за `21.03 с`. Затем supervisor завершился с реальным
`exit_code=101`, не по timeout. Дочерний Cargo был запущен из корня
репозитория, а не из `native/hu20-buckets`, и вывел:

```text
error: could not find `Cargo.toml` in `C:\Users\Cassmall\Desktop\CFR-NLHE-6max-test` or any parent directory
```

Следовательно, ignored-test не стартовал: ни один из 28 gates не был
проверен, `turn-trace.log` и `checkpoint-28.bin` не созданы, SHA-256
checkpoint отсутствует. Timeout не срабатывал. После завершения процессов
`hu20-buckets` не осталось.

## Вывод

Это отказ test-only runner из-за отсутствующего рабочего каталога у
supervised `cargo.exe`, а не свидетельство о production Wasserstein или
replay-траектории. По правилу одного запуска повтор и альтернативный probe
не выполнялись. Перед любым новым согласованием нужно исправить runner,
например передав Cargo явный `--manifest-path native/hu20-buckets/Cargo.toml`,
проверить этот путь малым тестом и заново согласовать единственную попытку.

## Вторая разрешённая попытка после исправления manifest path

Абсолютный manifest path, точный test filter с `--release --ignored --exact`
и малый PowerShell-тест были добавлены. Без выполнения replay discovery
обнаружил нужный тест:

```text
tests::test_only_ranks7_turn_k10_replays_verified_step_28_only: test
```

Обычный Rust suite прошёл (`70` tests, `0` failures, `9` ignored). Затем
была выполнена одна новая разрешённая команда с новым пустым каталогом:

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-replay-only.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-fixed-runner-20261103 -TimeoutSeconds 300
```

Release test build завершился с кодом `0`, но runner остановился на
внутреннем discovery до supervised процесса. Проверка использовала PowerShell
`-notmatch` для массива строк: среди результата `--list` была нужная строка
и служебные несовпадающие строки, поэтому условие ложно сработало. Это не
ошибка Cargo, не timeout и не численный replay.

Ignored-test не запускался: trace, stdout/stderr, checkpoint и его SHA-256
отсутствуют; процессов `hu20-buckets` после остановки нет. HEAD на момент
второй попытки: `0ffb1cee86cb91c09b842ae05daca001ee07dd7e`.

По условию «при любой ошибке — без автоматического повтора» третья попытка
не выполнялась. До нового отдельного согласования runner должен заменить
проверку массива на булеву проверку наличия хотя бы одной совпавшей строки и
получить малый тест для multi-line discovery-вывода.

## Runner verification без replay

Discovery исправлен без изменения stepper, production-кода, production
launcher, допусков или numerical gates. Проверка отдельно отклоняет ненулевой
код Cargo, нормализует CRLF/LF в строки и требует хотя бы одну строку,
в точности равную полному имени test с суффиксом `: test`.

Малый PowerShell-тест покрывает нужный test среди заголовков и сводки,
похожие имена, пустой вывод, ненулевой exit code и CRLF/LF. Он прошёл вместе
с parser-проверкой скриптов. Обычный Rust suite: `70` passed, `0` failed,
`9` ignored; `cargo fmt --check` и `git diff --check` успешны.

Тот же runner был безопасно вызван без вычислительной ветки:

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-replay-only.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-verify-only-20261103 -TimeoutSeconds 300 -VerifyOnly
```

Рабочий каталог shell: `C:\Users\Cassmall\Desktop\CFR-NLHE-6max-test`.
Runner сохранил абсолютный manifest
`C:\Users\Cassmall\Desktop\CFR-NLHE-6max-test\native\hu20-buckets\Cargo.toml`.
Release build и discovery завершились с кодом `0`; discovery нашёл ровно:

```text
tests::test_only_ranks7_turn_k10_replays_verified_step_28_only: test
```

`verify_only_complete=true`. Supervised replay не вызывался, `turn-trace.log`
и checkpoint не созданы, процессов `hu20-buckets` нет. HEAD:
`5aaec75a53397193c5cc0370c5fc75f0ca617a58`. Новый вычислительный запуск
требует отдельного согласования.

## Вычислительная попытка: Cargo выполнил ноль тестов

После успешного `-VerifyOnly` была разрешена ровно одна вычислительная
replay-only попытка:

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-replay-only.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-run-20261103 -TimeoutSeconds 300
```

Release build и discovery вновь имели код `0`, а supervisor записал
`exit_code=0` и не достиг timeout. Однако stdout Cargo содержит:

```text
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 67 filtered out
```

`--exact` получил короткое имя
`test_only_ranks7_turn_k10_replays_verified_step_28_only`, тогда как
discovery возвращает полное `tests::test_only_ranks7_turn_k10_replays_verified_step_28_only`.
Следовательно, exact filter не выбрал ignored-test. Это infrastructure/filter
failure: river/turn вычисления не стартовали, gates не выполнялись,
`turn-trace.log`, checkpoint и SHA-256 отсутствуют. Внешний memory monitor
также не успел присоединиться к уже завершившемуся процессу; это следствие
нулевого test-run, а не timeout.

HEAD на момент попытки: `9d0dcc3a883a4562d0c2dca6749c051cd2ebc3ae`. Процессов
`hu20-buckets` не осталось. По условию автоматический повтор и альтернативный
probe не выполнялись. До нового решения runner должен хранить полное libtest
имя и его малая проверка должна дополнительно доказывать, что exact command
выбирает ровно один test, а не ноль.

## Exact selection verification без replay

Runner теперь хранит единое полное libtest-имя
`tests::test_only_ranks7_turn_k10_replays_verified_step_28_only` и передаёт
один selection array как discovery, так и будущему run. Он содержит:

```text
test --manifest-path C:\Users\Cassmall\Desktop\CFR-NLHE-6max-test\native\hu20-buckets\Cargo.toml --release --lib tests::test_only_ranks7_turn_k10_replays_verified_step_28_only
```

Replay добавляет к тому же selection array только libtest flags:

```text
--ignored --exact --nocapture
```

Малые PowerShell-проверки подтвердили: полное имя выбирает ровно один test;
короткое, похожее или отсутствующее имя выбирает ноль и отклоняется; два
совпадения также отклоняются. Дополнительно покрыты ненулевой Cargo exit code
и CRLF/LF.

Без вычислительной ветки runner выполнен с `-VerifyOnly`:

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-replay-only.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-exact-verify-20261103 -TimeoutSeconds 300 -VerifyOnly
```

`release_build_exit_code=0`, `discovery_exit_code=0`,
`discovery_found=true`, `verify_only_complete=true`. Вывод exact discovery:

```text
tests::test_only_ranks7_turn_k10_replays_verified_step_28_only: test

1 test, 0 benchmarks
```

Replay не запускался; trace, checkpoint и SHA-256 отсутствуют, процессов
`hu20-buckets` нет. Нужна новая отдельная авторизация вычислительной попытки.
