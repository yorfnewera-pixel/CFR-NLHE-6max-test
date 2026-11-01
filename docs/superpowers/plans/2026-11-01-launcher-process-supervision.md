# Launcher Process Supervision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Добавить наблюдаемость текущего PowerShell launcher при недоступном exit code, не меняя генератор, порядок управления процессом и не превращая неизвестный исход в успех.

**Architecture:** Launcher сохраняет PID и окружение сразу после старта. После `WaitForExit`, `Receive-Job` и существующего `Refresh()` он снимает независимые защищённые показания `HasExited` и `ExitCode`, не позволяя ошибке одного показания затереть остальные. Smoke-скрипт использует тот же блок диагностики с короткими системными процессами.

**Tech Stack:** PowerShell 7.6.5; совместимость с Windows PowerShell 5.1 проверяется отдельно и не обходит Execution Policy.

**Spec:** Переписанный пользователем ответ Sol от 2026-11-01 в текущем чате.

## Global Constraints

- Не запускать генератор и K=20 в рамках этой задачи.
- Не менять алгоритм, метрику, seed, веса, центры и параметры генератора.
- Не подставлять `0`, не восстанавливать код из `status.json` и не использовать `$LASTEXITCODE` вместо кода `Start-Process`.
- Не обходить Execution Policy автоматически.
- Не добавлять `Handle`, задержки, retry или новую архитектуру supervision.

## Review Focus

- PID, command и версии среды должны быть записаны до запуска memory-job.
- Каждая из трёх точек чтения `ExitCode` должна либо записывать значение, либо полный исходный exception.
- Ошибка memory-job должна быть сохранена отдельно и не прерывать последующие диагностические показания.
- Недоступный финальный exit code должен остаться ошибкой launcher.

---

### Task 1: Диагностический helper и test-first smoke-проверки

**Files:**
- Create: `native/hu20-buckets/scripts/process-diagnostics.ps1`
- Create: `native/hu20-buckets/scripts/test-process-diagnostics.ps1`

**Interfaces:**
- Produces: `Write-ProcessDiagnostics`, принимающий объект `Process`, имя точки измерения и путь context; функция не прерывает сбор при ошибке свойства.

- [ ] **Step 1: Написать smoke-проверку для exit 0**

Проверка вызывает `Write-ProcessDiagnostics` для короткого `cmd.exe /c exit 0`, затем требует запись `ExitCode=0`.

- [ ] **Step 2: Запустить smoke-проверку и подтвердить ожидаемое падение из-за отсутствующего helper**

Run: `pwsh -NoProfile -File native/hu20-buckets/scripts/test-process-supervision.ps1`
Expected: FAIL, потому что `Write-ProcessDiagnostics` ещё не определён.

- [ ] **Step 3: Реализовать минимальный `Write-ProcessDiagnostics`**

Записать защищённые чтения `HasExited` и `ExitCode`; при исключении записать `Exception.ToString()` и `ScriptStackTrace`, но не бросать исключение.

- [ ] **Step 4: Расширить smoke-проверку exit 7 и сбоя свойства процесса**

Для exit 7 проверить запись `ExitCode=7`. Для недоступного свойства проверить запись исключения без остановки дальнейшей проверки.

- [ ] **Step 5: Запустить smoke-проверки в PowerShell 7.6.5**

Run: `pwsh -NoProfile -File native/hu20-buckets/scripts/test-process-supervision.ps1`
Expected: PASS для всех четырёх сценариев; без запуска генератора.

### Task 2: Подключение launcher к диагностическому helper

**Files:**
- Modify: `native/hu20-buckets/scripts/run-potential-measurement.ps1:50-82`
- Modify: `native/hu20-buckets/scripts/test-process-diagnostics.ps1`

**Interfaces:**
- Consumes: `Write-ProcessDiagnostics` из `process-diagnostics.ps1`.
- Produces: существующие артефакты launcher с PID, средой, тремя точками диагностического чтения и раздельной диагностикой memory-job.

- [ ] **Step 1: Добавить failing smoke-утверждение о том, что launcher использует helper**

Проверка статически требует dot-source `process-diagnostics.ps1` и три вызова `Write-ProcessDiagnostics` в launcher.

- [ ] **Step 2: Запустить проверку и подтвердить ожидаемое падение**

Run: `pwsh -NoProfile -File native/hu20-buckets/scripts/test-process-supervision.ps1`
Expected: FAIL, поскольку launcher ещё не снимает три точки диагностики.

- [ ] **Step 3: Добавить диагностические вызовы в существующий блок supervision launcher**

Сохранить существующий порядок `WaitForExit → Receive-Job → Refresh → ExitCode`, параметры, release-сборку, проверку таблиц, summary/status и cleanup; не менять аргументы генератора.

- [ ] **Step 4: Запустить полный smoke и синтаксические проверки**

Run: `pwsh -NoProfile -File native/hu20-buckets/scripts/test-process-supervision.ps1`; PowerShell parser для обоих script; `cargo fmt --check`; `cargo test`; `git diff --check`.
Expected: smoke PASS, Rust-набор зелёный, parser без ошибок.
