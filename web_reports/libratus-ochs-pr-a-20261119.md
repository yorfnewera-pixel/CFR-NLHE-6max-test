# PR A — OCHS-8: квалификационный отчёт

## Решение

**Ворота пустых групп пройдены.** Точный сертификат полной колоды доказывает
нулевую долю пустых групп; малая колода проверяет лишь fallback. PR B не
запускался и требует отдельного решения владельца. Полный build, обучение и
интеграция CFR не выполнялись и не разрешены этим отчётом.

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
- Commit точного сертификата: `95502399bff2c38ad81c7a3e1a1dfb41f8898cef`.
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

Финальный полный прогон после commit `9550239`: 78 unit-тестов crates и 12
unit-тестов binary прошли; 13 явно resource-heavy тестов остались ignored.
Oracle сравнил все
восемь компонент на парной, monotone-flush и straight досках. Карта проверена
для всех 169 классов и 24 глобальных перестановок мастей.

## Пустые группы

Малая `deck(5)` содержит только ранги `T..A`; потому группы 0, 1, 2, 3 и 5
физически недоступны. Результат: `any_empty_weight / total_weight =
1 627 920 / 1 627 920 = 100 %`. Это ожидаемая проверка правила fallback,
а не оценка частоты полной колоды и не применение порога `0,01 %`.

Полная колода проверена на заранее закреплённых текстурах:

- `Ah Kd 7c 7s 2h` — paired;
- `Ah Kh Qh Jh Th` — monotone straight/flush;
- `9h Th Jc Qd Ks` — non-monotone straight;
- `Ah Ad As Ac 2h` — сильные блокеры.

Результат: для каждой из восьми групп `empty_weight = 0`; суммарно
`any_empty_weight / total_weight = 0 / 84 356 = 0 %`. Это ограниченный
пилот, подтверждённый следующим точным сертификатом.

### Точный сертификат полной колоды — 21 ноября 2026

Для каждой OCHS-группы unit-тест нашёл восемь попарно непересекающихся
физических рук. Любое удалённое множество, опустошающее группу, обязано
содержать минимум по одной карте из каждой руки и поэтому имеет размер не
меньше восьми. River удаляет ровно семь карт — две hero и пять board — так
что пустая группа невозможна для любого состояния полной колоды.

Следствие точное, а не выборочное: `any_empty_weight / total_weight = 0 %`
и доля каждой отдельной группы тоже `0 %`. Диагностика не использует
`evaluate`, не строит таблицы и не запускает обучение.

Воспроизводимый прогон `cargo test --manifest-path
native/hu20-buckets/Cargo.toml
ochs_full_deck_groups_require_at_least_eight_removed_cards -- --nocapture`
напечатал следующие сертификаты:

```text
group=0: 2c 3c, 2d 3d, 2h 3h, 2s 3s, 4c 5c, 4d 5d, 4h 5h, 4s 5s
group=1: 2c 8c, 2d 8d, 2h 8h, 2s 8s, 3c 9c, 3d 9d, 3h 9h, 3s 9s
group=2: 3c Tc, 3d Td, 3h Th, 3s Ts, 5c 6c, 5d 6d, 5h 6h, 5s 6s
group=3: 2c 2d, 2h 2s, 3c Jc, 3d Jd, 3h Jh, 3s Js, 4c Qc, 4d Qd
group=4: 6c Qc, 6d Qd, 6h Qh, 6s Qs, 7c Tc, 7d Td, 7h Th, 7s Ts
group=5: 2c Ac, 2d Ad, 2h Ah, 2s As, 3c 3d, 3h 3s, 4c 4d, 4h 4s
group=6: 6c 6d, 6h 6s, 7c 7d, 7h 7s, 8c Ac, 8d Ad, 8h Ah, 8s As
group=7: 8c 8d, 8h 8s, 9c 9d, 9h 9s, Tc Td, Th Ts, Jc Jd, Jh Js
```

## Ресурсы и ограничения

Целевой полный тестовый прогон занял около 2,1 секунды; OCHS-квалификация —
около 1,7 секунды. Peak RSS не измерялся отдельным профилировщиком, поэтому
этот PR не содержит ресурсной квалификации. Полные таблицы, checkpoints и
обучение не запускались.

Ворота `0,01 %` пройдены. Следующий возможный шаг — отдельное разрешение на
PR B с детерминированной кластеризацией и малыми таблицами; перед
масштабированием всё ещё необходимы отдельные замеры памяти, времени и
диска.
