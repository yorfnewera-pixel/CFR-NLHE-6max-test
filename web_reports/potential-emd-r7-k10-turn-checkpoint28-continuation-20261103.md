# Test-only continuation R7/K10 из checkpoint-28 — стабилизация на шаге 29

Дата: 2026-11-03. Итоговый continuation-run достиг численной согласованности
назначений на шаге 29: final changed fraction равна IEEE-754 signed zero,
состояние точно повторилось на переходе `29→30` и оставалось неизменным до
границы 50. Это подтверждает преждевременность старой пороговой остановки
turn для данного контроля; replay шагов 1–28, flop и production-таблицы не
запускались.

## История первой остановленной попытки

### Подготовка

Test-only commit: `3a0410243b33c9425fb932a8fb8c0e7a53a12d88`; дерево перед actual
run было чистым. Добавлены только ignored continuation-test, его PowerShell
runner и `.gitignore` для локальных результатов. Production-код, метрика,
центры, допуски и production-launcher не менялись.

Малые проверки runner подтвердили полный exact filter, включая отклонение
нулевого и множественного выбора. Обычный Rust suite прошёл: `71 passed`,
`0 failed`, `10 ignored`. `VerifyOnly` собрал release test-бинарник и
обнаружил ровно один test:

```text
tests::test_only_ranks7_turn_k10_continues_after_verified_step_28_only: test

1 test, 0 benchmarks
```

До Cargo run runner проверил существующий checkpoint SHA-256:
`94251D3D492741A62D1E38216F4C472597BAF6A4F12FDE71FB203C9AB3B213FC`.

### Единственный actual run

```powershell
pwsh -NoProfile -File .\native\hu20-buckets\scripts\run-r7-turn-continuation-probe.ps1 -OutputDirectory .\native\hu20-buckets\results\test-only-r7-turn-k10-checkpoint28-continuation-20261103 -CheckpointPath .\native\hu20-buckets\results\test-only-r7-turn-k10-replay-only-execution-20261103\checkpoints\checkpoint-28.bin -TimeoutSeconds 300
```

Параметры: `ranks=7`, `K=10`, seed `202610050003`, два Rayon-потока,
плановые шаги `29–50`, timeout `300 с`. Exact run выполнил ровно один test,
но он завершился `exit_code=101`: `0 passed; 1 failed; 68 filtered out`.
Timeout не сработал; libtest работал `1.15 с`. Максимум единственной
доступной выборки памяти: private bytes `3,272,704`, working set `9,564,160`.

Причина — строгий gate исходной немутирующей диагностики до шага 29:

```text
left:  0.000019111323459144
right: 0.000019111323459150
```

Контроль ранее сохранил округлённую десятичную строку
`0.00001911132345915`, а точное IEEE-754 значение после загрузки checkpoint
равно `0.000019111323459144`. Gate намеренно использовал `assert_eq!` и не
расширял допуск, поэтому continuation остановлен до любого шага 29.

`turn-trace.log` пуст, checkpoints 29–50 не созданы, повторов полных состояний
нет и вопрос о достижении фактического нуля остаётся неотвеченным. После
остановки процессов `hu20-buckets` нет. Повтор, альтернативный probe и
изменение gate автоматически не выполнялись.

## Единственный continuation-run после побитового preflight

Inspection preflight на commit `52a426fc27e2fe3baec667340692fe238adcd64d`
зафиксировал serialised objective checkpoint-28 `0x3FB007C287E6DCF2` и
заново вычисляемые final objective `0x3FB007BBAA348F28` / changed fraction
`0x3EF40A2825D92ED3`. Final diagnostic не сериализована в checkpoint; она
привязана к SHA входа, fingerprint восстановленных входов и этой версии
диагностического кода. Preflight: `checkpoint_gate_passed=true`,
`continuation_started=false`.

Ровно один actual run в новом локальном каталоге
`results/test-only-r7-turn-k10-checkpoint28-continuation-actual-20261103`
прошёл с `exit_code=0`: выполнен один целевой test (`1 passed; 0 failed`) за
`3.04 с`, timeout не сработал. До него SHA checkpoint совпал с
`94251D3D492741A62D1E38216F4C472597BAF6A4F12FDE71FB203C9AB3B213FC`, а
release build и discovery имели код `0` и выбрали ровно один continuation-test.

| Шаги | Objective до / после update | Moved fraction | Занятые | Final objective / changed |
| --- | --- | --- | ---: | --- |
| 29 | `0.06261799722102224` / `0.06261780246870059` | `0.00001911132345915` | 10 | `0.06261780246870059` / `-0.0` |
| 30–50 (каждый) | `0.06261780246870059` / `0.06261780246870059` | `-0.0` | 10 | `0.06261780246870059` / `-0.0` |

Полный trace содержит отдельные строки для каждого шага 29–50. Final changed
fraction стал численно нулевым на шаге 29 и сохранялся до шага 50 (trace
показывает IEEE-754 signed zero `-0.0`). Полное algorithmic state точно
повторилось на `29→30`; штатная причина остановки — test-only граница 50.
Сохранены атомарные checkpoints 29–50. Максимумы 4 выборок памяти: private
bytes `3,117,056`, working set `9,416,704`. Процессов `hu20-buckets` после
run нет; следующих экспериментов не запускалось.
