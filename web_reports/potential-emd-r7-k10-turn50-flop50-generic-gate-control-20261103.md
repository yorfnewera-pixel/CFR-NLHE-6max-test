# Production-контроль R7/K10 generic final-assignment gate

Дата: 2026-11-03. Выполнен ровно один production-control; повторов не было.

## Preflight и provenance

До запуска проверены исходный production gate-control `10/50/30`, capture
flop-30 и raw trace test-only continuation 31–50. Новый каталог результатов
до старта отсутствовал.

| Параметр | Значение |
| --- | --- |
| Commit до build/launch | `d53e51aac40c3a883877cebc15a0962fbe29ee78` |
| Дерево до build/launch | чистое |
| Параметры | `ranks=7`, `K=10`, seed `202610050003`, budgets `10/50/50`, Rayon `2` |
| Timeout launcher | `600 с` |
| Final diagnostics / flop EMD profile | включены |
| PID / процесс | `22728` / `hu20-buckets.exe` |
| Реальный exit code | `0` |
| Финальный status | `complete` |
| Время полного контроля | `182.7 с` |

Release build завершился до измерения. Внешняя оболочка на короткое время
потеряла ожидание launcher, но тот же PID продолжал работу; его локальный
supervisor затем записал `wait_for_exit=True` и `exit_code=0`. Новый процесс
и повтор не создавались.

## Сверки

| Проверка | Результат |
| --- | --- |
| River objectives 1–10 с gate-control | побитовое совпадение |
| Turn objectives 1–29 с gate-control | побитовое совпадение |
| Turn шаг 29 | `EarlyStopConsistent`, `final_assignment_consistent=true`, final changed `-0.0` |
| Flop objectives 1–30 с gate-control | побитовое совпадение |
| Flop 31–40 с continuation | одинаковые записанные показатели в одинаковых итерациях |
| Flop шаг 40 | `EarlyStopConsistent`, `final_assignment_consistent=true`, final changed `-0.0` |

Для continuation raw trace хранит decimal-представления, не все из которых
содержат достаточно значащих цифр для round-trip одного f64-бита (например,
шаг 33). Поэтому побитовое утверждение для шагов 31–40 по этому trace
недоступно без изменения исторического формата. Их записанные значения
совпадают; новых допусков не вводилось.

## SHA-256 таблиц

| Таблица | Контроль `10/50/30` | Новый `10/50/50` |
| --- | --- | --- |
| River | `AED0CC04033CD2F1C61FCD947499A3AFA7FB3E17B653AB7EB6986E17759EBDE3` | тот же |
| Turn | `625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990` | тот же |
| Flop | `209EA342311EEED8AE9367E90B3123DD62F696ACDEE4EF8811BE5F72D8F39485` | `0BC243B8813600F6CBF0F69AFC68EC34734BF98970DD1992F7956F8934D3E9E8` |

Новый flop hash ожидаемо отличается: теперь таблица отражает финальное
состояние шага 40, а историческая production-таблица была записана после
шага 30.

## Профиль и ресурсы

Flop EMD profile: wall clock `178.965273600 с`; 40 итераций. Gate final
assignment: `3.585217600 с`, `487690` EMD-вызовов. Отдельная финальная
диагностика отсутствует (`null`), потому что финальный gate-result
переиспользован для opt-in вывода.

Монитор реального PID сохранил 183 выборки. Максимумы: Private Bytes
`80,252,928 B`, Working Set `81,203,200 B`. После контроля процессов
`hu20-buckets` и `cargo` не осталось.

Код, метрика, EMD-решатель, центры, tie-breaking, seed, допуски и budgets
не менялись. После этой единственной попытки generator, capture/replay,
continuation и иные запуски не выполнялись.
