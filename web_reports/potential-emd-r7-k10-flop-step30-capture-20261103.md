# Test-only capture flop R7/K10 до шага 30

Дата: 2026-11-03. Выполнен ровно один supervised test-only capture в новом
локальном каталоге
`native/hu20-buckets/results/test-only-r7-k10-flop-step30-capture-20261103`.
Continuation шагов 31–50, replay после capture и production-таблицы не
выполнялись.

## Provenance и запуск

| Параметр | Значение |
| --- | --- |
| Commit кода capture до build/run | `3d6bc352df7f91cd87eb2a17a8875571e0ac45e9` |
| Дерево до build/run | чистое |
| Baseline | `potential-emd-r7-k10-turn50-flop30-gate-control-20261103` |
| Revision baseline | `80895882faf3fbfa293bf1553d124fc21b185566` |
| Параметры | `ranks=7`, `K=10`, seed `202610050003`, budgets `10/50/30`, Rayon `2` |
| Timeout | `300 с` |
| Выбранный тест | `tests::test_only_ranks7_flop_k10_captures_step_thirty` |
| Реальный exit code | `0` |
| Фактическое время test | `196.83 с` |

До вычислений runner проверил наличие и полноту baseline: ровно 10 river,
29 turn и 30 flop objectives; hash baseline turn равен
`625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990`.
Release discovery выбрал ровно один ignored capture-test. `stdout.log`
подтверждает `1 passed; 0 failed; 77 filtered out`.

## Сверка с историческим production-контролем

Сверялись только исторически доступные результаты, без новых допусков:

| Объект | Результат |
| --- | --- |
| River objectives 1–10 | побитовое совпадение с baseline |
| Turn objectives 1–29 | побитовое совпадение с baseline |
| Turn шаг 29 | `0.06261799722102224`; сохранена `EarlyStopConsistent` и финальная signed-zero согласованность |
| Flop objectives 1–30 | побитовое совпадение с полным массивом `summary.json` |
| Финальная flop-диагностика | objective `0.046481510`, changed weight fraction `0.005177045` |

Это подтверждает восстановление траектории objectives до шага 30. Полного
исторического flop-state/checkpoint не существовало, поэтому побитовое
совпадение полного состояния с историческим production-запуском не заявляется.

## Новый checkpoint-30

После сверки capture атомарно записал `checkpoint-30.bin`. Он содержит:

- точные flop distribution, веса и `turn_ground`;
- fingerprint этих входов `45E6F95C08AB346B`;
- assignments, биты центров, RNG, objectives, полную историю diagnostics и
  флаг стабилизации.

Тот же тест загрузил checkpoint без выполнения шага 31 и побитово сравнил
assignments/центры/RNG через state words, objectives и каждое поле diagnostics.
Round-trip прошёл. SHA-256 checkpoint:

```text
C65A5C0B011B6841D0BB0F0DDB23710938D4EDDE8EE1DF9A5DC80495A4B5FC65
```

## Ресурсы и границы

Внешний монитор сохранил 197 выборок: максимум Private Bytes — `3,219,456 B`,
Working Set — `9,478,144 B`. После run нет процессов `cargo` или test-бинарника.
Созданы только test-only checkpoint и логи; production-таблицы отсутствуют.
Никакой continuation после шага 30 не запускался.
