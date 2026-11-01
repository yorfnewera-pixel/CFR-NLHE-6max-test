# Potential-Aware K200 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Добавить детерминированное построение potential-aware K200 таблиц без изменения legacy equity-histogram пути.

**Architecture:** Rust builder сначала строит river assignment, затем превращает chance-runout'ы в распределения по следующим bucket ID. Для turn и flop k-means применяет EMD с рекурсивно построенной ground matrix. CLI маркирует артефакты отдельным форматом.

**Tech Stack:** Rust 2021, Rayon, существующий `hu20-buckets`.

**Spec:** `docs/superpowers/specs/2026-10-31-potential-aware-k200-design.md`

## Global Constraints

- Не изменять существующий `equity` режим и его формат файлов.
- Не запускать полный 52-карточный build или CFR training.
- Все новые API имеют unit-тесты, сначала наблюдаемые красными.

## Review Focus

- Масса распределения должна оставаться равной единице при каждом числе runout'ов.
- Ground matrix должна соответствовать расстояниям центроидов, а не индексам bucket ID.
- Новый режим не должен записывать артефакты под equity format tag.
- Повторный запуск с одинаковым seed обязан дать идентичные assignment.
- Small-deck build не должен зависеть от порядка потоков Rayon.

---

### Task 1: Точный EMD и k-means для произвольной ground matrix

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: `pub fn emd(...) -> f32`, `pub fn kmeans_emd(...) -> Clustering`.

- [ ] Написать тесты стоимости переноса для известных распределений и запустить их с ожидаемым FAIL.
- [ ] Реализовать минимальный exact transportation EMD и assignment/centroid update для `kmeans_emd`.
- [ ] Запустить unit-тесты crate.

### Task 2: Потенциальные признаки снизу вверх

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `RiverTable`, `Clustering`, `emd`.
- Produces: `PotentialFeatures::build(deck, board_size, river, next_assignment, next_k)`.

- [ ] Написать small-deck тесты нормировки и размерности turn/flop признаков, запустить с ожидаемым FAIL.
- [ ] Реализовать sparse chance-runout mapping и materialized row-major distributions.
- [ ] Запустить unit-тесты crate.

### Task 3: Изолированный CLI-режим и метаданные

**Files:**
- Modify: `native/hu20-buckets/src/main.rs`
- Modify: `docs/hu20-equity-buckets.md`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `--abstraction potential`.
- Produces: `hu20-potential-aware-emd-v1` summary и потенциальные таблицы.

- [ ] Написать unit-тест детерминизма small-deck pipeline и запустить с ожидаемым FAIL.
- [ ] Добавить potential branch без изменения default `equity` и документировать новый экспериментальный режим.
- [ ] Выполнить `cargo test --manifest-path native/hu20-buckets/Cargo.toml` и проверить CLI на small deck.
