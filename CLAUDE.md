# Контекст проекта

## Назначение и устройство

Это проект покерного агента no-limit Texas Hold'em: основная логика на Python, критичные по скорости компоненты на Rust. Заявления о силе требуют заранее объявленных парных экспериментов и доверительного интервала.

- `src/game/` — правила раздачи, легальные наблюдения и сессии.
- `src/blueprint/` — information keys, reference CFR и чтение average policy.
- `src/policies/` — проверка и загрузка закреплённых артефактов моделей.
- `src/arena/` — парная оценка и отчёты; `src/play_api/` и `apps/poker-web/` — локальная веб-игра.
- `native/hu20-trainer/` — нативное обучение и parity-проверки.
- `native/hu20-buckets/` — генерация card buckets, evaluator и potential-aware EMD.

Перед изменением правил игры или доступной игроку информации прочитайте `docs/rules.md` и `docs/observations.md`.

## Команды проверки

Из корня репозитория:

```powershell
python -m pytest -q
python -m scripts.check_game --players 6 --hands 20
python -m scripts.check_session --hands 30
```

После `git add` запускайте проверку staged-артефактов:

```powershell
python -m scripts.check_repository_artifacts
```

Для изменения bucket builder:

```powershell
cargo fmt --manifest-path native/hu20-buckets/Cargo.toml --check
cargo test --manifest-path native/hu20-buckets/Cargo.toml
cargo build --release --manifest-path native/hu20-buckets/Cargo.toml
```

## Potential-Aware EMD

Источник требований: `articles/Potential-Aware EMD/План Sol.txt`. Главный код — `native/hu20-buckets/src/lib.rs`, CLI и артефакты — `native/hu20-buckets/src/main.rs`.

- Не менять метрику, seed, веса, ключи или правило центра без отдельного согласованного задания.
- Turn использует точный одномерный Wasserstein-1; не применять эту оптимизацию к flop, где ground matrix состоит из EMD между turn-центрами.
- Общий EMD обязан завершаться диагностической ошибкой при нарушении инвариантов, а не возвращать приближённое расстояние или пропускать пару.
- Assignment использует EMD, а update центра — арифметическое взвешенное среднее. Это эвристика: objective может увеличиваться; не заменяйте центр на medoid/барицентр незаметно.
- Различайте objective до update, после update с прежними assignment и после следующего reassignment.

Контрольный запуск выполняется только вручную через:

```powershell
& .\native\hu20-buckets\scripts\run-potential-measurement.ps1 `
  -OutputDirectory .\results\<новый-пустой-каталог> `
  -Ranks 4 -K 10 -Iterations 10 -Threads 2 -TimeoutSeconds 120
```

Launcher требует чистое Git-дерево, сохраняет полный SHA, команду, stdout/stderr, память, статус и SHA-256 таблиц. Не запускайте K=20, `ranks>=5`, полный 52-карточный прогон, обучение CFR или GPU без отдельного решения. `results/` игнорируется Git; храните отчёты в `web_reports/`, но не коммитьте таблицы, модели, checkpoints, логи или credentials.

## Текущее состояние EMD

Общий solver уже исправлен и защищён от циклов предков; diagnostics сохраняют контекст ошибочной EMD-пары. Воспроизводимый K=10 завершился с `exit_code=0`, `status=complete` и полным SHA `8d3a78fca9ca00286abe51b295f63ffea2276122`; SHA-256 river/turn/flop совпали с первоначальным K=10. Результаты: `web_reports/potential-emd-k10-repro-20261101.md`; аудит реализации: `web_reports/potential-emd-audit-20261101.md`.

Не повторяйте K=10, если EMD-код не менялся. Исключение: разрешён один согласованный диагностический контроль с `-DiagnoseFinalAssignments` для проверки согласованности финальных assignment с центрами; он не является сравнительным замером производительности и не меняет алгоритм или таблицы. Если EMD-код менялся, сначала выполните тесты и создайте новый пустой каталог результатов. Известное ограничение: mean-update может повысить EMD-objective; это подтверждённое свойство эвристики, не повод молча менять алгоритм. Следующий выбор — отдельное ТЗ: оставить эвристику либо исследовать medoid/EMD-барицентр с оценкой стоимости и корректности.

## Изменения

Перед реализацией нетривиальной задачи сначала составьте план и согласуйте границы. Для bugfix/изменения поведения: воспроизводящий тест → минимальная правка → целевой тест → полный релевантный набор. Не переписывайте работающую логику без явного основания.

Коммиты должны описывать фактическое изменение. Не делайте push, релиз, крупный/платный эксперимент, изменение game scope, default-модели или публичное заявление о силе без явного разрешения владельца.
