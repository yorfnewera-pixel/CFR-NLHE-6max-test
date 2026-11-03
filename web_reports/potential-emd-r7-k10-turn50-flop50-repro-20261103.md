# Production repro R7/K10: budgets 10/50/50

Дата: 2026-11-03. Выполнен ровно один production repro успешного контроля
R7/K10. При расхождении, ошибке или timeout повтор не предусматривался; ни
одного повтора не выполнялось.

## Preflight и запуск

До build/run дерево было чистым. Commit до build и до launch одинаковый:
`9f21d7c2ab3fae0491d3431eb763fea12bfd63b7`. Сравнение вычислительного
кода (`native/hu20-buckets/src`, `Cargo.toml`, `Cargo.lock`) с исходным
контролем `d53e51aac40c3a883877cebc15a0962fbe29ee78` пусто.

Новый ранее отсутствовавший каталог:
`native/hu20-buckets/results/potential-emd-r7-k10-turn50-flop50-repro-20261103`.

| Параметр | Значение |
| --- | --- |
| Seed | `202610050003` |
| Ranks / K | `7` / `10` |
| Budgets river / turn / flop | `10` / `50` / `50` |
| Rayon-потоки | `2` |
| Final-assignment diagnostics | включены |
| Flop EMD profile | включён |
| Timeout | `600 с` |
| Реальный PID | `18800` (`hu20-buckets.exe`) |
| Реальный exit code | `0` |
| Финальный status | `complete` |
| Время генератора | `180.5 с` |

## Воспроизводимость

Полные массивы objectives из нового и исходного production `summary.json`
сравнены по каждому элементу через `f64::to_bits()`:

| Улица | Число objectives | Побитовая сверка с исходным контролем |
| --- | ---: | --- |
| River | 10 | совпадает |
| Turn | 29 | совпадает |
| Flop | 40 | совпадает |

Логи обоих запусков фиксируют для turn и flop
`termination=EarlyStopConsistent`, `final_assignment_consistent=true` и
final changed fraction `-0.000000000` (IEEE-754 signed zero, то есть
численно ноль).

| Таблица | SHA-256 исходного контроля и repro |
| --- | --- |
| River | `AED0CC04033CD2F1C61FCD947499A3AFA7FB3E17B653AB7EB6986E17759EBDE3` |
| Turn | `625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990` |
| Flop | `0BC243B8813600F6CBF0F69AFC68EC34734BF98970DD1992F7956F8934D3E9E8` |

## Ресурсы и завершающие проверки

Монитор реального PID записал максимум Private Bytes `65,421,312 B` и
Working Set `67,493,888 B`. Время и память отличаются от исходного контроля
и не трактуются как нарушение воспроизводимости. После завершения процессов
`hu20-buckets` и `cargo` нет. `git diff --check` успешно завершён.

Код, параметры, seed и таблицы существующего исходного контроля не менялись.
