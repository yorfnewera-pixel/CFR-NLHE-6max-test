//! hu20-buckets build --out DIR [--ranks 13] [--ks 50,200] [--bins 50] [--seed 1] [--iterations 100] [--diagnose-final-assignments]
//! hu20-buckets key HOLE BOARD     e.g. key "Ah Kh" "2h 7c 9d"
use hu20_buckets::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(temporary, path)
}

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.iter()
        .position(|a| a == name)
        .map(|i| args[i + 1].clone())
        .unwrap_or_else(|| default.to_string())
}

fn cards(text: &str) -> Vec<u8> {
    text.split_whitespace().map(parse_card).collect()
}

struct Log {
    started: Instant,
    status: PathBuf,
}
impl Log {
    fn stage(&self, stage: &str, detail: &str) {
        let elapsed = self.started.elapsed().as_secs_f64();
        eprintln!("[{elapsed:9.1}s] {stage} {detail}");
        let text = format!("{{\"stage\": \"{stage}\", \"detail\": \"{detail}\", \"elapsed_seconds\": {elapsed:.1}}}\n");
        atomic_write(&self.status, text.as_bytes()).expect("status");
    }
}

fn write_table(path: &Path, street: u32, k: u32, keys: &[u128], assignment: &[u16]) {
    BucketTable::new(street, k, keys, assignment)
        .write(path)
        .expect("table");
}

fn abort_emd(out: &Path, stage: &str, error: EmdContextError) -> ! {
    let path = out.join(format!("emd-failure-{stage}.txt"));
    let diagnostic = format!(
        "code_revision={}\nstage={stage}\n{error:#?}\n",
        code_revision(),
    );
    atomic_write(&path, diagnostic.as_bytes()).expect("write EMD failure diagnostic");
    panic!(
        "EMD failed during {stage}; diagnostic saved to {}",
        path.display()
    );
}

fn code_revision() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_owned())
        .filter(|revision| !revision.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn working_tree_is_dirty() -> bool {
    std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| !output.stdout.is_empty())
}

