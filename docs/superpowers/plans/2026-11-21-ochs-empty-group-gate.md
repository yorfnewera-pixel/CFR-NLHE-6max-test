# OCHS Empty-Group Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Точно закрыть ворота пустых OCHS-групп на полной колоде без build таблиц и без equity-вычислений.

**Architecture:** Вместо перебора `C(52, 7)` удалённых множеств тест строит для каждой OCHS-группы сертификат из восьми попарно непересекающихся физических рук. Любое множество удалённых карт, опустошающее группу, обязано пересечь каждую руку сертификата и потому содержит не менее восьми карт; river удаляет только семь. Следовательно, пустые группы на полной колоде невозможны, а точная доля равна нулю.

**Tech Stack:** Rust 2021, Cargo unit tests.

**Spec:** `articles/Libratus_buckets/EMD.md`; `web_reports/libratus-ochs-pr-a-20261119.md`.

## Global Constraints

- Не изменять K50, CFR, Python, checkpoint, форматы таблиц или карту OCHS.
- Не строить таблицы и не вычислять showdown equity в диагностике.
- Малая колода проверяет только `uniform-equity-fill-v1`; частоту пустых групп определяет сертификат полной колоды.

## Review Focus

- Каждый сертификат содержит восемь различных физических рук именно указанной OCHS-группы.
- Ни одна карта не входит в две руки одного сертификата.
- Вывод «доля 0 %» использует только доказанный нижний предел восемь удалений, а не ограниченные board-текстуры.

---

### Task 1: Сертификат отсутствия пустых групп

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` в `mod tests`.

**Interfaces:**
- Consumes: `deck(13)`, `combinations`, `ochs_group`, `OCHS_GROUP_COUNT`.
- Produces: unit-тест `ochs_full_deck_groups_require_at_least_eight_removed_cards`.

- [ ] **Step 1: Написать падающий тест сертификата**

Для всех физических рук полной колоды сгруппировать пары по `ochs_group`; жадно выбрать восемь рук без общих карт в каждой группе и проверить точную принадлежность и непересечение.

- [ ] **Step 2: Запустить RED**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_full_deck_groups_require_at_least_eight_removed_cards`

Expected: FAIL, пока сертификат не реализован.

- [ ] **Step 3: Реализовать test-only построение сертификата**

Использовать только физические пары и 52-битовую маску занятых карт; запретить любое обращение к `evaluate` или `river_ochs_equities`.

- [ ] **Step 4: Запустить GREEN и полный crate suite**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_full_deck_groups_require_at_least_eight_removed_cards; cargo test --manifest-path native/hu20-buckets/Cargo.toml`

Expected: PASS; тест печатает сертификаты и доказывает точную нулевую частоту на полной колоде.

### Task 2: Скорректировать экспериментальные ворота и отчёт

**Files:**
- Modify: `articles/Libratus_buckets/EMD.md`.
- Modify: `web_reports/libratus-ochs-pr-a-20261119.md`.

- [ ] **Step 1: Записать математическое обоснование**

Заменить блокировку из-за `deck(5)` на: малая колода — fallback-only; восемь непересекающихся рук на группу — сертификат `min_removed >= 8`; river с семью удалениями даёт долю `0 %`.

- [ ] **Step 2: Зафиксировать фактическое решение ворот**

Разрешить подготовку PR B только для малых таблиц; полный build и обучение оставить запрещёнными без отдельного решения владельца.

- [ ] **Step 3: Проверить и зафиксировать**

Run: `cargo fmt --check --manifest-path native/hu20-buckets/Cargo.toml; cargo test --manifest-path native/hu20-buckets/Cargo.toml; git diff --check`

Expected: PASS.
