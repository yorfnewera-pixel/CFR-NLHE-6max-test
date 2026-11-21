# Libratus PR B Small Tables Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Выпустить проверяемый генератор малых turn CDF и river OCHS-8 таблиц с отдельным форматом и детерминированными режимами mean/median.

**Architecture:** `StreetFeatures` остаётся источником turn CDF, а river OCHS-8 рассчитывается потоково из канонических river-досок без полной OCHS-таблицы в RAM. Новый `LibratusBucketTable`-формат и манифест изолируют эксперимент от `HU20BKT1`; mean использует текущий `kmeans`, median — отдельное обновление CDF-центров при том же L1/first-minimum назначении; river использует L2 и weighted sampling.

**Tech Stack:** Rust 2021, Rayon, Cargo tests, Python/NumPy reader, SHA-256.

**Spec:** `docs/superpowers/specs/2026-11-21-libratus-pr-b-design.md`; `articles/Libratus_buckets/EMD.md`.

## Global Constraints

- Только `ranks=7` и `K=50`/`K=200`; запрещены full build, `K=1_000/5_000` и обучение.
- Не менять K50, `HU20BKT1`, CFR, checkpoint или существующий Python-читатель K50.
- Mean — основной режим turn; median — отдельный экспериментальный режим манифеста. River всегда OCHS-8/L2.
- EMD — `sum(abs(CDF_i - CDF'_i))`; tie-break — `first_minimum_emd`.
- Файлы записываются атомарно; одинаковые входы дают идентичный SHA-256. Первый выпуск использует `seed=2026112101`, три рестарта, weighted sample до `20_000` с replacement и вероятностью по весу (без весов в k-means); при `N ≤ 20_000` — все точки с исходными весами. Максимум 50 итераций, порог смены массы `1e-4`, сохранение прежнего центра пустого кластера; побеждает минимум полной взвешенной ошибки, затем меньший индекс рестарта.

## Review Focus

- Старый читатель отклоняет новый magic и сохраняет побайтное поведение K50 — Task 3.
- Неполный, несортированный или содержащий ID вне `K` файл отвергается Rust и Python — Task 2.
- В median одинаковое правило чётного веса применяется к каждой координате, CDF остаётся монотонной и оканчивается `1` — Task 1.
- Равные L1-расстояния выбирают меньший ID центра в обоих режимах — Task 1.
- Идентичные seed, sample и restarts дают одинаковые файлы и SHA-256 — Task 4.

---

### Task 1: Закрытие ворот PR A

**Files:**
- Modify: `articles/Libratus_buckets/EMD.md`
- Modify: `web_reports/libratus-ochs-pr-a-20261119.md`
- Modify: `docs/superpowers/plans/2026-11-21-ochs-empty-group-gate.md`

**Interfaces:**
- Consumes: commit `95502399bff2c38ad81c7a3e1a1dfb41f8898cef` и вывод его полного прогона.
- Produces: закрытые ворота с точным сертификатом и отчёт воспроизводимости.

- [x] **Step 1: Прогнать сертификат и полный crate suite после `9550239`** (`9550239`)

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_full_deck_groups_require_at_least_eight_removed_cards -- --nocapture; cargo test --manifest-path native/hu20-buckets/Cargo.toml`
Expected: PASS; получены восемь напечатанных сертификатов и число прошедших тестов.

- [x] **Step 2: Зафиксировать сертификат в отчёте и журнале** (`49c2777`)

Записать SHA commit, результат прогона и восемь сертификатов; заменить
устаревшее измерение перед PR B утверждением `min_removed >= 8`; малая колода
остаётся fallback-only.

- [x] **Step 3: Отметить выполненные чекбоксы и проверить документацию** (`49c2777`)

Run: `git diff --check`
Expected: PASS.

- [x] **Step 4: Commit** (`3343314`)

Run: `git add articles/Libratus_buckets/EMD.md web_reports/libratus-ochs-pr-a-20261119.md docs/superpowers/plans/2026-11-21-ochs-empty-group-gate.md; git commit -m "docs: close OCHS empty-group gate"`
Expected: создан отдельный commit до PR B.

### Task 2: Режимы кластеризации CDF

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: `CdfCentroidMode::{Mean, LowerWeightedMedian}` и `kmeans_cdf(...) -> Clustering`.
- Consumes: CDF points, weights, `first_minimum_emd`.

- [ ] **Step 1: Добавить падающие unit-тесты median-центра и tie-break**

Проверить lower weighted median для чётного веса, неубывание центра и последний
элемент `1`; проверить меньший ID при одинаковой L1-дистанции. Имена тестов:
`cdf_median_preserves_cdf_invariants` и `cdf_tie_uses_first_minimum`.

- [ ] **Step 2: Запустить целевые тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml cdf_median`
Expected: FAIL с ненулевым числом запущенных тестов, так как режим и функция отсутствуют.

- [ ] **Step 3: Реализовать `kmeans_cdf`**

