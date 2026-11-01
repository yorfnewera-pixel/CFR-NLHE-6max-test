# Контрольный прогон Potential-Aware EMD: K=10

## Статус

- Каталог результата: `results/potential-emd-k10-control-20261101`
- Финальный `status.json`: `complete` — `potential-aware tables written`
- EMD-диагностических файлов `emd-failure-*.txt` нет.
- Генератор завершил построение всех трёх таблиц. PowerShell-launcher ошибочно сообщил пустой `ExitCode`, но это ошибка обвязки после завершения процесса, а не ошибка генератора.

## Параметры

| Параметр | Значение |
| --- | --- |
| Abstraction | `potential` |
| Ranks / deck cards | 4 / 16 |
| K | 10 |
| Максимум итераций | 10 |
| Фактически flop-итераций | 8 |
| Rayon-потоков | 2 |
| Seed | `202610050003` |
| Время генератора | 2.6 с |

Команда:

```text
hu20-buckets.exe build --out results/potential-emd-k10-control-20261101 --ranks 4 --ks 10 --iterations 10 --seed 202610050003 --abstraction potential
```

## Память

`memory.csv` содержит 4 измерения.

| Показатель | Максимум |
| --- | ---: |
| Private Bytes | 3,227,648 B (≈3.08 MiB) |
| Working Set | 5,947,392 B (≈5.67 MiB) |

## Хеши таблиц SHA-256

| Таблица | SHA-256 |
| --- | --- |
| `potential-river-k10.bin` | `F8DD9AB754AF1E091246A6FE220BA8A6399E71E04A5C2298B1F8F9925A62AE17` |
| `potential-turn-k10.bin` | `EFC9CA76DBF301519A8F8337D10610910E13619D2124FBFA627B5B869826BED5` |
| `potential-flop-k10.bin` | `8494906B562181683629EF2C6EC78C4E2B355F147A41DB3FA222E184450AACC6` |

## Objective

| Улица | Значения objective до обновления центра |
| --- | --- |
| River | 0.0315332840 → 0.0284718676 → 0.0276884715 → 0.0276905424 |
| Turn | 0.0767265282 → 0.0739610554 → 0.0697873567 → 0.0693904618 |
| Flop | 0.0340480963 → 0.0396608689 → 0.0396318107 → 0.0393036230 → 0.0390724176 → 0.0383339132 → 0.0373468135 → 0.0373213092 |

### Важное наблюдение

После первого flop-update objective вырос с `0.0340480963` до `0.0401351010` (значение после update; следующее значение до assignment — `0.0396608689`). Это не сбой общего EMD-solver: текущий `kmeans_emd()` назначает точки по EMD, но обновляет центр арифметическим взвешенным средним распределений. Такой центр валиден, однако не обязан уменьшать EMD-цель. Это отдельный вопрос шага проверки корректности кластеризации; алгоритм не менялся.

## Воспроизводимость

`summary.json` записал `code_revision: 7cbc0f6`, потому что запуск состоялся до commit текущего рабочего состояния. После прогона изменения были зафиксированы и опубликованы как `04ce008f2903b99e2ed1ca03aa9888673a455810` на `main`.

Для следующего прогона `summary.json` уже будет содержать актуальный commit. Артефакты этого прогона не находятся в Git, так как `results/` игнорируется.

## Полезные исходные файлы

- `results/potential-emd-k10-control-20261101/summary.json`
- `results/potential-emd-k10-control-20261101/status.json`
- `results/potential-emd-k10-control-20261101/memory.csv`
- `results/potential-emd-k10-control-20261101/table-hashes.json`
- `results/potential-emd-k10-control-20261101/stderr.log` — полный лог; не нужен для обычного обзора.
