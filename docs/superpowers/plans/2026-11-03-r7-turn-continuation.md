# R7 Turn Continuation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Однократно выполнить test-only продолжение turn с шага 28 до шага 50 без изменения production-пути.

**Architecture:** Расширить только test-модуль `lib.rs`: stepper получит
метод шага без установки test-only ранней стабилизации, а ignored probe
воспроизведёт проверяемый префикс и продолжит его с checkpoint после каждого
шага. Внешний PowerShell-supervisor задаст один дедлайн 300 с и сохранит trace.

**Tech Stack:** Rust, Cargo test, Rayon, PowerShell Core.

**Spec:** `docs/superpowers/specs/2026-11-03-r7-turn-continuation-design.md`

## Global Constraints

- `ranks=7`, `K=10`, seed `202610050003`, два Rayon-потока.
- Без flop, production-таблиц, production-кода и автоматических повторов.
- Проверять полные неокруглённые objectives и совместимость форматированной финальной диагностики перед continuation.
- Один общий timeout 300 с, включая river и turn-префикс.

## Review Focus

- Отсутствующий checkpoint-28 должен приводить к replay, а не к восстановлению из лога; тест: probe создаёт собственный полный state.
- Расхождение полного префикса не должно запускать шаг 29; тест: строгая проверка objectives и diagnostics до continuation.
- Финальная диагностика не должна менять stepper; тест: сравнение `state_words` до и после diagnostics.
- Порог ранней остановки не должен применяться после шага 28 только в test-only ветке; тест: шаг 29 возвращает диагностику при стабилизированном обычном stepper.
- Повтор полного состояния должен включать assignments, биты центров и RNG; тест: `state_words` используется как ключ истории.

---

### Task 1: Test-only API stepper и ignored probe

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs` (встроенный Rust test-модуль)

**Interfaces:**
- Produces: `TestOnlyEmdStepper::step_without_early_stop()` и ignored test
  `test_only_ranks7_turn_k10_continues_after_verified_step_28`.

- [ ] **Step 1: Написать падающий unit test для шага без ранней остановки**

Проверить, что шаг после состояния с `stabilized=true` выполняется только через
явный test-only метод и что production `step()` остаётся без изменений.

- [ ] **Step 2: Запустить test и подтвердить failure**

Run: `cargo test test_only_stepper_can_continue_past_early_stop`
Expected: FAIL, отсутствует test-only API.

- [ ] **Step 3: Реализовать минимальный test-only API и probe**

Вынести общий шаг в приватный helper с параметром установки стабилизации;
`step()` сохраняет текущую семантику, новый метод отключает только test-only
ранний выход. Probe проверяет точный checkpoint-28 при наличии, иначе
воспроизводит префикс и сверяет полные данные; после шага 28 пишет checkpoints,
trace и диагностики до терминального условия.

- [ ] **Step 4: Запустить сфокусированные тесты**

Run: `cargo test test_only_stepper_can_continue_past_early_stop`
Expected: PASS.

### Task 2: Один supervised release probe и отчёт

**Files:**
- Create: `native/hu20-buckets/scripts/run-r7-turn-continuation-probe.ps1`
- Modify: `web_reports/potential-emd-r7-k10-turn50-flop30-seed-202610050003-control-20261103.md`
- Modify: `articles/Potential-Aware EMD/EMD.md`

**Interfaces:**
- Consumes: ignored test из Task 1 и переменные путей trace/checkpoint.
- Produces: один trace, атомарные test-only checkpoint-файлы и документированный результат.

- [ ] **Step 1: Написать падающую parser-проверку параметров supervisor**

Проверить, что скрипт требует пустой выходной каталог, запускает только
названный ignored test и передаёт timeout 300 с без перезапуска.

- [ ] **Step 2: Реализовать supervisor и запустить parser-проверку**

Скрипт использует существующий Job Object supervisor, release-сборку и один
deadline, передаёт два Rayon-потока, проверяет отсутствие процессов после
завершения.

- [ ] **Step 3: Выполнить ровно один probe**

Run: новый supervisor с timeout 300 с.
Expected: один trace и checkpoints либо зафиксированная terminal-причина без повторов.

- [ ] **Step 4: Оформить отчёт и ссылку в EMD.md**

Отделить численное совпадение исходного replay от побитовых checkpoint-проверок
новой ветки; зафиксировать причины остановки и отсутствие flop.

- [ ] **Step 5: Запустить финальные проверки**

Run: `cargo fmt --check; cargo test; git diff --check`.
Expected: все обычные тесты проходят, diff форматно корректен.