Mean делегирует существующему `kmeans`. Median сортирует значения каждой
координаты кластера по величине и выбирает первое со взвешенным префиксом не
меньше половины массы; пустой кластер не обновляется.

- [ ] **Step 4: Запустить unit-тесты кластеризации**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml cdf_ -- --nocapture`
Expected: PASS.

### Task 3: Изолированный формат и читатели

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Modify: `src/blueprint/equity_buckets.py`
- Modify: `tests/test_equity_buckets.py`

**Interfaces:**
- Produces: Rust `LibratusBucketTable` и Python `LibratusBucketTable`, оба с
  magic `HU20LIB1`, street, `K`, sorted hashes и `u16` bucket IDs.
- Consumes: `class_key`, `hash_key`, atomic writer.

- [ ] **Step 1: Написать падающие тесты round-trip и отказов формата**

Проверить Rust/Python чтение одной таблицы, а также неверный magic, размер,
сортировку, дубликат hash и ID `>= K`; отдельный fixture доказывает, что
старый `BucketTable` K50 возвращает те же bytes и bucket IDs.

- [ ] **Step 2: Запустить целевые тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml libratus_bucket_table; pytest tests/test_equity_buckets.py -k libratus`
Expected: FAIL до реализации; каждый вывод содержит хотя бы один запущенный тест.

- [ ] **Step 3: Реализовать новый читатель рядом с K50**

Не менять `BucketTable` и `load_cards`; shared binary layout получает отдельные
имена, magic и строгую валидацию в обоих языках.

- [ ] **Step 4: Проверить читатели**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml libratus_bucket_table; pytest tests/test_equity_buckets.py -k libratus`
Expected: PASS.

### Task 4: OCHS-8/L2, малый builder, manifest и воспроизводимость

**Files:**
- Modify: `native/hu20-buckets/src/main.rs`
- Modify: `native/hu20-buckets/README.md`

**Interfaces:**
- Produces: CLI `--abstraction libratus-ochs --cdf-centroid mean|lower-weighted-median`, отдельные `HU20LIB1` таблицы и JSON manifest.
- Consumes: `river_ochs_equities`, `kmeans_cdf`, новый L2 k-means, `LibratusBucketTable`, `atomic_write`.

- [ ] **Step 1: Добавить падающие binary-тесты разбора параметров и manifest**

Проверить L2-OCHS на oracle-векторах, воспроизводимую weighted-подвыборку и
потоковое назначение; проверить побайтную регрессию turn-mean `K=50` против
результата настоящего `--abstraction equity` на `ranks=7` с теми же seed,
инициализацией и параметрами выборки контроля. Новая схема sampling
проверяется отдельным тестом; если контроль не поддерживает вход, builder
останавливается с явной диагностикой.
Проверить отказ
неподдерживаемых `K`, полной колоды и неизвестного режима; проверить отдельное
имя режима, L1/L2-конвенции, seed/restarts/sample, empty-cluster rule,
критерий выбора рестарта и SHA-256 в manifest.

- [ ] **Step 2: Запустить binary-тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml --bin hu20-buckets libratus`
Expected: FAIL до реализации.

- [ ] **Step 3: Реализовать ограниченный builder**

Явно разрешить только `ranks=7`, `K in {50, 200}`, запретить flop и обучение.
River строит OCHS-8 по одной канонической доске, обучает L2-центры на трёх
фиксированных weighted samples и назначает все классы потоково. Turn mean
делегирует current `kmeans`; median использует lower weighted median. Для
каждой конфигурации записывать таблицы и manifest атомарно; повторный запуск
сравнивает SHA-256.

- [ ] **Step 4: Запустить детерминированный малый build для mean и median**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml --bin hu20-buckets libratus; cargo run --release --manifest-path native/hu20-buckets/Cargo.toml -- build --abstraction libratus-ochs --ranks 7 --ks 50,200 --seed 2026112101 --iterations 50`
Expected: PASS; два одинаковых прогона дают равные SHA-256.

### Task 5: Отчёт PR B

**Files:**
- Create: `web_reports/libratus-pr-b-small-tables-20261121.md`

**Interfaces:**
- Consumes: фактические тестовые результаты, manifests и SHA-256.
- Produces: воспроизводимый отчёт сравнения режимов и явная остановка перед PR C.


- [ ] **Step 1: Записать сравнение режимов**

Зафиксировать полную взвешенную L1 CDF-ошибку turn, полную взвешенную L2
OCHS-ошибку river, взвешенные размеры бакетов, число пустых кластеров,
итерации, время, SHA-256 и ограничение: на `ranks=7` часть OCHS-компонент
использует fallback, поэтому river-метрики не переносятся на полную колоду;
после отчёта не переходить к PR C.

- [ ] **Step 2: Полная проверка**

Run: `cargo fmt --check --manifest-path native/hu20-buckets/Cargo.toml; cargo test --manifest-path native/hu20-buckets/Cargo.toml; pytest tests/test_equity_buckets.py; git diff --check`
Expected: PASS.
