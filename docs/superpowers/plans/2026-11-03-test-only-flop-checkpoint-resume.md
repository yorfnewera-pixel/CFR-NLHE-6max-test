# Test-only flop checkpoint resume Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Продолжить единственный flop probe из локального checkpoint шага 18 до общего шага 20 без повторного вычисления flop-шагов 1–18.

**Architecture:** В `#[cfg(test)]` добавить отдельный ignored resume-test, который воспроизводит только обязательный river/turn-префикс для точных входов, проверяет сохранённый checkpoint и создаёт `TestOnlyEmdStepper` из snapshot. Production-путь, launcher и формат таблиц не затрагиваются. PowerShell supervisor уже использует Windows Job Object и остаётся единственным процессным контуром запуска.

**Tech Stack:** Rust, Cargo release tests, PowerShell/Windows Job Object.

**Spec:** Пользовательское ТЗ от 2026-11-03 в текущем чате; `articles/Potential-Aware EMD/EMD.md`; `web_reports/potential-emd-r6-k20-flop-trajectory-probe-20261103.md`.

## Global Constraints

- Менять только `#[cfg(test)]` код и документацию; production-код, launcher, метрика, центры и допуски неизменны.
- Проверять SHA-256 `1B80CD796BD240D37F380D861698431709BC7F54C517B84ABD564E48E9A15A0E`, шаг 18, `dim=20`, `K=20`, 21 150 точек и fingerprint `DF7E894991B2639B`; несовпадение останавливает запуск.
- Один release-probe: 2 Rayon-потока, timeout 300 с, новый пустой каталог, максимум общий шаг 20; повторов и повышения лимита нет.
- Предварительная release-сборка не входит в лимит; зависимый префикс и финальная немутирующая диагностика входят.

## Review Focus

- Несовместимый checkpoint не может привести к запуску с нуля — тест загрузки отклоняет его до stepper.
- Resume не пересчитывает flop-шаги 1–18 — новый test начинает с `from_snapshot` и пишет только `checkpoint-19/20`.
- Непрерывный и resumed малые прогоны совпадают побитово по assignments, центрам и диагностике.
- При timeout Job Object уничтожает всё дерево и supervisor фиксирует последний завершённый checkpoint.
- История состояний 1–18 не заявляется доступной, если checkpoint содержит лишь состояние 18.

---

### Task 1: Test-only resume-path

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs` (встроенные Rust tests)

**Interfaces:**
- Consumes: `TestOnlyEmdCheckpoint::load`, `TestOnlyEmdStepper::from_snapshot`, `test_only_write_checkpoint`.
- Produces: ignored test `test_only_ranks6_flop_k20_resume_from_checkpoint_to_twenty_iterations`.

- [ ] **Step 1: Написать failing test для resume из snapshot**

Проверить, что два шага после загруженного малого checkpoint побитово совпадают с соответствующими шагами непрерывного stepper.

- [ ] **Step 2: Запустить targeted test и зафиксировать ожидаемый RED**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml test_only_resume_ -- --nocapture`

Expected: FAIL, поскольку отдельного resume-пути ещё нет.

- [ ] **Step 3: Добавить минимальный test-only resume-test**

После проверки точных входов загрузить checkpoint, построить stepper через `from_snapshot`, выполнить только шаги 19–20, flush-ить метрики и атомарные checkpoints, затем выполнить `final_assignment_diagnostic`.

- [ ] **Step 4: Запустить targeted и полный Rust suite**

Run: `cargo test --manifest-path native/hu20-buckets/Cargo.toml`

Expected: PASS; ignored probe не запускается автоматически.

### Task 2: Один supervised release-probe и отчёт

**Files:**
- Modify: `web_reports/potential-emd-r6-k20-flop-trajectory-probe-20261103.md`
- Modify: `articles/Potential-Aware EMD/EMD.md`

**Interfaces:**
- Consumes: checkpoint шага 18 и результаты supervisor.
- Produces: воспроизводимая фиксация результата шагов 19–20 либо timeout/ранней остановки.

- [ ] **Step 1: Предварительно собрать release-бинарник**

Run: `cargo build --release --manifest-path native/hu20-buckets/Cargo.toml`.

- [ ] **Step 2: Выполнить ровно один probe через существующий Job Object supervisor**

Создать новый пустой каталог; в лимит 300 с включить проверку checkpoint и зависимый river/turn-префикс. Запустить ignored resume-test при `RAYON_NUM_THREADS=2`.

- [ ] **Step 3: Проверить артефакты supervisor**

Проверить `TimedOut`, exit code, отсутствие оставшихся процессов, последний checkpoint и trace.

- [ ] **Step 4: Обновить журналы фактическими данными**

Отдельно указать длительность восстановления входов, шаги 19–20, финальную диагностику либо её отсутствие, ограничение истории состояний и единственный предложенный следующий шаг без запуска.