fn bucket_summary(k: usize, assignment: &[u16], weight: &[u64], equity: &[f32]) -> String {
    let mut mass = vec![0f64; k];
    let mut eq = vec![0f64; k];
    for ((&c, &w), &e) in assignment.iter().zip(weight).zip(equity) {
        mass[c as usize] += w as f64;
        eq[c as usize] += w as f64 * e as f64;
    }
    let total: f64 = mass.iter().sum();
    let parts: Vec<String> = (0..k)
        .map(|c| {
            format!(
                "[{:.8}, {:.6}]",
                mass[c] / total,
                if mass[c] > 0.0 { eq[c] / mass[c] } else { 0.0 }
            )
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

fn build(args: &[String]) {
    let out = PathBuf::from(arg(args, "--out", "buckets"));
    let ranks: u8 = arg(args, "--ranks", "13").parse().unwrap();
    let bins: usize = arg(args, "--bins", "50").parse().unwrap();
    let seed: u64 = arg(args, "--seed", "202610050003").parse().unwrap();
    let iterations: usize = arg(args, "--iterations", "100").parse().unwrap();
    let ks: Vec<usize> = arg(args, "--ks", "50,200")
        .split(',')
        .map(|k| k.parse().unwrap())
        .collect();
    let abstraction = arg(args, "--abstraction", "equity");
    let diagnose_final_assignments = args
        .iter()
        .any(|argument| argument == "--diagnose-final-assignments");
    assert!(
        matches!(abstraction.as_str(), "equity" | "potential"),
        "--abstraction must be equity or potential"
    );
    fs::create_dir_all(&out).unwrap();
    let log = Log {
        started: Instant::now(),
        status: out.join("status.json"),
    };
    let deck = deck(ranks);
    let mut summary = vec![format!(
        "\"format\": \"{}\", \"ranks\": {ranks}, \"deck_cards\": {}, \"bins\": {bins}, \"seed\": {seed}, \"ks\": {ks:?}, \"threads\": {}, \"code_revision\": \"{}\", \"working_tree_dirty\": {}",
        if abstraction == "potential" { "hu20-potential-aware-emd-v1" } else { "hu20-equity-buckets-v1" }, deck.len(), rayon::current_num_threads(), code_revision(), working_tree_is_dirty())];

    log.stage("river-equity", "computing");
    let river = RiverTable::build(&deck);
    log.stage("river-equity", &format!("{} classes", river.keys.len()));
    if abstraction == "potential" {
        let mut parts = Vec::new();
        for &k in &ks {
            log.stage("river-clustering", &format!("start k={k}"));
            let river_clusters = kmeans(&river.equity, 1, &river.weight, k, seed, iterations);
            log.stage("river-clustering", &format!("end k={k}"));
            log.stage("write-river-table", &format!("start k={k}"));
            write_table(
                &out.join(format!("potential-river-k{k}.bin")),
                3,
                k as u32,
                &river.keys,
                &river_clusters.assignment,
            );
            log.stage("write-river-table", &format!("end k={k}"));
            log.stage("river-ground-matrix", &format!("start k={k}"));
            let mut river_ground = vec![0.0; k * k];
            for left in 0..k {
                for right in 0..k {
                    river_ground[left * k + right] =
                        (river_clusters.centroids[left] - river_clusters.centroids[right]).abs();
                }
            }
            log.stage("river-ground-matrix", &format!("end k={k}"));
            log.stage("turn-features", &format!("start k={k}"));
            let turn =
                PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, k);
            log.stage(
                "turn-features",
                &format!("end k={k}, classes={}", turn.keys.len()),
            );
            log.stage("turn-clustering", &format!("start k={k}"));
            let turn_clusters = kmeans_wasserstein_1d_with_diagnostics(
                &turn.distribution,
                &turn.weight,
                &river_clusters.centroids,
                k,
                seed,
                iterations,
                diagnose_final_assignments,
            );
            for (iteration, diagnostic) in turn_clusters.diagnostics.iter().enumerate() {
                log.stage("turn-clustering", &format!("k={k}, iteration={}, objective_before={:.9}, objective_after={:.9}, moved_weight_fraction={:.9}, occupied_buckets={}", iteration + 1, diagnostic.objective_before_update, diagnostic.objective_after_update, diagnostic.moved_weight_fraction, diagnostic.occupied_buckets));
            }
            if let Some(diagnostic) = &turn_clusters.final_assignment_diagnostic {
                log.stage("turn-clustering", &format!("k={k}, final_assignment_diagnostic=true, changed_weight_fraction={:.9}, objective={:.9}", diagnostic.changed_weight_fraction, diagnostic.objective));
            }
            log.stage("turn-clustering", &format!("end k={k}"));
            log.stage("write-turn-table", &format!("start k={k}"));
            write_table(
                &out.join(format!("potential-turn-k{k}.bin")),
                2,
                k as u32,
                &turn.keys,
                &turn_clusters.assignment,
            );
            log.stage("write-turn-table", &format!("end k={k}"));
            log.stage("turn-ground-matrix", &format!("start k={k}"));
            let turn_ground = centroid_emd_matrix(&turn_clusters.centroids, k, &river_ground)
                .unwrap_or_else(|error| abort_emd(&out, "turn-ground-matrix", error));
            log.stage("turn-ground-matrix", &format!("end k={k}"));
            log.stage("flop-features", &format!("start k={k}"));
            let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_clusters.assignment, k);
            log.stage(
                "flop-features",
                &format!("end k={k}, classes={}", flop.keys.len()),
            );
            log.stage("flop-clustering", &format!("start k={k}"));
            let flop_started = Instant::now();
            let flop_progress: EmdProgressCallback = Arc::new(move |event| match event {
                EmdProgressEvent::IterationStart { iteration } => {
                    eprintln!("[{:9.1}s] flop-clustering k={k}, iteration={iteration}, phase=assignment, start", flop_started.elapsed().as_secs_f64());
                }
                EmdProgressEvent::AssignmentProgress {
                    iteration,
                    processed,
                    total,
                    emd_calls,
                } => {
                    eprintln!("[{:9.1}s] flop-clustering k={k}, iteration={iteration}, phase=assignment, processed={processed}/{total}, emd_calls={emd_calls}", flop_started.elapsed().as_secs_f64());
                }
                EmdProgressEvent::IterationEnd {
                    iteration,
                    objective_before_update,
                    objective_after_update,
                } => {
                    eprintln!("[{:9.1}s] flop-clustering k={k}, iteration={iteration}, phase=objective-after-update, objective_before={objective_before_update:.9}, objective_after={objective_after_update:.9}", flop_started.elapsed().as_secs_f64());
                }
            });
            let flop_clusters = kmeans_emd_with_diagnostics(
                &flop.distribution,
                k,
                &flop.weight,
                &turn_ground,
                k,
                seed,
                iterations,
                Some(flop_progress),
                diagnose_final_assignments,
            )
            .unwrap_or_else(|error| abort_emd(&out, "flop-clustering", error));
            for (iteration, diagnostic) in flop_clusters.diagnostics.iter().enumerate() {
                log.stage("flop-clustering", &format!("k={k}, iteration={}, objective_before={:.9}, objective_after={:.9}, moved_weight_fraction={:.9}, occupied_buckets={}", iteration + 1, diagnostic.objective_before_update, diagnostic.objective_after_update, diagnostic.moved_weight_fraction, diagnostic.occupied_buckets));
            }
            if let Some(diagnostic) = &flop_clusters.final_assignment_diagnostic {
                log.stage("flop-clustering", &format!("k={k}, final_assignment_diagnostic=true, changed_weight_fraction={:.9}, objective={:.9}", diagnostic.changed_weight_fraction, diagnostic.objective));
            }
            log.stage("flop-clustering", &format!("end k={k}"));
            log.stage("write-flop-table", &format!("start k={k}"));
            write_table(
                &out.join(format!("potential-flop-k{k}.bin")),
                1,
                k as u32,
                &flop.keys,
                &flop_clusters.assignment,
            );
            log.stage("write-flop-table", &format!("end k={k}"));
            parts.push(format!("{{\"k\": {k}, \"river_classes\": {}, \"turn_classes\": {}, \"flop_classes\": {}, \"river_objective\": {:?}, \"turn_objective\": {:?}, \"flop_objective\": {:?}}}", river.keys.len(), turn.keys.len(), flop.keys.len(), river_clusters.objective, turn_clusters.objective, flop_clusters.objective));
        }
        summary.push(format!("\"tables\": [{}]", parts.join(", ")));
        summary.push(format!(
            "\"elapsed_seconds\": {:.1}",
            log.started.elapsed().as_secs_f64()
        ));
        atomic_write(
            &out.join("summary.json"),
            format!("{{{}}}\n", summary.join(", ")).as_bytes(),
        )
        .expect("summary");
        log.stage("complete", "potential-aware tables written");
        return;
    }
    let mut streets = Vec::new();
    for (street, size) in [("turn", 4usize), ("flop", 3usize)] {
        log.stage(street, "features");
        let features = StreetFeatures::build(&deck, size, bins, &river);
        log.stage(street, &format!("{} classes", features.keys.len()));
        streets.push((street, features));
    }
    let mut parts = Vec::new();
    // River buckets cluster scalar equity.
    for &k in &ks {
        log.stage("river-kmeans", &format!("k={k}"));
        let c = kmeans(&river.equity, 1, &river.weight, k, seed, iterations);
        write_table(
            &out.join(format!("river-k{k}.bin")),
            3,
            k as u32,
            &river.keys,
            &c.assignment,
        );
        parts.push(format!(
            "{{\"street\": \"river\", \"k\": {k}, \"classes\": {}, \"weight\": {}, \"objective\": {:?}, \"buckets\": {}}}",
            river.keys.len(), river.weight.iter().sum::<u64>(), c.objective,
            bucket_summary(k, &c.assignment, &river.weight, &river.equity)));
    }
    for (street, f) in &streets {
        let code = if *street == "turn" { 2 } else { 1 };
        for &k in &ks {
            log.stage(&format!("{street}-kmeans"), &format!("k={k}"));
            let c = kmeans(&f.cdf, f.bins, &f.weight, k, seed, iterations);
            write_table(
                &out.join(format!("{street}-k{k}.bin")),
                code,
                k as u32,
                &f.keys,
                &c.assignment,
            );
            parts.push(format!(
                "{{\"street\": \"{street}\", \"k\": {k}, \"classes\": {}, \"weight\": {}, \"objective\": {:?}, \"buckets\": {}}}",
                f.keys.len(), f.weight.iter().sum::<u64>(), c.objective,
                bucket_summary(k, &c.assignment, &f.weight, &f.mean_equity)));
        }
    }
    summary.push(format!("\"tables\": [{}]", parts.join(", ")));
    summary.push(format!(
        "\"elapsed_seconds\": {:.1}",
        log.started.elapsed().as_secs_f64()
    ));
    atomic_write(
        &out.join("summary.json"),
        format!("{{{}}}\n", summary.join(", ")).as_bytes(),
    )
    .expect("summary");
    log.stage("complete", "all tables written");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("build") => build(&args),
        Some("key") => {
            let (hole, board) = (cards(&args[2]), cards(&args[3]));
            let key = class_key(&hole, &board);
            println!(
                "{key:032x} {:016x} {}",
                hash_key(key),
                orbit_size(&hole, &board)
            );
        }
        Some("eval") => {
            // One seven-card hand per stdin line; prints the comparable value.
            let stdin = std::io::stdin();
            let mut out = std::io::stdout().lock();
            for line in std::io::BufRead::lines(stdin.lock()) {
                writeln!(out, "{}", evaluate(&cards(&line.unwrap()))).unwrap();
            }
        }
        _ => {
            eprintln!("usage: hu20-buckets build --out DIR [--ranks 13] [--ks 50,200] [--bins 50] | key HOLE BOARD");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_status_without_partial_content() {
        let directory =
            std::env::temp_dir().join(format!("hu20-buckets-status-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("status.json");
        atomic_write(&path, b"{\"stage\":\"first\"}\n").unwrap();
        atomic_write(&path, b"{\"stage\":\"second\"}\n").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{\"stage\":\"second\"}\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn code_revision_is_full_git_sha() {
        let revision = code_revision();
        assert!(revision == "unknown" || revision.len() == 40, "{revision}");
    }
}
