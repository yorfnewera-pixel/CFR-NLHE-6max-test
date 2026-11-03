# Production-контроль R7/K10: новый gate остановки turn

Дата: 2026-11-03. Выполнен ровно один production-запуск в новом пустом
каталоге `native/hu20-buckets/results/potential-emd-r7-k10-turn50-flop30-gate-control-20261103`.
Повторов, другого seed, изменения budgets или production-кода после него не
было.

## Параметры и provenance

| Параметр | Значение |
| --- | --- |
| Commit до сборки и запуска | `80895882faf3fbfa293bf1553d124fc21b185566` |
| Дерево до сборки и запуска | чистое |
| `summary.code_revision` / `working_tree_dirty` | тот же commit / `false` |
| Параметры | `ranks=7`, `K=10`, общий budget `10`, turn `50`, flop `30`, seed `202610050003`, Rayon `2` |
| Флаги | `--diagnose-final-assignments --profile-flop-emd` |
| Timeout | `600 с` |
| Реальный exit code / status | `0` / `complete` |

Release-сборка launcher завершилась до запуска; `launcher-context.txt`
содержит успешные `after_wait_for_exit`, `after_receive_job` и
`after_refresh` с `exit_code=0`. После завершения процессов `hu20-buckets` и
`cargo` нет; failure-dump отсутствует.

## Время и память

| Метрика | Значение |
| --- | ---: |
| Общее время генератора | `144.8 с` |
| River: equity → запись | около `0.6 с` |
| Turn features | около `0.9 с` |
| Turn clustering и диагностика | около `1.9 с` |
| Flop clustering (профиль) | `140.900680500 с` |
| Final flop diagnostic | `3.934951800 с`, `487 690` EMD-вызовов |
| Максимум Private Bytes (145 внешних выборок) | `79 835 136 B` |
| Максимум Working Set (145 внешних выборок) | `80 793 600 B` |

## Turn: проверка нового правила остановки

Turn прошёл историческую пороговую точку на шаге 28:

| Шаг | Objective до / после update | Moved weight fraction |
| ---: | --- | ---: |
| 28 | `0.06261840646662956` / `0.06261799722102224` | `0.000053087` |
| 29 | `0.06261799722102224` / `0.06261780246870059` | `0.000019111` |

Полные objectives 1–28 побитово совпали с прежним R7/K10 production-контролем.
Значения шага 29 совпали с проверенным test-only continuation. Финальная
немутирующая диагностика после шага 29 дала objective
`0.06261780246870059` и IEEE-754 signed zero (`-0.0`) для changed weight
fraction. Причина остановки — `EarlyStopConsistent`, а
`final_assignment_consistent=true`; эти признаки зафиксированы отдельно.

Это подтверждает production-путь нового gate для данного входа. Результат не
доказывает общую сходимость или монотонность objective.

## Flop

Flop исчерпал явный budget `30`, а не остановился по раннему порогу. На
последней итерации objective до/после update равен
`0.04644749949229097` / `0.046484311` (точность лога), moved weight fraction —
`0.007887668`. Финальная диагностика: objective `0.046481510`, changed
weight fraction `0.005177045`. Flop не изменялся и его budget не повышался.

## Хеши таблиц

| Таблица | SHA-256 | Сравнение |
| --- | --- | --- |
| river | `AED0CC04033CD2F1C61FCD947499A3AFA7FB3E17B653AB7EB6986E17759EBDE3` | совпадает с прежним R7/K10 production-контролем |
| turn | `625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990` | ожидаемо изменился после 29-го шага turn |
| flop | `209EA342311EEED8AE9367E90B3123DD62F696ACDEE4EF8811BE5F72D8F39485` | изменился вследствие нового входного turn; совпадение не требовалось |

Сырые таблицы, `summary.json`, логи, `memory.csv` и `table-hashes.json`
сохранены только в указанном локальном каталоге результатов.
