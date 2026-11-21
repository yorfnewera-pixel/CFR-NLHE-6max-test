# PR A: OCHS-признаки и проверки корректности — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Реализовать опубликованную карту OCHS-8 и проверяемый расчёт river-признака, не меняя K50 и CFR.

**Architecture:** В `hu20-buckets` появятся чистые функции: первая относит физическую стартовую руку к одной из восьми опубликованных групп, вторая для каждого hero holding на готовой доске считает восемь conditional equity. Реализация использует обработку блоков равного showdown-ранга и счётчики карт, а тестовый brute-force oracle остаётся независимым и медленным. Квалификационные результаты фиксируются в отчёте PR и сводятся в `EMD.md` только после завершения PR A.

**Tech Stack:** Rust 2021, Rayon, встроенный тестовый фреймворк Cargo.

**Spec:** `articles/Libratus_buckets/EMD.md`; `articles/Libratus_buckets/План Opus Libratus бакеты.txt`.

## Global Constraints

- Только `native/hu20-buckets`; не менять Python, trainer, CFR, checkpoint и форматы K50.
- Каждый запуск обязан выводить `running N tests`, где `N > 0`; число ignored
  тестов указывается в отчёте.
- Источник карты: Johanson et al., AAMAS 2013, Table 1; опубликованные номера `1..8` преобразуются во внутренние `0..7`.
- Контекст класса — `hole + current board`; порядок улиц не добавляется.
- Пустая группа заполняется scalar uniform equity: `uniform-equity-fill-v1`.
- Полные генерации таблиц и обучение запрещены; разрешены малая колода и отдельные заранее заданные river-доски.

## Review Focus

- Герой и оппонент делят карту: она не должна попадать в eligible-руки или в числитель equity.
- Руки одного showdown-ранга дают половину очка, включая ситуации, где доска играет для обоих.
- Пустая после card removal группа возвращает scalar uniform equity, а не `0`, `0.5` или `NaN`.
- Перестановка мастей сохраняет все восемь компонент и номер preflop-группы.
- Классы на границе диагонали таблицы (`pair`, suited, offsuit) не перепутаны.

---

### Task 1: Опубликованная карта восьми OCHS-групп

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Produces: `pub const OCHS_GROUP_COUNT: usize = 8` и `pub fn ochs_group(hand: [u8; 2]) -> u8`.
- Consumes: кодирование карты `rank * 4 + suit` и `RANKS` из того же модуля.

- [ ] **Step 1: Написать падающие тесты для карты Table 1**

Добавить тесты покрытия всех 169 классов и граничных ячеек: `22 → 3`, `AA → 7`, `65s → 2`, `65o → 0` во внутренней нумерации. Для каждой физической пары карт проверить инвариантность группы при глобальной перестановке мастей.

- [ ] **Step 2: Запустить тесты карты и подтвердить падение**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_group`

Expected: FAIL, поскольку `ochs_group` и `OCHS_GROUP_COUNT` отсутствуют.

- [ ] **Step 3: Реализовать `ochs_group(hand: [u8; 2]) -> u8`**

Добавить в `native/hu20-buckets/src/lib.rs` неизменяемую матрицу Table 1 из 13×13 `u8`. Для пары использовать диагональ; для suited — верхний треугольник (`low_rank`, `high_rank`), для offsuit — нижний. Проверить различие карт через `assert_ne!`; вернуть опубликованный номер минус один.

- [ ] **Step 4: Запустить целевые тесты**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_group`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add native/hu20-buckets/src/lib.rs
git commit -m "feat: add published OCHS hand groups"
```

### Task 2: River OCHS-8 и независимый oracle

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `OCHS_GROUP_COUNT`, `ochs_group`, `evaluate`, `combinations`.
- Produces: `pub fn river_ochs_equities(board: &[u8], deck: &[u8]) -> Vec<([u8; 2], [f32; OCHS_GROUP_COUNT])>`.

- [ ] **Step 1: Написать падающие тесты против локального brute-force oracle**

В test-модуле добавить oracle, который для каждого hero holding перебирает все допустимые opponent holdings, группирует их через `ochs_group` и напрямую считает `(win + tie / 2) / eligible`. Покрыть split pot, flush/straight, blockers и сравнение всех восьми компонент с точностью `1e-6`.

- [ ] **Step 2: Запустить oracle-тест и подтвердить падение**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml river_ochs_matches_brute_force`

