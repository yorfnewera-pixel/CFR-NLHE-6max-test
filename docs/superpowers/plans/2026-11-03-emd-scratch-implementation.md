# EmdScratch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Устранить повторные аллокации точного EMD без изменения его наблюдаемого результата или ошибок.

**Architecture:** Публичный `try_emd` останется совместимой обёрткой. Новое внутреннее ядро получает локальный `EmdScratch`; в k-means scratch создаётся отдельно для Rayon-задачи либо последовательной фазы. Дорефакторинговое ядро сохраняется только под `cfg(test)` как независимый oracle.

**Tech Stack:** Rust, Rayon 1.10, стандартная библиотека.

**Spec:** `web_reports/potential-emd-flop-hot-path-audit-20261103.md`; пользовательское ТЗ от 2026-11-03.

## Global Constraints

- Переиспользовать только allocation capacity; очищать логическое состояние перед каждым вызовом.
- Не менять метрику, порядок арифметики и обхода рёбер, tie-breaking, допуски, ошибки, центры и условия остановки.
- Scratch локален Rayon-задаче, без разделяемого изменяемого состояния и блокировок.
- Не запускать генератор, тяжёлые probe, новые K, seed или размеры.

## Review Focus

- Ошибка между успешными вызовами не загрязняет следующий расчёт — Task 1.
- Смена dimension и sparse/dense входы сбрасывают все буферы — Task 1.
- Tie и signed zero сохраняют тот же первый минимум — Task 2.
- Кластеризация сохраняет assignment, центроиды и диагностику побитово — Task 2.
- Benchmark не экстраполируется на генератор и имеет timeout — Task 3.

---

### Task 1: Независимый test-only oracle и scratch-ядро

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: внутренний `EmdScratch` и workspace-вариант `try_emd`.
- Produces: `#[cfg(test)]` дорефакторинговый oracle с комментарием исходного commit.

- [ ] Написать падающие tests для bitwise-сравнения oracle/scratch, ошибок и последовательного reuse.
- [ ] Запустить выбранные tests и зафиксировать ожидаемое падение из-за отсутствия scratch API.
- [ ] Реализовать минимальный scratch без изменения публичного `try_emd`.
- [ ] Запустить выбранные tests и убедиться в прохождении.

### Task 2: Локальный scratch в k-means и регрессия кластеризации

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `EmdScratch` и workspace-вариант из Task 1.
- Produces: k-means EMD с локальным scratch в assignment, objective-after-update и final diagnostic.

- [ ] Написать падающий test побитовой эквивалентности streaming clustering/reference oracle, включая tie и signed zero.
- [ ] Запустить его до замены вызовов и подтвердить падение из-за отсутствия test-only reference-path API.
- [ ] Подключить scratch без изменения порядка `0..k` и `first_minimum_emd`.
- [ ] Запустить целевой test и полный `cargo test`.

### Task 3: Ограниченный test-only benchmark и отчёт

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Create: `web_reports/potential-emd-scratch-20261103.md`
- Modify: `articles/Potential-Aware EMD/EMD.md`

- [ ] Добавить ignored release benchmark с `black_box`, прогревом, чередованием путей, timeout и фактическим числом вызовов.
- [ ] Собрать release до измерения и выполнить только benchmark.
- [ ] Зафиксировать время и ns/call без вывода о времени генератора.
- [ ] Выполнить `cargo fmt --check`, `cargo test` и `git diff --check`.
