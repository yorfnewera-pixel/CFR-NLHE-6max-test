# EMD No-Threshold Test Candidate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Проверить на test-only кандидате, доставляет ли EMD без per-edge порога всю требуемую массу и сохраняет ли корректность reduced cost.

**Architecture:** Рядом с fixture появится самостоятельный test-only successive-shortest-path кандидат с порогом capacity только `> 0.0`, лимитом итераций как диагностической защитой и типизированным исходом. Успех разрешён лишь когда доставленная масса совпадает с требуемой в явно выбранном `f64`-допуске; частичная стоимость никогда не является успешным результатом.

**Tech Stack:** Rust 2021, стандартная библиотека Rust, Cargo.

**Spec:** `articles/Potential-Aware EMD/EMD.md`, `articles/Potential-Aware EMD/План Sol.txt`, переданное пользователем ТЗ Sol от 2026-11-02.

## Global Constraints

- Production `try_emd` и CLI не менять.
- Не запускать генератор и `ranks=5`.
- Не объявлять ограничитель итераций доказательством завершения.
- Не возвращать частичную стоимость как успешный результат.

## Review Focus

- Остаток меньше старого per-edge порога, но больше старого aggregate-порога: требуется полный транспорт.
- Положительная augmentation, не меняющая `delivered` или residual capacity из-за `f64`: отдельная диагностическая ошибка.
- Sink недостижим: отдельный аварийный исход без стоимости успеха.
- Отрицательный reduced cost на реально релаксируемом ребре нового поиска.
- Ненормированные, разреженные, повторяющиеся и нулевые входы: сравнение с независимым оракулом либо явная ошибка входа.

---

### Task 1: Test-only кандидат и инварианты поиска

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` (модуль `tests`)

**Interfaces:**
- Produces: `test_only_emd_without_capacity_threshold(...) -> TestOnlyEmdOutcome` и снимки инвариантов каждого поиска.

- [ ] Написать RED-тесты для полного транспорта fixture и малого residual ниже `1e-9`.
- [ ] Реализовать test-only outcome, явные проверки полного транспорта, отсутствия пути и численного прогресса.
- [ ] Зафиксировать reduced cost всех реально релаксируемых рёбер до каждого поиска.
- [ ] Запустить целевые тесты и проверить GREEN.

### Task 2: Оракул и граничные входы

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` (модуль `tests`)

**Interfaces:**
- Consumes: test-only кандидат и `independent_transport_cost`.
- Produces: тесты совпадения стоимости/успеха для fixture, малых остатков, разреженных, ненормированных и нулевых ground cases.

- [ ] Написать RED-тесты для входов из Review Focus.
- [ ] Добавить минимальные test-only генераторы cases без изменения production validation.
- [ ] Запустить целевые тесты и проверить GREEN.

### Task 3: Документирование пределов вывода

**Files:**
- Modify: `web_reports/potential-emd-r5-k10-potential-investigation-20261102.md`
- Modify: `articles/Potential-Aware EMD/EMD.md`

- [ ] Зафиксировать фактические исходы, допуски и reduced-cost проверку нового кандидата.
- [ ] Явно отделить пройденные тесты от отсутствующего строгого доказательства завершения `f64`-реализации.
- [ ] Выполнить `cargo fmt --check`, `cargo test`, `git diff --check`.
