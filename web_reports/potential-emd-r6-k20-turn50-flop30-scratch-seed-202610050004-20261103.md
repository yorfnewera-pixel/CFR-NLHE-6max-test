# Второй seed: production-контроль EmdScratch, ranks=6, K=20

Дата: 2026-11-03. Выполнен ровно один launcher-контроль с вторым seed в новом пустом каталоге `results/potential-emd-r6-k20-turn50-flop30-scratch-seed-202610050004-20261103`. Код, размеры, K, бюджеты и число потоков не менялись. Других запусков не выполнялось.

## Проверяемое происхождение и параметры

Перед измерением успешно выполнены `cargo fmt --check`, `cargo test` (65 passed, 0 failed, 8 ignored), release-сборка и синтаксический разбор launcher-скрипта.

| Параметр | Значение |
| --- | --- |
| SHA до сборки | `4e28c57bbd7e0727d7cfa720f67b224bc6c591dd` |
| Дерево до сборки | чистое |
| SHA перед запуском | `4e28c57bbd7e0727d7cfa720f67b224bc6c591dd` |
| Дерево перед запуском | чистое |
| `summary.code_revision` / dirty | `4e28c57bbd7e0727d7cfa720f67b224bc6c591dd` / `false` |
| Параметры | `ranks=6`, `K=20`, общий `iterations=10`, `turn_iterations=50`, `flop_iterations=30`, `threads=2`, seed `202610050004` |
| Флаги | `--diagnose-final-assignments --profile-flop-emd` |
| Timeout | 600 с |
| Exit code / status | `0` / `complete` |
| Время генератора | 174,3 с |
| Таблицы / summary / failure-dump | 3 / присутствует / отсутствует |

После завершения процессов `hu20-buckets` не осталось.

## Финальная согласованность

| Стадия | Итераций | Final objective | Changed weight fraction |
| --- | ---: | ---: | ---: |
| river | 6 | `0.010894333` | — |
| turn | 47 | `0.054295516` | `0.000000000` |
| flop | 13 | `0.040057571` | `0.000000000` |

В stderr обе нулевые доли напечатаны как `-0.000000000`; это IEEE-754 signed zero. Objectives конечны. Нулевая доля reassignment у turn и flop подтверждает финальную согласованность назначений для этого запуска.

## Новый исходный контроль seed `202610050004`

Полные массивы `river_objective`, `turn_objective` и `flop_objective` сохранены в локальном `summary.json`; они не сопоставлялись с seed `202610050003`, поскольку иной seed меняет допустимую траекторию и таблицы.

| Таблица | SHA-256 |
| --- | --- |
| river | `2114BA230A480290AA22DEE555D4ECA366B7A34B5C9E106FDDD94DAE4E55BAC0` |
| turn | `83DEF91EB463B3102309A42785E33589CA86E680239BB83614345F99EDA22C09` |
| flop | `1943937331C40E0B62EA00E8262C83F4C47133ADBB5A3AEB054FE7D9D8CF60BA` |

## Профиль и память

| Показатель | Значение |
| --- | ---: |
| Flop clustering | 170,827272400 с |
| Flop assignment: 13 проходов / EMD-вызовы | 5 499 000 |
| Objective-after-update: EMD-вызовы | 274 950 |
| Final diagnostic: EMD-вызовы / время | 423 000 / 11,644475700 с |
| Turn ground matrix: EMD-вызовы / время | 190 / 0,026430800 с |
| Внешние замеры памяти | 175 |
| Максимум Private Bytes | 32 972 800 B (≈ 31,44 MiB) |
| Максимум Working Set | 32 952 320 B (≈ 31,42 MiB) |

Время данного запуска не сопоставляется с первым seed как измерение ускорения: второй seed выполнил другое число turn/flop итераций. Это новый исходный контроль для параметров данного seed, а не обобщение на иные входы.
