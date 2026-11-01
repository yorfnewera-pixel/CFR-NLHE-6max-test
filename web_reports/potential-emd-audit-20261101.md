# Аудит Potential-Aware EMD перед воспроизводимым K=10

## Статус исходного плана

| Область | Статус | Доказательство |
| --- | --- | --- |
| Этапы и атомарный `status.json` | Выполнено | `main.rs`, `atomic_write_replaces_status_without_partial_content` |
| Общий EMD и защита от зацикливания | Выполнено | `lib.rs`, тесты цикла предков и независимого транспорта |
| Точный W1 тёрна | Выполнено | `wasserstein_1d_*`, W1 применяется только в turn-ветке |
| `mean_equity` в potential-признаках | Выполнено | `PotentialFeatures` содержит только `keys`, `weight`, `distribution`, `dimension`; `mean_equity` принадлежит исключительно equity-ветке `StreetFeatures` |
| Objective-диагностика | Выполнено | до update, после update и после следующего reassignment связаны через `objective_after_reassignment` |
| Launcher | Выполнено до контрольного запуска | чистое дерево, process tree timeout, status/tables/hashes validation, stdout/stderr |
| Полный идентификатор версии | Выполнено до контрольного запуска | полный SHA + `working_tree_dirty` в `summary.json` |

## Причина роста objective

`kmeans_emd()` назначает точки по EMD, но обновляет центр арифметическим взвешенным средним компонент. Центр остаётся допустимым нормированным распределением, однако не является общим EMD-барицентром и потому не гарантирует убывание EMD-цели.

Тест `arithmetic_mean_can_increase_emd_objective` фиксирует минимальный пример. Это математическое ограничение текущей эвристики, не скрытая замена алгоритма и не доказательство ошибки общего EMD-solver.

## Измерения objective

- `objective_before_update`: после assignment к прежним центрам.
- `objective_after_update`: с теми же assignment после арифметического update центров.
- `objective_after_reassignment`: objective следующего assignment к обновлённым центрам; для последней итерации `None`, поскольку дополнительный дорогой проход не выполняется только ради диагностики.

Все три значения используют те же EMD, веса и знаменатель суммарного веса.

## Проверка перед запуском

- `cargo test --manifest-path native/hu20-buckets/Cargo.toml`: 27 активных тестов passed, 1 ignored.
- `cargo build --release --manifest-path native/hu20-buckets/Cargo.toml`: success.
- PowerShell AST launcher: success.

## Следующий шаг

Сделать commit текущих изменений, удостовериться в чистом `git status -sb`, затем вручную выполнить **один** K=10 через launcher в новом пустом каталоге. K=20 не запускать.
