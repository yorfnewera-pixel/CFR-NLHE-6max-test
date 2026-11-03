# Production-контроль EmdScratch: ranks=6, K=20, turn=50, flop=30

Дата: 2026-11-03. Выполнен ровно один launcher-контроль после интеграционной проверки `EmdScratch`, в новом пустом каталоге `results/potential-emd-r6-k20-turn50-flop30-scratch-control-20261103`. Test-only checkpoint не использовался; дополнительных запусков не выполнялось.

## Интеграционная проверка до измерения

Vec-reference clustering действительно использовал новый scratch solver, поэтому он не был независимым oracle интеграции. Добавлен test-only путь, который использует прежнее EMD-ядро `try_emd_pre_scratch_reference` внутри того же `kmeans_emd_profiled_with_assignment`, без второй реализации clustering. На малом детерминированном fixture production scratch-путь и старое ядро точно совпали по assignment, числу итераций, битам центроидов, objective, diagnostics и final diagnostic при 1 и 2 Rayon-потоках.

До launcher успешно выполнены `cargo fmt --check`, `cargo test` (65 passed, 0 failed, 8 ignored), release-сборка и `git diff --check`.

## Проверяемое происхождение и завершение

| Параметр | Значение |
| --- | --- |
| SHA до сборки | `3e40c4a9f69005a098c9e1df7180a901a0373d6b` |
| Дерево до сборки | чистое |
| SHA перед запуском | `3e40c4a9f69005a098c9e1df7180a901a0373d6b` |
| Дерево перед запуском | чистое |
| `summary.code_revision` / dirty | `3e40c4a9f69005a098c9e1df7180a901a0373d6b` / `false` |
| Параметры | `ranks=6`, `K=20`, общий `iterations=10`, `turn_iterations=50`, `flop_iterations=30`, `threads=2`, seed `202610050003` |
| Флаги | `--diagnose-final-assignments --profile-flop-emd` |
| Timeout | 600 с |
| Exit code / status | `0` / `complete` |
| Время генератора | 312,9 с |
| Таблицы / summary / failure-dump | 3 / присутствует / отсутствует |

После завершения процессов `hu20-buckets` не осталось.

## Точное сопоставление с provenance-repro

Сравнение выполнено с `results/potential-emd-r6-k20-turn50-flop30-provenance-repro-20261103`. Все три SHA-256 и полные массивы `river_objective`, `turn_objective` и `flop_objective` совпали без допуска.

| Таблица | SHA-256 |
| --- | --- |
| river | `A0F2D9F1CE652706248AF8FD310B09018869A838C4B6DF510317310BEB9BDABB` |
| turn | `D31A8C66A4D039BDC7C90FC669186AC52C4DD4A1C5C24ED920393D16BAB18075` |
| flop | `16EAEF73DC8F721895B146A68D732D507ADBABF4D68E24DD8E43C4E51131191C` |

| Стадия | Итераций | Objective до последнего update | Objective после последнего update | Objective final reassignment | Changed weight fraction |
| --- | ---: | ---: | ---: | ---: | ---: |
| turn | 21 | `0.053350390` | `0.053350386` | `0.053350386` | `0.000000000` |
| flop | 24 | `0.042501192` | `0.042501379` | `0.042501379` | `0.000000000` |

Лог печатает нулевые смены как `-0.000000000`; это IEEE-754 signed zero. River использовал 10 итераций общего бюджета; полные trajectory всех трёх улиц совпали с контролем.

## Профиль и память

| Метрика | Provenance-repro | EmdScratch-контроль |
| --- | ---: | ---: |
| Общее время | 392,9 с | 312,9 с |
| Flop clustering | 390,778688600 с | 310,786529900 с |
| Assignment: 10 152 000 EMD-вызовов | 345,797006500 с | 276,781815000 с |
| Objective-after-update: 507 600 EMD-вызовов | 30,464586200 с | 22,999918400 с |
| Final diagnostic: 423 000 EMD-вызовов | 14,508271700 с | 10,994880200 с |
| Внешние замеры памяти | 393 | 313 |
| Максимум Private Bytes | 33 058 816 B | 33 550 336 B |
| Максимум Working Set | 32 305 152 B | 32 817 152 B |

На одном сопоставимом запуске время стало меньше на 80,0 с (около 20,4 %); число EMD-вызовов и результаты не изменились. Один новый прогон не устанавливает устойчивое ускорение: внешние условия и дискретность замеров памяти не контролировались как benchmark.

## Граница вывода

Контроль подтверждает интеграционную эквивалентность и воспроизводимость данного параметрического набора с `EmdScratch`. Он не доказывает производительность для иных входов, K, seed, числа потоков или размеров колоды. Другие production-запуски не выполнялись.
