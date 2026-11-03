# Test-only Wasserstein-1D stepper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Добавить test-only stepper и checkpoint, которые побитово повторяют production 1D Wasserstein turn-кластеризацию на малых fixtures.

**Architecture:** Новый stepper находится только в `#[cfg(test)]` и повторяет production `kmeans_wasserstein_1d_with_diagnostics`, включая преобразование входов и вычислительный порядок. Его checkpoint маркирует backend и fingerprint исходных входов, чтобы исключить смешивание с transport-EMD состоянием.

**Tech Stack:** Rust 2021, Rayon, встроенный Rust test framework.

**Spec:** `docs/superpowers/specs/2026-11-03-test-only-wasserstein-stepper-design.md`

## Global Constraints

- Изменения только в test-only коде `native/hu20-buckets/src/lib.rs`.
- Production, launcher, метрика, stopping rule, допуски и таблицы не меняются.
- Не запускать дорогие replay/continuation, ranks=7, flop или generator.
- Новые свойства подтверждаются строгими побитовыми сравнениями, не расширением допусков.

## Review Focus

- Несортированные и совпадающие координаты: test сопоставляет production и stepper после перестановки.
- Равные расстояния: fixture проверяет выбор первого минимального cluster-id.
- Пустой кластер: fixture подтверждает сохранение его центра.
- Восстановление: continuous и round-trip траектории совпадают по RNG, центрам, assignments и diagnostics.
- Несовместимый state: backend-id и coordinates вызывают явный отказ загрузки checkpoint.

---

### Task 1: Production-faithful test-only stepper

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` — test module около `TestOnlyEmdStepper`
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `wasserstein_1d_sorted`, `Rng`, `Clustering`, `ClusteringIteration`
- Produces: `TestOnlyWasserstein1dStepper::{new, step, final_assignment_diagnostic, state_words}`

- [x] **Step 1: Write failing exact-trajectory test**

Создать fixture с несортированными coordinates и вызвать production
`kmeans_wasserstein_1d_with_diagnostics` и новый stepper. Проверить assignments,
биты центров, каждый objective/diagnostic и RNG state.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml test_only_wasserstein_stepper_matches_production_trajectory`

Expected: FAIL, потому что stepper отсутствует.

- [x] **Step 3: Implement the minimal test-only stepper**

Реализовать только production-эквивалентную preparation/iteration/final
diagnostic логику; сохранить первый минимум при равных расстояниях и порядок
суммирования production функции.

- [x] **Step 4: Run focused test to verify it passes**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml test_only_wasserstein_stepper_matches_production_trajectory`

Expected: PASS.

### Task 2: Backend-aware test-only checkpoint

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` — test-only checkpoint helpers
- Test: `native/hu20-buckets/src/lib.rs`

**Interfaces:**
- Consumes: `TestOnlyWasserstein1dStepper`
- Produces: backend-marked write/load and exact resumed state

- [x] **Step 1: Write failing round-trip and incompatibility tests**

Проверить: continuous и resumed trajectories совпадают побитово; transport-EMD
checkpoint отклоняется как другой backend; изменение одной coordinates
отклоняется как другой distance input.

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml test_only_wasserstein_checkpoint`

Expected: FAIL, потому что checkpoint не хранит backend/coordinates.

- [x] **Step 3: Implement minimal versioned checkpoint support**

Добавить отдельный Wasserstein checkpoint magic/backend-id и fingerprint,
включающий исходные coordinates. Не менять формат существующего transport-EMD
checkpoint; новый loader обязан отклонять иной backend до deserialization state.

- [x] **Step 4: Run focused tests to verify they pass**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml test_only_wasserstein_checkpoint`

Expected: PASS.

### Task 3: Regression verification

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs` — только если потребуется test cleanup

- [x] **Step 1: Run full Rust test suite**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml`

Expected: PASS; ignored manual probes не запускаются.

- [x] **Step 2: Run format and diff checks**

Run: `cargo fmt --manifest-path native/hu20-buckets/Cargo.toml --check; git diff --check`

Expected: exit code 0.

- [x] **Step 3: Inspect production boundary**

Verify: diff не меняет non-test `kmeans_wasserstein_1d_with_diagnostics`, `wasserstein_1d`, `main.rs` или `scripts/`.
