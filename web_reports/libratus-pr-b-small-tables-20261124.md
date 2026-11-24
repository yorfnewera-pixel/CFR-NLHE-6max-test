# PR B: малые таблицы Libratus-inspired — 24 ноября 2026

Исходная ревизия `cfcd49bb296643b4cb6be1c7163c754c6d365bc8` (`main`);
сборки выполнены с незакоммиченными изменениями PR B, что отражено полем
`working_tree_dirty=true`. Полная колода, `K=1000/5000`, обучение, CFR и
checkpoint не запускались. `ranks=7` не оценивает частоту пустых OCHS-групп
на полной колоде: в нём применяется `uniform-equity-fill-v1`.

Изменения: изолированный builder и проверка манифеста в Rust и Python,
взвешенный river L2, тесты, сценарии измерения и сверки; существующие K50
и `HU20BKT1` не менялись. Таблицы находятся в игнорируемом Git каталоге
`results/libratus-pr-b/release1/`, повтор — в соседнем `release2/`; при переносе
работы их необходимо сохранить отдельно. `HU20LIB1` хранит отсортированные
64-битные хеши классов и `u16` ID; манифест связывает их с SHA-256,
параметрами и эталонным отпечатком полного набора классов `ranks=7`.

## Сборка и ресурсы

Команды из корня репозитория, PowerShell:

```powershell
cargo build --release --offline --manifest-path native/hu20-buckets/Cargo.toml
./native/hu20-buckets/scripts/measure-libratus-build.ps1 -OutputDirectory results/libratus-pr-b/release1
./native/hu20-buckets/scripts/measure-libratus-build.ps1 -OutputDirectory results/libratus-pr-b/release2
python -m scripts.verify_libratus_tables native/hu20-buckets/target/release/hu20-buckets.exe results/libratus-pr-b/release1
python -m scripts.verify_libratus_tables native/hu20-buckets/target/release/hu20-buckets.exe results/libratus-pr-b/release2
./native/hu20-buckets/scripts/measure-libratus-build.ps1 -OutputDirectory results/libratus-pr-b/median -Ks 50 -Centroid lower-weighted-median
```

Параметры: `ranks=7`, `bins=50`, `seed=2026112101`, не более 50 итераций,
остановка при смене менее `1e-4` веса, первый минимум, пустой кластер
сохраняет прежний центр. Turn mean использует инициализацию и назначения
существующего K50; для медианы используется тот же начальный выбор.
River использует три рестарта с seed `seed + index`, одну фиксированную
подвыборку 20 000 с возвращением и вероятностью по весу при `N > 20 000`;
при меньшем `N` используются все точки с исходными весами. Каждая итерация и
финальное назначение river проходят по всем классам; выбран минимум полной
взвешенной суммы квадратов L2 с первым индексом при равенстве.

Два полных запуска `K=50,200` заняли **50,27 с / 46,37 с**, пиковый рабочий
набор процесса Windows — **335 675 392 / 336 142 336 байт** соответственно.
Median `K=50` занял 12,08 с, пиковый рабочий набор 336 408 576 байт.
Метрика памяти — `PeakWorkingSet64`, не частная память процесса.

| Таблица | Классов | Сумма весов | Размер, байт | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| turn K50 mean | 260862 | 5651100 | 2608648 | `b5ebf984d0332aaf3d75411884f98afa0d09517510b0c7c820ed9996627b1e15` |
| turn K200 mean | 260862 | 5651100 | 2608648 | `50f68aae03ac80dad99d1475c43f4faeef30da237027a089137ad12939ea3710` |
| river K50 | 1103011 | 24864840 | 11030138 | `6773a520315c6583bda477dac92b12fe8576736ac08e8054190d32eddbbfc436` |
| river K200 | 1103011 | 24864840 | 11030138 | `8e05868fe1fdc0cf0c056ef8b3d9b4fab1bb073f7a59e9dcf009dec6fefa1961` |
| turn K50 median | 260862 | 5651100 | 2608648 | `ea32131fa6ca79c1a8f395a0cf5ede30a8fcff5018033ede81bae9bdccdf52ed` |

Все четыре mean/river файла двух запусков побайтно равны. Rust читает,
валидирует и повторно сериализует каждый файл; Python восстанавливает те же
массивы ключей и ID и проверяет тот же SHA-256. Оба отвергают неправильный
magic, изменённые данные, несовпадающий манифест и пропущенный класс, даже
если исправлены счётчик и SHA-256. Имеющиеся тесты дополнительно проверяют
порядок, дубликаты, размер и диапазон ID. Проверка набора классов привязана
к опубликованным здесь отпечаткам `ranks=7`; остальные колоды проверяемым
читателем не принимаются.

## Проверки

```powershell
cargo test --offline --manifest-path native/hu20-buckets/Cargo.toml
python -m pytest tests/test_equity_buckets.py -q
cargo test --release --offline --manifest-path native/hu20-buckets/Cargo.toml --test libratus_control -- --ignored --nocapture
cargo test --release --offline --manifest-path native/hu20-buckets/Cargo.toml libratus_turn_mean_median_small_deck_objectives -- --ignored --nocapture
python -m pytest -q
```

Rust: 97 библиотечных и 13 binary тестов прошли, 15 библиотечных и один
интеграционный отложены; отложенный интеграционный тест выше запущен отдельно
и прошёл. Python `tests/test_equity_buckets.py`: 20 прошли, один пропущен.
Общий `pytest` завершился с кодом 1 на сборе: 239 ошибок, из них 173 из-за
отсутствия модуля `resource` и 66 из-за `fcntl` в Windows. Это не результат
тестирования остальных компонентов. В установленной WSL pytest отсутствует;
полное Python-окружение в рамках этого этапа не устанавливалось. Serena не обнаружила ошибок в изменённых
Python-модулях; в существующем тестовом helper есть предупреждение типов
`bytearray`/`bytes`. Context7 MCP в текущей среде недоступен.

Прямой независимый перебор showdown на `deck(5)` совпал с `StreetFeatures`.
Проверены монотонность CDF, lower weighted median, пустой кластер и первый
минимум; тест выше порога выборки подтверждает использование всех весов в
обновлении. При одинаковых начальных центрах на `ranks=7`, `K=50` средняя
взвешенная L1-цель после одного обновления: mean **2,263766888514**, median
**2,030750074880**; после полного прогона: **2,129305809338** за 50 и
**1,899106883006** за 19 обновлений. В этой диагностике цель считается после
назначения к финальным центрам. Для совместимости выпущенный mean использует
назначения последней итерации настоящего K50; финальный дополнительный
пересчёт менял 151 байт ID и поэтому не подменяет контроль.

Решение: результаты достаточны для ревью малых таблиц PR B. PR C начинать
после ревью и общего Python-suite в поддерживаемой среде; провал сбора на
Windows нельзя считать зелёным прогоном. Переход к полному build, большим `K`
или обучению не разрешён этим отчётом.
