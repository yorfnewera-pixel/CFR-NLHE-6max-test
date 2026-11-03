# Inspection/preflight: test-only continuation flop R7/K10 из шага 30

Дата: 2026-11-03. Выполнена только read-only инспекция последнего
production-контроля
`results/potential-emd-r7-k10-turn50-flop30-gate-control-20261103` на commit
`a2f21291b581247a30af46243adefd7b4eccea34` (результаты были получены на
`80895882faf3fbfa293bf1553d124fc21b185566`). Генератор, ignored-тесты,
replay и код не запускались и не изменялись.

## Артефакты gate-control

В каталоге есть три production-таблицы, `summary.json`, `stderr.log`,
`memory.csv`, `table-hashes.json`, `status.json`, `command.txt` и
`launcher-context.txt`. `summary.json` подтверждает `flop_iterations=30`,
последний objective `0.04644749949229097`, revision
`80895882faf3fbfa293bf1553d124fc21b185566`, а `status.json` — `complete`.
`launcher-context.txt` содержит фактический `exit_code=0`.

Checkpoint шага 30, каталог checkpoint, trace полного состояния и SHA-256
такого checkpoint отсутствуют. Поиск локальных checkpoint находит только
артефакты test-only continuation turn R7/K10 и прежних R6/K20 flop-probe;
они не являются состоянием flop текущего gate-control.

## Привязка к новым входам flop

Новый turn этого контроля завершился на шаге 29 с
`EarlyStopConsistent` и `final_assignment_consistent=true`. Его production
table имеет SHA-256
`625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990`; flop
построен именно на этих назначениях и связанных turn-центрах/ground matrix.
Однако production-формат таблицы записывает только street, K, keys и
assignments. Поэтому таблица turn не сохраняет:

- биты turn-центров, из которых был построен `turn_ground`;
- саму ground matrix;
- flop distribution и веса в порядке stepper;
- RNG-state, assignments и биты центров flop после шага 30;
- полную final diagnostic flop (в логе она округлена до девяти знаков).

`summary.json` сохраняет objectives, но не восполняет эти данные. Значит,
точно загрузить состояние шага 30 либо сверить его с исходным контролем без
нового допуска невозможно.

## Существующий test-only путь

`TestOnlyEmdCheckpoint` технически умеет валидировать размерности, K,
отпечаток точных входов, assignments, биты центров, RNG, objective и полную
историю diagnostics. Но все существующие flop-test-only probe и resume
фиксированы на `ranks=6`, `K=20` и их собственных fingerprints. Для R7/K10
нет runner, тестового fixture, ожидаемого checkpoint-30 или continuation
window `31–50`.

Turn checkpoint R7/K10 также непригоден: он сериализует Wasserstein-1D
состояние turn на шаге 28, а не EMD-state flop после шага 30.

## Вывод и минимальный следующий шаг

Продолжение шагов 31–50 сейчас **нельзя** выполнять корректно. Запуск с
текущими локальными данными был бы replay с нуля, а не continuation, и не
позволил бы доказать точное восстановление step-30 state.

Минимальный отдельный шаг, требующий нового согласования:

1. Добавить test-only R7/K10 capture-path, совместимый с production
   `kmeans_emd` flop и новым turn termination gate.
2. В единственном supervised запуске восстановить необходимый river/turn
   префикс, строго сверить objectives flop 1–30 с `summary.json` без
   расширения допуска, а затем атомарно сохранить `checkpoint-30.bin` с
   полным состоянием и input fingerprint. Не продолжать за 30.
3. Только при успешной загрузке checkpoint и точном сравнении считать
   допустимым отдельный continuation 31–50: `RAYON_NUM_THREADS=2`, timeout
   `300 с`, новый пустой каталог, проверка SHA-256 checkpoint, exact input
   fingerprint, final assignment diagnostic и отсутствие оставшихся
   процессов.

Ни этот capture, ни continuation автоматически не выполнялись.
