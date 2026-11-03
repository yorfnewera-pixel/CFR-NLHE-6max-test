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
