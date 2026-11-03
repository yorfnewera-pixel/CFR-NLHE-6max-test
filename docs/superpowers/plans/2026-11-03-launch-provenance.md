# План provenance launcher-измерений

> **Для агентных исполнителей:** обязательный поднавык — `superpowers:executing-plans`; шаги отмечаются флажками.

**Цель:** не допускать `code_revision: "unknown"` и ложного clean state в production summary.

**Архитектура:** Rust получает revision и dirty state только как единый проверенный Git metadata result, запуская Git из корня репозитория. Launcher сохраняет проверенный SHA и clean state до сборки и перед стартом дочернего процесса; любая ошибка Git прерывает измерение.

**Технологии:** Rust, PowerShell.

**Спецификация:** подтверждённое пользователем ТЗ текущего чата; production-контроль flop30.

## Ограничения

- Не подменять неизвестный revision текущим HEAD задним числом.
- Ошибка Git не считается чистым деревом.
- Не запускать генератор или повторный контроль.

### Задача 1: Проверенный Git metadata

**Файлы:**
- Изменить: `native/hu20-buckets/src/main.rs`
- Тест: встроенные тесты `main.rs`.

- [x] Написать падающие тесты на успешный metadata и ошибку любой Git-команды.
- [x] Реализовать тестируемый helper с явным `Result` и использовать его в summary.
- [x] Проверить `cargo test`.

### Задача 2: Launcher и отчёт

**Файлы:**
- Изменить: `native/hu20-buckets/scripts/run-potential-measurement.ps1`
- Изменить: `web_reports/potential-emd-r6-k20-turn50-flop30-control-20261103.md`

- Изменить: `articles/Potential-Aware EMD/EMD.md`

- [x] Сохранить SHA и state до build и перед дочерним запуском; расхождение или ошибка прерывают launcher.
- [x] Разделить последний objective flop на до/после update и final reassignment.
- [x] Проверить PowerShell parser и Rust tests без генератора.
