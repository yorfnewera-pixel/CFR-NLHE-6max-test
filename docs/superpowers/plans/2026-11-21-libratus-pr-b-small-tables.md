# Libratus PR B Small Tables Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Выпустить проверяемый генератор малых turn/river-таблиц с отдельным форматом и детерминированными режимами mean/median.

**Architecture:** Существующие `RiverTable` и `StreetFeatures` остаются источниками признаков. Новый `PotentialBucketTable`-формат и манифест изолируют эксперимент от `HU20BKT1`; mean использует текущий `kmeans`, median — отдельное обновление CDF-центров при том же L1/first-minimum назначении.

**Tech Stack:** Rust 2021, Rayon, Cargo tests, Python/NumPy reader, SHA-256.

**Spec:** `docs/superpowers/specs/2026-11-21-libratus-pr-b-design.md`; `articles/Libratus_buckets/EMD.md`.

## Global Constraints

- Только малая колода и `K=50`/`K=200`; запрещены full build, `K=1_000/5_000` и обучение.
- Не менять K50, `HU20BKT1`, CFR, checkpoint или существующий Python-читатель K50.
- Mean — основной режим; median — отдельный экспериментальный режим манифеста.
- EMD — `sum(abs(CDF_i - CDF'_i))`; tie-break — `first_minimum_emd`.
- Файлы записываются атомарно; одинаковые входы дают идентичный SHA-256.

## Review Focus

- Старый читатель отклоняет новый magic и сохраняет прежнее поведение — Task 2.
- Неполный, несортированный или содержащий ID вне `K` файл отвергается Rust и Python — Task 2.
- В median одинаковое правило чётного веса применяется к каждой координате, CDF остаётся монотонной и оканчивается `1` — Task 1.
- Равные L1-расстояния выбирают меньший ID центра в обоих режимах — Task 1.
- Идентичные seed, sample и restarts дают одинаковые файлы и SHA-256 — Task 3.

---

### Task 1: Режимы кластеризации CDF

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: `CdfCentroidMode::{Mean, LowerWeightedMedian}` и `kmeans_cdf(...) -> Clustering`.
- Consumes: CDF points, weights, `first_minimum_emd`.

- [ ] **Step 1: Добавить падающие unit-тесты median-центра и tie-break**

Проверить lower weighted median для чётного веса, неубывание центра и последний
элемент `1`; проверить меньший ID при одинаковой L1-дистанции.

- [ ] **Step 2: Запустить целевые тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml cdf_median -- --exact`
Expected: FAIL, так как режим и функция отсутствуют.

- [ ] **Step 3: Реализовать `kmeans_cdf`**

Mean делегирует существующему `kmeans`. Median сортирует значения каждой
координаты кластера по величине и выбирает первое со взвешенным префиксом не
меньше половины массы; пустой кластер не обновляется.

- [ ] **Step 4: Запустить unit-тесты кластеризации**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml cdf_ -- --nocapture`
Expected: PASS.

### Task 2: Изолированный формат и читатели

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Modify: `src/blueprint/equity_buckets.py`
- Modify: `tests/test_equity_buckets.py`

**Interfaces:**
- Produces: Rust `PotentialBucketTable` и Python `PotentialBucketTable`, оба с
  magic `HU20POT1`, street, `K`, sorted hashes и `u16` bucket IDs.
- Consumes: `class_key`, `hash_key`, atomic writer.

- [ ] **Step 1: Написать падающие тесты round-trip и отказов формата**

Проверить Rust/Python чтение одной таблицы, а также неверный magic, размер,
сортировку, дубликат hash и ID `>= K`.

- [ ] **Step 2: Запустить целевые тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml potential_bucket_table -- --exact; pytest tests/test_equity_buckets.py -k potential`
Expected: FAIL до реализации.

- [ ] **Step 3: Реализовать новый читатель рядом с K50**

Не менять `BucketTable` и `load_cards`; shared binary layout получает отдельные
имена, magic и строгую валидацию в обоих языках.

- [ ] **Step 4: Проверить читатели**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml potential_bucket_table; pytest tests/test_equity_buckets.py -k potential`
Expected: PASS.

### Task 3: Малый builder, manifest и воспроизводимость

**Files:**
- Modify: `native/hu20-buckets/src/main.rs`
- Modify: `native/hu20-buckets/README.md`

**Interfaces:**
- Produces: CLI `--abstraction libratus-potential --cdf-centroid mean|lower-weighted-median`, отдельные `HU20POT1` таблицы и JSON manifest.
- Consumes: `kmeans_cdf`, `PotentialBucketTable`, `atomic_write`.

- [ ] **Step 1: Добавить падающие binary-тесты разбора параметров и manifest**

Проверить отказ неподдерживаемых `K`, полной колоды и неизвестного режима;
проверить отдельное имя режима, EMD-конвенцию, seed/restarts/sample и SHA-256
в manifest.

- [ ] **Step 2: Запустить binary-тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml --bin hu20-buckets libratus`
Expected: FAIL до реализации.

- [ ] **Step 3: Реализовать ограниченный builder**

Явно разрешить только `ranks < 13`, `K in {50, 200}`, запретить flop и
обучение. Для одной конфигурации записывать таблицы и manifest атомарно;
повторный запуск сравнивает SHA-256.

- [ ] **Step 4: Запустить детерминированный малый build для mean и median**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml --bin hu20-buckets libratus; cargo run --release --manifest-path native/hu20-buckets/Cargo.toml -- build ...`
Expected: PASS; два одинаковых прогона дают равные SHA-256.

### Task 4: Закрытие PR A и отчёт PR B

**Files:**
- Modify: `articles/Libratus_buckets/EMD.md`
- Modify: `web_reports/libratus-ochs-pr-a-20261119.md`
- Create: `web_reports/libratus-pr-b-small-tables-20261121.md`
- Modify: `docs/superpowers/plans/2026-11-21-ochs-empty-group-gate.md`

**Interfaces:**
- Consumes: фактические тестовые результаты, сертификаты, manifests и SHA-256.
- Produces: воспроизводимые отчёты и закрытые чекбоксы реализации сертификата.

- [ ] **Step 1: Зафиксировать запуск после `9550239`**

Добавить в PR A SHA `95502399bff2c38ad81c7a3e1a1dfb41f8898cef`, число тестов и
восемь сертификатов; отметить выполненные шаги плана ворот.

- [ ] **Step 2: Исправить ворота в EMD.md**

Заменить обязательное измерение частоты на сертификат `min_removed >= 8`;
малая колода остаётся только fallback-проверкой.

- [ ] **Step 3: Записать сравнение режимов**

Зафиксировать внутрикластерную ошибку, размеры бакетов, итерации, время,
SHA-256 и ограничение: после отчёта не переходить к PR C.

- [ ] **Step 4: Полная проверка**

Run: `cargo fmt --check --manifest-path native/hu20-buckets/Cargo.toml; cargo test --manifest-path native/hu20-buckets/Cargo.toml; pytest tests/test_equity_buckets.py; git diff --check`
Expected: PASS.
