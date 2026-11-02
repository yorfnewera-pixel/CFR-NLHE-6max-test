# Ranks=5 EMD Test Investigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Подтвердить либо опровергнуть корректность потенциалов на сохранённой fixture ranks=5 без изменения production-решателя.

**Architecture:** В `#[cfg(test)]` рядом с существующей fixture расширить независимую от production-трассу: до каждого поиска она фиксирует eligible-рёбра и их reduced cost, а при нарушении — конкретное ребро, capacity, оба потенциала и достижимость. Стоимость fixture проверить отдельным эталонным min-cost-flow/LP-перебором, не копирующим shortest-path loop решателя; результат оформить в отчёте.

**Tech Stack:** Rust 2021, стандартная библиотека Rust, Cargo.

**Spec:** `articles/Potential-Aware EMD/EMD.md`, `articles/Potential-Aware EMD/План Sol.txt`, переданное пользователем ТЗ Sol от 2026-11-02.

## Global Constraints

- Только test-only расследование: production-код `try_emd` и поведение CLI не менять.
- Не повторять `ranks=5` и не запускать новые генераторные прогоны.
- Не принимать увеличение epsilon или частичную стоимость как исправление.
- Не использовать копию production shortest-path loop как независимый оракул.

## Review Focus

- Reduced cost меньше нуля на eligible-рёбре, достижимом из source до запуска Dijkstra.
- Отрицательное ребро только вне достижимой области: не выдавать его за нарушение инварианта поиска.
- Исходящие residual-рёбра sink: фиксировать, обрабатываются ли они до завершения поиска.
- Остаток массы в диапазоне aggregate/per-edge порогов: не спутать с исходной несвязностью.
- Эталонная стоимость fixture: независимый алгоритм должен завершаться при том же входе.

---

### Task 1: Расширить test-only трассировку инвариантов

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` (модуль `tests`)

**Interfaces:**
- Produces: `EmdFailureTrace` с покадровыми проверками перед каждым shortest-path поиском и идентификацией первого отрицательного eligible reduced-cost ребра.

- [ ] Написать тест, ожидающий запись конкретного отрицательного eligible ребра с `from`, `to`, capacity, potentials и признаком достижимости.
- [ ] Запустить тест и зафиксировать RED из-за отсутствующей диагностической записи.
- [ ] Добавить только test-only структуры и трассировку; отдельно фиксировать, посещён ли sink и были ли просмотрены его исходящие рёбра.
- [ ] Запустить целевые тесты и проверить GREEN.

### Task 2: Добавить независимый оракул для fixture

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` (модуль `tests`)

**Interfaces:**
- Produces: test-only `independent_transport_cost(left, right, ground, dimension) -> f64`, не использующий potentials или shortest-path loop production-решателя.

- [ ] Написать тест, требующий конечной эталонной стоимости fixture и совпадения с допустимой стоимостью транспорта.
- [ ] Запустить тест и зафиксировать RED из-за отсутствующего оракула.
- [ ] Реализовать малый exhaustive/basis min-cost transport oracle для dimension=10 в test-only коде.
- [ ] Запустить целевые тесты и проверить GREEN.

### Task 3: Зафиксировать выводы и регрессионный план

**Files:**
- Create: `web_reports/potential-emd-r5-k10-potential-investigation-20261102.md`
- Modify: `articles/Potential-Aware EMD/EMD.md`

**Interfaces:**
- Consumes: результаты test-only трассировки и независимого оракула.
- Produces: документированное решение о том, какие инварианты подтверждены, и минимальный план будущего production-исправления без его реализации.

- [ ] Зафиксировать ребро/достижимость, состояние sink и эталонную стоимость.
- [ ] Описать допустимую границу погрешности будущей обработки остатка и регрессионные тесты.
- [ ] Выполнить `cargo fmt --check`, `cargo test` и `git diff --check`.
