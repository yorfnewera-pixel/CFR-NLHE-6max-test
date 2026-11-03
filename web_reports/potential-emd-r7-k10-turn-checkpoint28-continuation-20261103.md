# Test-only continuation R7/K10 из checkpoint-28 — остановлен на gate

Дата: 2026-11-03. Это ровно одна разрешённая вычислительная попытка
continuation из сохранённого checkpoint-28; replay шагов 1–28, flop и
production-таблицы не запускались.

## Подготовка

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

## Единственный actual run

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
