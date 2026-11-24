# hu20-buckets

Контрольный potential-aware замер выполняется только вручную:

```powershell
pwsh -File .\scripts\run-potential-measurement.ps1 -OutputDirectory .\results\control-before
```

Скрипт собирает release-бинарник, по умолчанию использует `RAYON_NUM_THREADS=2` и параметры `ranks=4`, `K=10`, `iterations=10`. Каталог результатов должен быть пустым. В нём сохраняются команда, stdout/stderr, `memory.csv` (Private Bytes и Working Set), JSON-артефакты и SHA-256 таблиц. При тайм-ауте процесс принудительно останавливается.

Полный прогон `ranks=13`, `K=200`, `iterations=100` этим скриптом не запускайте без отдельного решения.
# Малые таблицы Libratus-inspired

Отдельный режим разрешает только `ranks=7`, 50 интервалов и `K=50,200`:

```powershell
cargo run --release --manifest-path native/hu20-buckets/Cargo.toml -- build --abstraction libratus-ochs --ranks 7 --ks 50,200 --seed 2026112101 --iterations 50 --out results/libratus-pr-b/tables
python -m scripts.verify_libratus_tables native/hu20-buckets/target/release/hu20-buckets.exe results/libratus-pr-b/tables
```

Для экспериментальной медианы добавьте `--cdf-centroid lower-weighted-median`.
Тёрн в режиме mean воспроизводит назначения исходного сборщика K50, включая
его правило завершения. Ривер инициализируется фиксированной взвешенной
выборкой не более 20 000 точек; все итерации и финальное назначение охватывают
все классы. Формат `HU20LIB1` и JSON-манифест проверяются отдельными
читателями; K50 остаётся в формате `HU20BKT1`. Флоп не имеет таблицы.
