# Inspection: production gate финальной согласованности flop R7/K10

Дата: 2026-11-03. Выполнена только статическая inspection production-кода и
сверка сырого test-only trace; генератор не запускался, production-код не
менялся.

## Вывод

Существующий gate финальной согласованности **не действует для production
flop**. Он реализован только в
`kmeans_wasserstein_1d_with_diagnostics()` для turn: после пороговой обычной
смены назначений функция выполняет немутирующую final-assignment диагностику
и прекращает работу лишь при точном нуле.

Production flop использует `kmeans_emd_profiled_with_assignment()`. В нём
порог `moved / total < 1e-4` непосредственно выполняет `break`; opt-in
final-assignment диагностика вычисляется уже после выхода из цикла и не
управляет остановкой. Возвращаемые `termination` и
`final_assignment_consistent` для этого пути сейчас остаются `None`/`false`.
Следовательно, production-контроль R7/K10 с budgets `10/50/50` запускать
нельзя: он мог бы принять пороговую остановку flop при ненулевой финальной
смене назначений.

## Минимальный требуемый патч

1. В `kmeans_emd_profiled_with_assignment()` при пороговой ordinary moved
   fraction вызвать ту же немутирующую final-assignment диагностику до
   решения об остановке. При ненулевой final changed fraction продолжать
   следующую итерацию без скрытого update; после следующего update stale
   gate-результат сбрасывать.
2. Записывать причину (`EarlyStopConsistent` либо
   `IterationBudgetExhausted`) и `final_assignment_consistent` для generic
   EMD, независимо от флага вывода диагностики. Флаг влияет только на
   публикацию diagnostic, а не на gate.
3. Переиспользовать уже вычисленный final gate-result как opt-in output и
   учесть его EMD-вызовы в profile. Вывести termination/consistency для flop
   в launcher-логе, чтобы production-контроль мог их проверить.

Production-метрика, центры, tie-breaking, допуски, seed и budgets при этом
не меняются.

## Необходимые регрессии

- Generic EMD: пороговая ordinary moved fraction при ненулевой финальной
  смене не останавливает цикл и не выполняет скрытый update.
- Generic EMD: exact zero финальной смены даёт `EarlyStopConsistent`.
- Исчерпание budget при согласованных назначениях даёт
  `IterationBudgetExhausted` и `final_assignment_consistent=true`.
- Прямой побитовый snapshot assignments/центров до и после generic
  final-assignment helper подтверждает немутирующий контракт.
- Включение opt-in diagnostic не меняет assignments, центры, objectives,
  termination или consistency.

## Исправление trace-отчёта

Сырой `flop-trace.log` показывает: на шаге 39 ordinary moved равен
`0.00048026048026048`, а final changed — `0.00001221001221001`; ordinary
значение `0.00001221001221001` относится к шагу 40. Описание в continuation
отчёте и журнале исправлено. Таблица отчёта уже содержала правильные числа.
