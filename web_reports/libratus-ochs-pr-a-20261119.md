# PR A — OCHS-8: квалификационный отчёт

## Решение

**Продолжать к PR B только для малых таблиц.** Полный build, обучение и
переход к интеграции CFR не выполнялись и не разрешены этим отчётом.

## Объём PR

Изменён только `native/hu20-buckets/src/lib.rs`:

- добавлена опубликованная карта OCHS-8 и `ochs_group`;
- добавлен быстрый `river_ochs_equities` с обработкой блоков равных
  showdown-рангов и точным card removal;
- добавлен независимый brute-force oracle;
- реализовано `uniform-equity-fill-v1` для пустой группы;
- добавлены suit-invariance и квалификационные тесты.

K50, форматы `HU20BKT1`, Python, trainer, CFR, checkpoint и обучение не
изменялись.

## Входы и воспроизводимость

- Базовый commit: `dd58361de0ab8576457e649cb3e2d491aba7b811`.
- Commit реализации: `6d3c20b`, `6ad268f`, `b31c219`, `fa6d646`.
- Карта — Table 1 из Johanson et al., *Evaluating State-Space Abstractions
  in Extensive-Form Games*, AAMAS 2013. Номера `1..8` переводятся во
  внутренние `0..7`.
- Каноническая сериализация карты: 169 row-major байтов, ранги `2..A`,
  значения `0..7`.
- SHA-256 карты:
  `6483f50df6ebd7e51d37353be6040f1c1bfd800907f870991576ae31c8f0a335`.

## Проверки

Выполнены:

```text
cargo fmt --check --manifest-path native/hu20-buckets/Cargo.toml
cargo test --manifest-path native/hu20-buckets/Cargo.toml ochs_ -- --nocapture
cargo test --manifest-path native/hu20-buckets/Cargo.toml river_ochs_matches_brute_force
cargo test --manifest-path native/hu20-buckets/Cargo.toml river_equity_matches_brute_force
cargo test --manifest-path native/hu20-buckets/Cargo.toml
git diff --check
```

Финальный полный прогон: 77 unit-тестов crates и 12 unit-тестов binary
прошли; 13 явно resource-heavy тестов остались ignored. Oracle сравнил все
восемь компонент на парной, monotone-flush и straight досках. Карта проверена
для всех 169 классов и 24 глобальных перестановок мастей.

## Пустые группы

Малая `deck(5)` содержит только ранги `T..A`; потому группы 0, 1, 2, 3 и 5
физически недоступны. Результат: `any_empty_weight / total_weight =
1 627 920 / 1 627 920 = 100 %`. Это ожидаемая проверка правила fallback,
а не оценка частоты полной колоды.

Полная колода проверена на заранее закреплённых текстурах:

- `Ah Kd 7c 7s 2h` — paired;
- `Ah Kh Qh Jh Th` — monotone straight/flush;
- `9h Th Jc Qd Ks` — non-monotone straight;
- `Ah Ad As Ac 2h` — сильные блокеры.

Результат: для каждой из восьми групп `empty_weight = 0`; суммарно
`any_empty_weight / total_weight = 0 / 84 356 = 0 %`. Пилот проходит порог
`0,01 %`, но не заменяет полного измерения классов.

## Ресурсы и ограничения

Целевой полный тестовый прогон занял около 2,1 секунды; OCHS-квалификация —
около 1,7 секунды. Peak RSS не измерялся отдельным профилировщиком, поэтому
этот PR не содержит ресурсной квалификации. Полные таблицы, checkpoints и
обучение не запускались.

Следующий обязательный этап — PR B: детерминированная кластеризация и новый
формат только для малых таблиц. Перед масштабированием необходимы отдельные
замеры памяти, времени и диска.
