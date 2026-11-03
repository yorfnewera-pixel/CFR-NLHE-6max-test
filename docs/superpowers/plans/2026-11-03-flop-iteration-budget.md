# План отдельного flop-бюджета

> **Для агентных исполнителей:** обязательный поднавык — `superpowers:executing-plans`; шаги отмечаются флажками.

**Цель:** добавить opt-in бюджет flop, сохранив полную совместимость поведения без override.

**Архитектура:** `main.rs` разбирает общий, turn и flop бюджеты единым helper и передаёт flop-бюджет только в potential flop clustering. В summary записываются effective budget и признак явного override. PowerShell launcher передаёт новый аргумент только при значении больше нуля.

**Технологии:** Rust, Cargo tests, PowerShell.

**Спецификация:** подтверждённое пользователем ТЗ текущего чата.

## Глобальные ограничения

- Без `--flop-iterations` flop наследует `--iterations` без изменения river и turn.
- `--flop-iterations N` принимает только положительный целый `N`; launcher `-FlopIterations 0` означает отсутствие override.
- Не менять метрику EMD, обновление центров, порог ранней остановки, seed и final diagnostic.
- Не запускать генератор или тяжёлый probe.

## Фокус ревизии

- Override flop не влияет на river и turn budget.
- `summary.json` отражает effective budget и явность override.
- Нулевые, отсутствующие и нечисловые значения CLI отклоняются.
- Launcher не передаёт флаг при нуле и передаёт при положительном значении.

---

### Задача 1: CLI и summary

**Файлы:**
- Изменить: `native/hu20-buckets/src/main.rs`
- Тест: встроенные тесты в `main.rs`.

- [ ] Написать падающие тесты наследования, override и невалидных `--flop-iterations`.
- [ ] Реализовать минимальный разбор effective flop budget и явности override.
- [ ] Передать budget только в `kmeans_emd_profiled` и записать оба поля в summary.
- [ ] Запустить targeted и полный `cargo test`.

### Задача 2: Launcher и документация

**Файлы:**
- Изменить: `native/hu20-buckets/scripts/run-potential-measurement.ps1`
- Изменить: `articles/Potential-Aware EMD/EMD.md`

- [ ] Добавить `-FlopIterations` с range 0..100000 и условную передачу CLI-флага.
- [ ] Обновить usage и журнал без переписывания исторических результатов.
- [ ] Проверить синтаксис PowerShell, форматирование Rust и полный набор тестов.
