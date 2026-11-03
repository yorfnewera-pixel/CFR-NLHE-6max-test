# EmdScratch: эквивалентность и ограниченный microbenchmark

Дата: 2026-11-03. Реализована минимальная оптимизация allocation hot path точного EMD. Генератор, тяжёлые probe, production-контроль, новые размеры, K и seed не запускались.

## Изменение

`try_emd()` сохранил публичный контракт, но делегирует внутреннему solver-у с `EmdScratch`. Scratch хранит только временные буферы: нормализованные распределения, residual-граф, потенциалы, Dijkstra-массивы, путь и visited-флаги. Перед каждым вызовом логическое содержимое полностью сбрасывается; сохраняется только allocation capacity.

В `kmeans_emd_profiled_with_assignment()` scratch локален Rayon-задаче в assignment и final-assignment diagnostic, а в последовательной фазе objective-after-update используется один локальный scratch. Нет `static`, `Mutex`, кэша EMD-результатов, центроидов, assignment или ground matrix. Порядок `0..K`, строгое сравнение в `first_minimum_emd()`, арифметические операции solver-а, допуски, проверки и правила k-means не менялись.

Прежнее ядро из commit `bbca55febbcf63c483ac829b8241685b751a3779` сохранено только под `#[cfg(test)]` как `try_emd_pre_scratch_reference`.

## Проверки эквивалентности

Новый test сравнивает reference, reusable scratch и публичный wrapper по `f32::to_bits` на разных размерностях, sparse/dense распределениях и повторном вызове после ошибки. Ошибки сравниваются по точному варианту и всем контекстным полям; floating-point вход с `NaN` сравнивается по битам, потому что обычный `PartialEq` намеренно считает `NaN != NaN`.

Существующая побитовая проверка streaming assignment против Vec-reference clustering сохраняет assignment, биты центроидов, objective, diagnostics, tie-breaking, signed zero и final diagnostic. Полный `cargo test` завершился: 64 passed, 0 failed, 8 ignored.

## Test-only release microbenchmark

Сборка release test-бинаря выполнена до измерения. Запущен только ignored test `tests::test_only_emd_scratch_microbenchmark`:

```text
cargo test --release tests::test_only_emd_scratch_microbenchmark -- --ignored --exact --nocapture
```

Параметры: фиксированные допустимые dense distributions и `20 × 20` ground matrix; 128 прогревочных вызовов; 3 чередующихся раунда reference/scratch по 5 000 вызовов; `black_box`; внутренний timeout 20 с.

| Путь | Вызовов | Время | ns/call |
| --- | ---: | ---: | ---: |
| Независимый прежний reference | 15 000 | 2,568020000 с | 171 201,333 |
| Reusable scratch | 15 000 | 2,426589900 с | 161 772,660 |

Оба пути завершили весь запланированный объём значительно раньше timeout. На этом synthetic microbenchmark scratch быстрее примерно на 5,5 %. Это измерение transport-kernel на одном фиксированном входе, не доказательство ускорения flop clustering или полного генератора.

## Границы вывода

Оптимизация не меняет метрику, центры, критерии остановки либо параметризацию генератора. Следующий production-контроль возможен только после отдельного согласования; текущий результат ограничен тестами эквивалентности и test-only microbenchmark.
