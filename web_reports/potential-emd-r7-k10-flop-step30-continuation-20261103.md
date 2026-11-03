# Test-only continuation flop R7/K10: шаги 31–50

Дата: 2026-11-03. Выполнен ровно один supervised test-only continuation из
проверенного `checkpoint-30.bin`; river/turn и flop 1–30 не пересчитывались.
Production-таблицы не создавались, шаг 51 не выполнялся.

## Preflight и запуск

| Параметр | Значение |
| --- | --- |
| Commit continuation-кода | `1f8d730fb875f857107df711e47067d8a1bf9762` |
| Дерево до build/run | чистое |
| SHA-256 входного checkpoint-30 | `C65A5C0B011B6841D0BB0F0DDB23710938D4EDDE8EE1DF9A5DC80495A4B5FC65` |
| Входной fingerprint | `45E6F95C08AB346B` |
| Параметры | `ranks=7`, `K=10`, seed `202610050003`, Rayon `2` |
| Диапазон | только 31–50 |
| Timeout | `300 с` |
| Реальный exit code | `0` |
| Время целевого test | `213.69 с` |

До шагов continuation loader проверил точные embedded flop inputs, веса,
`turn_ground`, fingerprint, `K=10`, 48 769 ситуаций, 30 objectives и 30
diagnostics. Release discovery выбрал ровно один test
`tests::test_only_ranks7_flop_k10_continues_checkpoint_thirty_to_fifty`.

## Траектория

На шаге 39 final-assignment диагностика ещё дала
`0.00001221001221001`, тогда как ordinary moved fraction стала ниже порога
только на шаге 40 (`0.00001221001221001`). Test-only continuation всё равно
продолжался до явной границы 50. Немутирующая final-assignment диагностика
стала IEEE-754 signed zero на шаге 40:

| Шаг | Objective до / после update | Ordinary moved | Final changed |
| ---: | --- | ---: | ---: |
| 39 | `0.04652177692903831` / `0.04652162350428025` | `0.00048026048026048` | `0.00001221001221001` |
| 40 | `0.04652162305651764` / `0.04652155073258403` | `0.00001221001221001` | `-0.0` |
| 41–50 | `0.04652155073258403` / то же значение | `-0.0` | `-0.0` |

Полное state-word состояние (assignments, биты центров и RNG) точно
повторилось на переходе `40→41`; следовательно, это фиксированная точка, а
не только нулевая диагностика. Повторные state words далее также сохранялись.

Objective после update вырос на шагах `31, 32, 33, 34, 36, 37`. Это
зафиксировано как свойство арифметического обновления центров относительно
EMD-цели; само по себе не трактуется как ошибка EMD-решателя. Других циклов
до границы 50 не обнаружено.

## Checkpoint и ресурсы

После каждого шага записан атомарный capture-format checkpoint: всего 20
файлов `checkpoint-31.bin` … `checkpoint-50.bin`. Контрольные SHA-256:

| Checkpoint | SHA-256 |
| --- | --- |
| 40 | `75BD0FC3D2D51D110AAAF3EBC95063255111939EFD6715707B044246EEAB2865` |
| 41 | `C8E9F1468B2FF689EB96174A6F8535D54B6B80582DDAFCC91C8F51CE568E2F99` |
| 50 | `4D73D8316FC0672C14CD70B346068198EABD3697DED86B846DE84FCDBC171AEF` |

Хеши checkpoint различаются, поскольку они включают растущую историю
objectives/diagnostics, хотя state words после шага 40 повторяются.
Полный список SHA сохранён локально в `checkpoint-hashes.json`.

Внешний монитор записал 214 выборок: максимум Private Bytes — `3,366,912 B`,
Working Set — `9,601,024 B`. После run процессов `cargo` и test-бинарника
нет. При ошибке или timeout повтор не выполнялся; в этом запуске они не
произошли.
