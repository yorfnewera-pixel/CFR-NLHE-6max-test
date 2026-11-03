# Test-only Wasserstein-1D stepper — design

## Intent

Создать test-only пошаговый executor, который численно повторяет production
turn-кластеризацию `kmeans_wasserstein_1d_with_diagnostics`. Он нужен для
строгого replay и checkpoint/continuation без изменения production-пути.

## Goals

- Воспроизводить production-обработку координат: сортировку, перестановку
  строк и обратную перестановку центров.
- Сохранить порядок RNG, выбор минимума, арифметическое обновление центров,
  поведение пустых кластеров, диагностику objective и немутирующую финальную
  проверку.
- Хранить backend-id и полный fingerprint входов, включая координаты, в
  test-only checkpoint; несовместимые форматы или backend отклонять явно.
- Доказать точное совпадение на малых fixtures через assignments, биты
  центров, RNG и все доступные diagnostics.

## Non-goals

- Любые изменения production `kmeans_wasserstein_1d_with_diagnostics`,
  `wasserstein_1d`, launcher, метрик, допусков, остановки или таблиц.
- Запуск ranks=7, flop, continuation либо production-generator.
- Интерпретация старого transport-EMD checkpoint как Wasserstein checkpoint.

## Approaches

1. Отдельный test-only `Wasserstein1dStepper` повторяет production-код. Он
   выбран: production остаётся неизменным, а checkpoint сохраняет точное
   состояние replay backend.
2. Общий production engine с параметризованным kernel отвергнут: это
   production-рефакторинг за пределами задачи.
3. Общий transport-EMD stepper отвергнут: он не воспроизводит численный путь
   production Wasserstein-1D.

## Design

В `#[cfg(test)]` рядом с существующим `TestOnlyEmdStepper` появится отдельный
`TestOnlyWasserstein1dStepper`. В конструкторе он сформирует production-
совместимые `order`, `sorted_coordinates` и `sorted_points`; initial centroids
будут выбраны тем же `Rng`. Один шаг использует
`wasserstein_1d_sorted`, идентичный tie-breaking `min_by`, ту же
последовательную агрегацию весов и центров, а также те же точки измерения
diagnostics. Финальная диагностика только читает состояние.

Checkpoint получит версию/идентификатор backend и fingerprint, рассчитанный
из backend-id, точек, весов, K и исходных координат. Загрузка принимает
ожидаемый backend-id; checkpoint transport-EMD или с другими координатами
отклоняется до восстановления состояния.

## Verification

Малый fixture сравнивает каждый шаг нового stepper с production 1D результатом
побитово для доступных f32/f64 метрик, assignments, центров и RNG. Отдельный
round-trip сравнивает непрерывную и восстановленную траектории. Негативные
тесты проверяют отказ при другом backend и координатах. Существующие тесты
общего EMD-stepper не меняются.
