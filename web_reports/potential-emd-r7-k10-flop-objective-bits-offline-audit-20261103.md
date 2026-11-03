# Offline-сверка objectives flop R7/K10: checkpoint-40 и production summary

Дата: 2026-11-03. Проверены только уже сохранённые артефакты. Генератор,
replay и continuation не запускались; production-таблицы не изменялись.

## Артефакты и целостность

Проверен capture-format checkpoint после шага 40:

`native/hu20-buckets/results/test-only-r7-k10-flop-step30-continuation-20261103/checkpoints/checkpoint-40.bin`.

| Проверка | SHA-256 |
| --- | --- |
| Файл checkpoint-40 | `75BD0FC3D2D51D110AAAF3EBC95063255111939EFD6715707B044246EEAB2865` |
| Опубликованный continuation-отчёт | `75BD0FC3D2D51D110AAAF3EBC95063255111939EFD6715707B044246EEAB2865` |
| Локальный `checkpoint-hashes.json` | `75BD0FC3D2D51D110AAAF3EBC95063255111939EFD6715707B044246EEAB2865` |

Итак, проверенный файл соответствует опубликованному значению. Loader
`TestOnlyEmdCaptureCheckpoint::load` прочитал checkpoint непосредственно,
без восстановления входов, без шага кластеризации и без изменения состояния.
История содержит ровно 40 objectives.

Production-сторона — существующий
`native/hu20-buckets/results/potential-emd-r7-k10-turn50-flop50-generic-gate-20261103/summary.json`,
массив `tables[0].flop_objective`, элементы 31–40. Это production-артефакт
контроля `10/50/50`, завершившегося на шаге flop 40.

## Побитовая сверка

Значение checkpoint получено как `f64::to_bits()` из загруженной истории.
Значение summary получено разбором исходного JSON-числа в `f64` и затем
`to_bits()`. Для всех строк сравнение выполнено строго по `u64`, без допуска.

| Шаг | Checkpoint `to_bits()` | Production summary `to_bits()` | Результат |
| ---: | --- | --- | --- |
| 31 | `3FA7CC6CA924BF19` | `3FA7CC6CA924BF19` | совпадает |
| 32 | `3FA7CDF9985B105C` | `3FA7CDF9985B105C` | совпадает |
| 33 | `3FA7CF5F36E774DE` | `3FA7CF5F36E774DE` | совпадает |
| 34 | `3FA7D030DE8E57D4` | `3FA7D030DE8E57D4` | совпадает |
| 35 | `3FA7D104CF19FE6D` | `3FA7D104CF19FE6D` | совпадает |
| 36 | `3FA7D0C16EE89B57` | `3FA7D0C16EE89B57` | совпадает |
| 37 | `3FA7D0D936413FD0` | `3FA7D0D936413FD0` | совпадает |
| 38 | `3FA7D1DEF3A2C4D7` | `3FA7D1DEF3A2C4D7` | совпадает |
| 39 | `3FA7D1B3CCEC8EC9` | `3FA7D1B3CCEC8EC9` | совпадает |
| 40 | `3FA7D1AEA32B4E71` | `3FA7D1AEA32B4E71` | совпадает |

## Точность production summary

Production writer записывает этот массив через Rust `{:?}` для `Vec<f64>`.
В данном сохранённом `summary.json` все десять исходных JSON-токенов 31–40
при разборе дали ровно те же f64-биты, что и checkpoint. Следовательно, для
этого production-артефакта формат сохраняет round-trip f64 этих objectives;
округлённый CLI-лог не использовался.

Это подтверждает только побитовое равенство десяти сохранённых показателей
`flop_objective`. Оно не доказывает побитовое равенство остальных частей
состояния — assignments, центров, RNG, diagnostics, ground matrix или таблиц.
