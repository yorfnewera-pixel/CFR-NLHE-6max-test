# Generic EMD final-assignment gate

Дата: 2026-11-03. Область задачи — только production-код и регрессии.

## Изменение

`kmeans_emd_profiled_with_assignment()` теперь при прежнем пороге ordinary
`moved_weight_fraction < 1e-4` выполняет немутирующую проверку final
assignment. Ранняя остановка возможна только когда
`changed_weight_fraction == 0.0`; это намеренно принимает IEEE-754 `-0.0` и
не вводит новый epsilon. Ненулевая финальная смена оставляет обычный цикл
продолжаться без дополнительного update или переназначения.

Каждая итерация сбрасывает cache проверки до изменения assignment/центров.
На выходе проверяется актуальное финальное состояние независимо от opt-in
вывода диагностики. Причина `IterationBudgetExhausted` сохраняется при
исчерпании бюджета даже в согласованном финальном состоянии.

`EmdClusteringProfile` разделяет стоимость проверок:

- `final_assignment_gate_*` — все проверки, вызванные пороговым gate;
- `final_assignment_diagnostic_*` — отдельная проверка на выходе, только если
  актуальный результат gate отсутствовал.

Запрошенный вывод final diagnostic переиспользует вычисленный результат и не
вызывает второй EMD-проход. CLI записывает flop termination/consistency
независимо от opt-in и публикует обе части профиля в `summary.json`.

Проверены generic call sites: публичные `kmeans_emd*`, test-only stepper и
единственный production caller flop в `src/main.rs`. Метрика, EMD-решатель,
centroid update, tie-breaking, seed, допуски и бюджеты не менялись.

## Регрессии

Добавлены проверки generic EMD для:

- порогового ordinary moved с ненулевой final changed fraction: цикл идёт до
  следующего состояния без скрытой мутации;
- точного нуля final changed fraction и `EarlyStopConsistent`;
- budget exhaustion с согласованным и несогласованным финальным состоянием;
- побитовой неизменности assignments и центров от diagnostic helper;
- идентичности результатов, termination и consistency при включённом и
  выключенном выводе, включая битовые поля iteration diagnostics;
- сброса cache после изменения состояния: профиль фиксирует два gate-вызова,
  а не повторно использует предыдущий результат.

## Проверка

Выполнены:

```text
cargo test --manifest-path native\hu20-buckets\Cargo.toml generic_emd_
cargo fmt --manifest-path native\hu20-buckets\Cargo.toml
cargo test --manifest-path native\hu20-buckets\Cargo.toml
cargo fmt --manifest-path native\hu20-buckets\Cargo.toml --check
git diff --check
```

Targeted набор: 4 passed. Полный набор: 70 library + 12 CLI passed; 13
явно ignored test-only сценариев не запускались. `fmt --check` и
`git diff --check` завершились успешно. Генератор, capture/replay и
continuation не запускались.
