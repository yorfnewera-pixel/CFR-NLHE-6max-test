# hu20-buckets

Контрольный potential-aware замер выполняется только вручную:

```powershell
pwsh -File .\scripts\run-potential-measurement.ps1 -OutputDirectory .\results\control-before
```

Скрипт собирает release-бинарник, по умолчанию использует `RAYON_NUM_THREADS=2` и параметры `ranks=4`, `K=10`, `iterations=10`. Каталог результатов должен быть пустым. В нём сохраняются команда, stdout/stderr, `memory.csv` (Private Bytes и Working Set), JSON-артефакты и SHA-256 таблиц. При тайм-ауте процесс принудительно останавливается.

Полный прогон `ranks=13`, `K=200`, `iterations=100` этим скриптом не запускайте без отдельного решения.