Expected: FAIL, поскольку `river_ochs_equities` отсутствует.

- [ ] **Step 3: Реализовать `river_ochs_equities`**

В `native/hu20-buckets/src/lib.rs` обобщить паттерн `river_equities`: один раз вычислить ранг и OCHS-группу каждой совместимой физической руки, отсортировать по рангу, обработать блоки равных рангов. Для каждой группы вести число более слабых рук и счётчики карт; исключать hero-карты без двойного вычитания их пары. Не менять `river_equities` и не создавать `RiverTable` с OCHS-векторами.

- [ ] **Step 4: Запустить oracle и регрессию scalar equity**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml river_ochs_matches_brute_force; cargo test --manifest-path native/hu20-buckets/Cargo.toml river_equity_matches_brute_force`

Expected: PASS; существующий scalar-тест остаётся без изменений.

- [ ] **Step 5: Commit**

```bash
git add native/hu20-buckets/src/lib.rs
git commit -m "feat: calculate river OCHS equities"
```

### Task 3: Правило пустой группы, изоморфизм и ворота измерения

> Ворота пустых групп закрыты точным сертификатом полной колоды. Step 4
> проверяет только fallback `uniform-equity-fill-v1` на малой колоде.

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `river_equities`, `river_ochs_equities`, `canonical_boards`, `orbit_size`.
- Produces: тестовое доказательство `uniform-equity-fill-v1` и измерение веса классов с пустой группой на малой колоде.

- [ ] **Step 1: Написать падающие тесты пустой группы и suit-isomorphism**

Воспроизвести доску и hero holding, которые исключают все руки выбранной OCHS-группы. Проверить, что соответствующая OCHS-компонента битово равна scalar equity из `river_equities`; для suit-перестановки сравнить полный массив из восьми компонент. Добавить малоколодочный проход по каноническим river-классам с суммой `orbit_size` для классов, где найдена пустая группа.

- [ ] **Step 2: Запустить тесты и подтвердить падение**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_empty_group; cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_respects_suit_symmetry`

Expected: FAIL до добавления правила fill и диагностического пути.

- [ ] **Step 3: Добавить правило `uniform-equity-fill-v1` и диагностический путь**

При нулевом `eligible_g` подставлять уже рассчитанную scalar uniform equity hero. В тестовом коде подсчитать взвешенную долю без сохранения полной OCHS-таблицы; результат выразить как `empty_weight / total_weight`.

- [ ] **Step 4: Выполнить малоколодочную квалификацию**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_ -- --nocapture`

Expected: PASS; вывод содержит числитель, знаменатель и долю веса пустых групп по каждой из 8 групп.

- [ ] **Step 5: Commit**

```bash
git add native/hu20-buckets/src/lib.rs
git commit -m "test: qualify OCHS empty-group handling"
```

### Task 4: Регрессия и отчётность PR A

**Files:**
- Modify: `articles/Libratus_buckets/EMD.md`
- Create: `web_reports/libratus-ochs-pr-a-20261119.md`

**Interfaces:**
- Consumes: фактические результаты задач 1–3 и `git rev-parse HEAD`.
- Produces: воспроизводимый отчёт PR A и краткая запись в журнале эксперимента.

- [ ] **Step 1: Запустить полный набор unit-тестов crates**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml`

Expected: PASS, без запуска ignored resource-heavy тестов.

- [ ] **Step 2: Зафиксировать результаты в отчёте PR**

В `web_reports/libratus-ochs-pr-a-20261119.md` записать commit, dirty-state, команды, результаты, SHA-256 канонической сериализации карты, доли пустых групп, время, peak RSS, ограничения и решение по порогу `0,01 %`.

- [ ] **Step 3: Обновить `EMD.md` по вехе PR A**

Заменить статус «реализация не начата» на фактический статус PR A и добавить компактную сводку результата, порога пустых групп и решения «продолжать / исправить / остановиться». Не вносить туда покомандный журнал.

- [ ] **Step 4: Проверить форматирование и изменения**

Run: `cargo fmt --check --manifest-path native/hu20-buckets/Cargo.toml; git diff --check; git status --short`

Expected: PASS; список изменённых файлов совпадает с планом.

- [ ] **Step 5: Commit**

```bash
git add native/hu20-buckets/src/lib.rs articles/Libratus_buckets/EMD.md web_reports/libratus-ochs-pr-a-20261119.md
git commit -m "docs: record OCHS PR A qualification"
```
