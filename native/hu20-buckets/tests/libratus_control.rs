use std::process::Command;

#[test]
#[ignore = "сборка двух наборов таблиц ranks=7"]
fn libratus_kmeans_cdf_mean_k50_matches_equity_builder() {
    let base = std::env::temp_dir().join(format!("hu20-libratus-control-{}", std::process::id()));
    let executable = env!("CARGO_BIN_EXE_hu20-buckets");
    for (mode, name) in [("equity", "control"), ("libratus-ochs", "libratus")] {
        let out = base.join(name);
        let status = Command::new(executable)
            .args([
                "build",
                "--abstraction",
                mode,
                "--ranks",
                "7",
                "--ks",
                "50",
                "--seed",
                "2026112101",
                "--iterations",
                "50",
                "--out",
            ])
            .arg(out)
            .status()
            .unwrap();
        assert!(status.success(), "ошибка сборщика {mode}");
    }
    let control = std::fs::read(base.join("control/turn-k50.bin")).unwrap();
    let experiment = std::fs::read(base.join("libratus/libratus-turn-k50.bin")).unwrap();
    assert_eq!(&control[24..], &experiment[28..]);
    std::fs::remove_dir_all(base).unwrap();
}
