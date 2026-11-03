Windows PowerShell, из корня репозитория:
- Rust crate test: `cargo test --manifest-path native\hu20-buckets\Cargo.toml`
- Rust formatting: `cargo fmt --manifest-path native\hu20-buckets\Cargo.toml --check`
- Python tests: `pytest` (см. requirements-dev.txt; использовать только релевантные файлы при локальной проверке).
- Проверка изменений: `git diff --check`, `git status --short`.

Контролируемый production launcher (только с явным согласованием): `pwsh -File .\native\hu20-buckets\scripts\run-potential-measurement.ps1 -OutputDirectory <новый-пустой-каталог>`. По README использует 2 Rayon threads и defaults ranks=4,K=10,iterations=10. Полный `ranks=13,K=200,iterations=100` не запускать автоматически.

Перед завершением: relevant Rust tests + `cargo fmt --check` + `git diff --check`; не заявлять прохождение без свежего вывода.