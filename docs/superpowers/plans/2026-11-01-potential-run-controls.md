# Potential Run Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Сделать potential-aware генератор наблюдаемым, безопасным для локальных измерений и быстрее на тёрне без изменения его метрики.

**Architecture:** `main.rs` владеет границами этапов, атомарным статусом и записью артефактов; библиотека публикует точный одномерный Wasserstein-1 и диагностические данные кластеризации. Быстрый EMD применяется лишь для тёрна, где ground — расстояние между equity-координатами; общий потоковый EMD остаётся эталоном и используется на флопе.

**Tech Stack:** Rust 2021, Rayon, PowerShell 7.

**Spec:** Пользовательское ТЗ от 2026-11-01 (вложение «Вставленный текст.txt»).

## Global Constraints

- Не запускать полный прогон или тяжёлые эксперименты автоматически.
- Не менять метрику флопа, seed, переходные распределения, веса или ключи.
- Скрипт запуска использует два Rayon-потока по умолчанию и отказывается от непустого каталога результатов.
- Все новые алгоритмические функции получают unit-тесты RED→GREEN.

## Review Focus

- Несортированные или совпадающие координаты river-центров: быстрая формула должна сохранять точность общего EMD.
- Ошибка нормировки `f32`: расстояние и objective остаются конечными.
- Пустой каталог результатов и тайм-аут: процесс не остаётся запущенным.
- Запись `status.json`: потребитель не видит частично записанный JSON.
- Пустые и повторяющиеся центры: диагностика явно показывает состояние, не скрывая его.

---

### Task 1: Точный одномерный EMD тёрна

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: `wasserstein_1d(left, right, coordinates) -> f32` и упорядочение распределений по координатам.

- [ ] Написать падающие тесты для идентичных распределений, переноса единичной массы, симметрии, нулевых компонент, неравных/совпадающих координат и случайных нормированных распределений против `emd`.
- [ ] Запустить `cargo test wasserstein --lib` и удостовериться, что тесты не компилируются до реализации.
- [ ] Реализовать точную накопительную формулу W1 с нормировкой, сортировкой координат и `f64`-аккумуляцией.
- [ ] Повторить `cargo test wasserstein --lib`; затем `cargo test --lib`.

### Task 2: Диагностика potential-кластеризации и удаление лишней equity

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: история итераций EMD-кластеризации с objective до/после обновления, долей перемещённой массы и числом занятых бакетов.

- [ ] Написать падающий тест целостности диагностических значений на малом наборе распределений.
- [ ] Запустить целевой тест до реализации.
- [ ] Добавить историю итераций; убрать `PotentialFeatures.mean_equity` и не вычислять river equity в ветке `board_size == 3`.
- [ ] Проверить целевой тест и `cargo test --lib`.

### Task 3: Наблюдаемость и атомарные артефакты

**Files:**
- Modify: `native/hu20-buckets/src/main.rs`
- Test: `native/hu20-buckets/src/main.rs`

**Interfaces:**
- Consumes: историю EMD-кластеризации из Task 2.
- Produces: start/end-события этапов, атомарный `status.json`, точные параметры и версию commit в `summary.json`.

- [ ] Написать падающие unit-тесты сериализации статуса и атомарной записи.
- [ ] Реализовать атомарную замену, этапы и лог диагностик; ошибки записи вернуть вызывающему коду.
- [ ] Запустить `cargo test`.

### Task 4: Безопасный локальный PowerShell-запуск

**Files:**
- Create: `native/hu20-buckets/scripts/run-potential-measurement.ps1`
- Create: `native/hu20-buckets/scripts/measure-process-memory.ps1`
- Modify: `native/hu20-buckets/README.md`

**Interfaces:**
- Consumes: release-бинарник и аргументы безопасного контрольного запуска.
- Produces: отдельный каталог с командой, логами, Private Bytes/Working Set и хешами таблиц.

- [ ] Добавить скрипт с параметрами по умолчанию `ranks=4`, `k=10`, `iterations=10`, `threads=2`, тайм-аутом и проверкой пустоты выходного каталога.
- [ ] Проверить синтаксис PowerShell без запуска генератора.

