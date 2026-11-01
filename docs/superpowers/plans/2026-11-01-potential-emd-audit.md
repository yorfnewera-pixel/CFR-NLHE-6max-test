# Potential EMD Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Сделать малый Potential-Aware EMD-прогон проверяемым, воспроизводимым и однозначно диагностируемым без смены алгоритма кластеризации.

**Architecture:** Библиотека сохраняет текущую эвристику assignment-by-EMD/mean-update, но публикует три явно названные измерения objective и проверяет свойства результата. Launcher проверяет чистый Git, записывает полный SHA и признак dirty state, а успех подтверждает одновременно кодом процесса, статусом и ожидаемыми артефактами.

**Tech Stack:** Rust 2021, Rayon, PowerShell 7.

**Spec:** ТЗ Sol от 2026-11-01 в прикреплённом тексте; `articles/Potential-Aware EMD/План Sol.txt`.

## Global Constraints

- Не запускать K=20, ranks=5+, полный 52-карточный прогон, CFR или GPU.
- Не менять метрику EMD, seed, распределения, веса, ключи или правило обновления центров.
- Не подменять арифметический центр medoid/барицентром.
- Не делать commit или push без отдельного разрешения пользователя.

## Review Focus

- Objective после update и после reassignment измеряются одинаковыми весами и знаменателем.
- Конечные assignment соответствуют сохранённым центрам или несоответствие диагностируется.
- W1 не применяется к flop и совпадает с общим EMD в пределах численной точности на фиксированных случайных входах.
- Launcher не объявляет успех без финального статуса, таблиц, хешей и нулевого exit code.
- Контрольный запуск привязан к полному SHA и чистому рабочему дереву.

---

### Task 1: Аудит и явные objective-измерения

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

- [ ] Написать падающий тест для трёх objective-значений и примера роста цели после mean-update.
- [ ] Измерить objective до update, после update с прежними assignment и после reassignment с обновлёнными центрами.
- [ ] Добавить проверки нормировки, неотрицательности, конечности и согласованности финальных assignment/centroids.
- [ ] Запустить `cargo test --manifest-path native/hu20-buckets/Cargo.toml`.

### Task 2: Проверка W1 и признаков

**Files:**
- Modify: `native/hu20-buckets/src/lib.rs`
- Test: `native/hu20-buckets/src/lib.rs`

- [ ] Написать падающий тест фиксированных случайных нормированных распределений W1 против общего EMD.
- [ ] Подтвердить отсутствие `mean_equity` в potential-признаках и инварианты переходов/весов/ключей тестом.
- [ ] Запустить целевые и полные Rust-тесты.

### Task 3: Воспроизводимый launcher и версия кода

**Files:**
- Modify: `native/hu20-buckets/src/main.rs`
- Modify: `native/hu20-buckets/scripts/run-potential-measurement.ps1`
- Test: `native/hu20-buckets/src/main.rs`

- [ ] Написать падающие тесты сериализации полной Git-идентификации.
- [ ] Записать полный SHA и dirty flag; launcher отклоняет dirty tree и проверяет нулевой exit code, финальный статус, три таблицы и хеши.
- [ ] Проверить PowerShell AST и Rust-тесты; не запускать генератор.

### Task 4: Итоговый аудит и подготовка контрольного запуска

**Files:**
- Create: `web_reports/potential-emd-audit-20261101.md`

- [ ] Составить таблицу выполнено/частично/отсутствует с тестами и файлами.
- [ ] Описать рост objective как ограничение текущей эвристики без изменения алгоритма.
- [ ] Подготовить единственную команду чистого K=10 без выполнения.
