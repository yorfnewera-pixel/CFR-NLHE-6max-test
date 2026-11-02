# Отдельный бюджет итераций turn для Potential-Aware EMD

Дата: 2026-11-03. Добавлена минимальная настройка бюджета только для turn,
чтобы следующая полная проверка могла оставить river и flop на прежнем общем
лимите. Генератор в этой задаче не запускался.

## Контракт

- CLI: `--turn-iterations N`, где `N > 0`.
- Launcher: `-TurnIterations N`; значение по умолчанию `0` означает, что флаг
  CLI не передаётся.
- Без нового флага turn получает прежний общий `--iterations`; поведение
  river и flop не изменяется.
- При явном значении только вызов turn-кластеризации получает `N`; river и
  flop продолжают получать общий `--iterations`.
- `summary.json` теперь содержит `iterations`, `turn_iterations` и
  `turn_iterations_explicit`. Это фиксирует фактические бюджеты, но не
  объявляет исчерпание любого из них сходимостью.

Пример следующего согласованного контроля, который **не выполнялся**:

```powershell
& .\native\hu20-buckets\scripts\run-potential-measurement.ps1 `
  -OutputDirectory .\results\<new-directory> `
  -Ranks 5 -K 20 -Iterations 10 -TurnIterations 50 -Threads 2 `
  -Seed 202610050003 -TimeoutSeconds 120 `
  -DiagnoseFinalAssignments -ProfileFlopEmd
```

## Регрессии и корректировка документации

Добавлены два unit-теста разбора CLI: default без override даёт `(10, 10,
false)`, а `--turn-iterations 50` при `--iterations 10` — `(10, 50, true)`.

Ignored test-only probe `ranks=5, K=20` усилен assertions на установленную
fixture: стабилизация за 17 итераций, нулевая финальная смена назначений,
objective `0.032710700` и первое повторение полного состояния между 16 и 17
итерациями. Он не относится к production-пути и не запускает flop clustering.

В отчёте контроля K=20 исправлена подпись: `0.033152026` — objective **до**
последнего turn update, `0.032806550` — после него, а `0.032777321` —
финальный reassignment.

## Проверки

- RED: до реализации тесты разбора не компилировались, потому что
  `iteration_limits` отсутствовала.
- GREEN: оба новых CLI unit-теста проходят.
- `cargo test --release test_only_ranks5_turn_k20_trajectory_to_fifty_iterations -- --ignored --nocapture` — пройден.
- Полный `cargo fmt --check`, `cargo test` и PowerShell parser launcher
  выполняются перед передачей изменения.

Production-код кластеризации, метрика, epsilon, обновление центров и финальный
reassignment не менялись.
