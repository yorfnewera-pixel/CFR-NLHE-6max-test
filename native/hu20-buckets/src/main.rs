//! hu20-buckets build --out DIR [--ranks 13] [--ks 50,200] [--bins 50] [--seed 1] [--iterations 100] [--turn-iterations N] [--flop-iterations N] [--diagnose-final-assignments] [--profile-flop-emd]
//! hu20-buckets key HOLE BOARD     e.g. key "Ah Kh" "2h 7c 9d"
use hu20_buckets::*;
use rayon::prelude::*;
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

fn positive_iteration_override(args: &[String], name: &str) -> Option<usize> {
    args.iter()
        .position(|argument| argument == name)
        .map(|index| {
            let value = args
                .get(index + 1)
                .unwrap_or_else(|| panic!("{name} requires a value"));
            let iterations = value
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{name} must be a positive integer"));
            assert!(iterations > 0, "{name} must be a positive integer");
            iterations
        })
}

fn iteration_limits(args: &[String]) -> (usize, usize, usize, bool, bool) {
    let iterations: usize = arg(args, "--iterations", "100").parse().unwrap();
    let turn_iterations = positive_iteration_override(args, "--turn-iterations");
    let flop_iterations = positive_iteration_override(args, "--flop-iterations");
    (
        iterations,
        turn_iterations.unwrap_or(iterations),
        flop_iterations.unwrap_or(iterations),
        turn_iterations.is_some(),
        flop_iterations.is_some(),
    )
}

fn validate_flop_iterations_abstraction(abstraction: &str, flop_iterations_explicit: bool) {
    assert!(
        abstraction == "potential" || !flop_iterations_explicit,
        "--flop-iterations requires --abstraction potential"
    );
}

fn validate_libratus_parameters(ranks: u8, ks: &[usize], bins: usize) -> Result<(), &'static str> {
    if ranks != 7 || bins != 50 || ks.is_empty() || ks.iter().any(|&k| k != 50 && k != 200) {
        return Err("libratus-ochs допускает только ranks=7, bins=50 и K=50/200");
    }
    Ok(())
}

fn build_libratus(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(arg(args, "--out", "libratus-buckets"));
    let ranks: u8 = arg(args, "--ranks", "7").parse()?;
    let bins: usize = arg(args, "--bins", "50").parse()?;
    let ks: Vec<usize> = arg(args, "--ks", "50,200")
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let seed: u64 = arg(args, "--seed", "2026112101").parse()?;
    let iterations: usize = arg(args, "--iterations", "50").parse()?;
    let centroid = arg(args, "--cdf-centroid", "mean");
    let mode = match centroid.as_str() {
        "mean" => CdfCentroidMode::Mean,
        "lower-weighted-median" => CdfCentroidMode::LowerWeightedMedian,
        _ => return Err("неизвестный CDF centroid".into()),
    };
    validate_libratus_parameters(ranks, &ks, bins)?;
    if iterations == 0 || iterations > 50 {
        return Err("число итераций должно быть от 1 до 50".into());
    }
    fs::create_dir_all(&out)?;
    let (revision, dirty) = git_metadata()?;
    let deck = deck(ranks);
    let started = Instant::now();
    eprintln!("river scalar equity");
    let river = RiverTable::build(&deck);
    eprintln!(
        "turn features, elapsed={:.1}s",
        started.elapsed().as_secs_f64()
    );
    let turn = StreetFeatures::build(&deck, 4, bins, &river);
    eprintln!(
        "river OCHS features, elapsed={:.1}s",
        started.elapsed().as_secs_f64()
    );
    let boards = canonical_boards(&deck, 5);
    let mut rows: Vec<(u128, u64, [f32; OCHS_GROUP_COUNT])> = boards
        .par_iter()
        .flat_map_iter(|board| {
            river_ochs_equities(board, &deck)
                .into_iter()
                .map(move |(hole, feature)| {
                    (class_key(&hole, board), orbit_size(&hole, board), feature)
                })
        })
        .collect();
    rows.par_sort_unstable_by_key(|row| row.0);
    rows.dedup_by(|a, b| {
        if a.0 == b.0 {
            assert_eq!(a.1, b.1);
            assert_eq!(a.2, b.2);
            true
        } else {
            false
        }
    });
    let river_keys: Vec<u128> = rows.iter().map(|r| r.0).collect();
    let river_weights: Vec<u64> = rows.iter().map(|r| r.1).collect();
    let river_points: Vec<f32> = rows.iter().flat_map(|r| r.2).collect();
    assert_eq!(river.keys, river_keys, "неполный набор river классов");
    for &k in &ks {
        eprintln!(
            "turn k={k}, elapsed={:.1}s",
            started.elapsed().as_secs_f64()
        );
        let turn_result = if mode == CdfCentroidMode::Mean {
            kmeans(&turn.cdf, bins, &turn.weight, k, seed, iterations)
        } else {
            kmeans_cdf(&turn.cdf, bins, &turn.weight, k, seed, iterations, mode)
        };
        emit_libratus_table(
            &out,
            2,
            k,
            &turn.keys,
            &turn.weight,
            &turn_result.assignment,
            ranks,
            &revision,
            dirty,
            seed,
            iterations,
            &centroid,
        )?;
        eprintln!(
            "river k={k}, elapsed={:.1}s",
            started.elapsed().as_secs_f64()
        );
        let mut best: Option<Clustering> = None;
        for restart in 0..3 {
            let candidate = libratus_l2(
                &river_points,
                OCHS_GROUP_COUNT,
                &river_weights,
                k,
                seed,
                seed.wrapping_add(restart),
                iterations,
            );
            eprintln!(
                "river k={k} restart={restart} objective={}",
                candidate.objective.last().unwrap()
            );
            if best.as_ref().map_or(true, |b| {
                candidate.objective.last().unwrap() < b.objective.last().unwrap()
            }) {
                best = Some(candidate);
            }
        }
        emit_libratus_table(
            &out,
            3,
            k,
            &river_keys,
            &river_weights,
            &best.unwrap().assignment,
            ranks,
            &revision,
            dirty,
            seed,
            iterations,
            "mean",
        )?;
    }
    eprintln!("complete elapsed={:.1}s", started.elapsed().as_secs_f64());
    Ok(())
}

fn emit_libratus_table(
    out: &Path,
    street: u32,
    k: usize,
    keys: &[u128],
    weights: &[u64],
    assignment: &[u16],
    ranks: u8,
    revision: &str,
    dirty: bool,
    seed: u64,
    iterations: usize,
    centroid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(keys.len(), assignment.len());
    let mut rows: Vec<(u64, u16)> = keys
        .iter()
        .map(|&key| hash_key(key))
        .zip(assignment.iter().copied())
        .collect();
    rows.sort_unstable_by_key(|r| r.0);
    let table = LibratusBucketTable::new(
        street,
        k as u32,
        rows.iter().map(|r| r.0).collect(),
        rows.iter().map(|r| r.1).collect(),
    )?;
    let bytes = table.serialize();
    let name = if street == 2 { "turn" } else { "river" };
    let table_path = out.join(format!("libratus-{name}-k{k}.bin"));
    let manifest_path = out.join(format!("libratus-{name}-k{k}.json"));
    let mut manifest = LibratusManifest::for_table(
        &bytes,
        street,
        k as u32,
        keys.len() as u64,
        weights.iter().sum(),
        ranks,
        revision,
        dirty,
    );
    manifest.seed = seed;
    manifest.max_iterations = iterations as u32;
    if street == 2 {
        manifest.centroid = centroid.into();
        manifest.initialization = "k50-weighted-sample-200000".into();
        manifest.restarts = 1;
    }
    atomic_write(&table_path, &bytes)?;
    atomic_write(&manifest_path, &serde_json::to_vec_pretty(&manifest)?)?;
    LibratusBucketTable::read_verified(&table_path, &manifest_path)?;
    Ok(())
}

#[cfg(test)]
#[test]
fn libratus_parameters_reject_full_deck_and_large_k() {
    assert!(validate_libratus_parameters(7, &[50, 200], 50).is_ok());
    assert!(validate_libratus_parameters(13, &[50], 50).is_err());
    assert!(validate_libratus_parameters(7, &[1000], 50).is_err());
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

fn write_emd_diagnostic(
    out: &Path,
    stage: &str,
    error: &EmdContextError,
) -> std::io::Result<PathBuf> {
    let path = out.join(format!("emd-failure-{stage}.txt"));
    let code_revision = git_metadata()
        .map(|(revision, _)| revision)
        .unwrap_or_else(|error| format!("unavailable: {error}"));
    let diagnostic = format!(
        "code_revision={}\nstage={stage}\n{error:#?}\n",
        code_revision,
    );
    atomic_write(&path, diagnostic.as_bytes())?;
    Ok(path)
}

fn abort_emd(out: &Path, stage: &str, error: EmdContextError) -> ! {
    let path = write_emd_diagnostic(out, stage, &error).expect("write EMD failure diagnostic");
    panic!(
        "EMD failed during {stage}; diagnostic saved to {}",
        path.display()
    );
}

fn git_metadata_with<F>(mut run_git: F) -> Result<(String, bool), String>
where
    F: FnMut(&[&str]) -> Result<String, String>,
{
    let revision = run_git(&["rev-parse", "HEAD"])?;
    let revision = revision.trim();
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("git rev-parse did not return a full SHA-1 revision".to_owned());
    }
    let status = run_git(&["status", "--porcelain"])?;
    Ok((revision.to_owned(), !status.is_empty()))
}

fn git_metadata() -> Result<(String, bool), String> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "cannot determine repository root from Cargo manifest".to_owned())?;
    git_metadata_with(|args| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(repository_root)
            .output()
            .map_err(|error| format!("cannot run git: {error}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        String::from_utf8(output.stdout)
            .map_err(|error| format!("git output is not UTF-8: {error}"))
    })
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
    if arg(args, "--abstraction", "equity") == "libratus-ochs" {
        build_libratus(args).unwrap_or_else(|error| panic!("ошибка libratus-ochs: {error}"));
        return;
    }
    let out = PathBuf::from(arg(args, "--out", "buckets"));
    let ranks: u8 = arg(args, "--ranks", "13").parse().unwrap();
    let bins: usize = arg(args, "--bins", "50").parse().unwrap();
    let seed: u64 = arg(args, "--seed", "202610050003").parse().unwrap();
    let (
        iterations,
        turn_iterations,
        flop_iterations,
        turn_iterations_explicit,
        flop_iterations_explicit,
    ) = iteration_limits(args);
    let ks: Vec<usize> = arg(args, "--ks", "50,200")
        .split(',')
        .map(|k| k.parse().unwrap())
        .collect();
    let abstraction = arg(args, "--abstraction", "equity");
    let diagnose_final_assignments = args
        .iter()
        .any(|argument| argument == "--diagnose-final-assignments");
    let profile_flop_emd = args.iter().any(|argument| argument == "--profile-flop-emd");
    assert!(
        matches!(abstraction.as_str(), "equity" | "potential"),
        "--abstraction must be equity or potential"
    );
    validate_flop_iterations_abstraction(&abstraction, flop_iterations_explicit);
    fs::create_dir_all(&out).unwrap();
    let log = Log {
        started: Instant::now(),
        status: out.join("status.json"),
    };
    let (code_revision, working_tree_dirty) = git_metadata()
        .unwrap_or_else(|error| panic!("verified Git provenance is required: {error}"));
    let deck = deck(ranks);
    let mut summary = vec![format!(
        "\"format\": \"{}\", \"ranks\": {ranks}, \"deck_cards\": {}, \"bins\": {bins}, \"seed\": {seed}, \"ks\": {ks:?}, \"iterations\": {iterations}, \"turn_iterations\": {turn_iterations}, \"turn_iterations_explicit\": {turn_iterations_explicit}, \"flop_iterations\": {flop_iterations}, \"flop_iterations_explicit\": {flop_iterations_explicit}, \"threads\": {}, \"code_revision\": \"{}\", \"working_tree_dirty\": {}",
        if abstraction == "potential" { "hu20-potential-aware-emd-v1" } else { "hu20-equity-buckets-v1" }, deck.len(), rayon::current_num_threads(), code_revision, working_tree_dirty)];

    log.stage("river-equity", "computing");
    let river = RiverTable::build(&deck);
    log.stage("river-equity", &format!("{} classes", river.keys.len()));
    if abstraction == "potential" {
        let mut parts = Vec::new();
        let mut profiling_parts = Vec::new();
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
                turn_iterations,
                diagnose_final_assignments,
            );
            for (iteration, diagnostic) in turn_clusters.diagnostics.iter().enumerate() {
                log.stage("turn-clustering", &format!("k={k}, iteration={}, objective_before={:.9}, objective_after={:.9}, moved_weight_fraction={:.9}, occupied_buckets={}", iteration + 1, diagnostic.objective_before_update, diagnostic.objective_after_update, diagnostic.moved_weight_fraction, diagnostic.occupied_buckets));
            }
            if let Some(diagnostic) = &turn_clusters.final_assignment_diagnostic {
                log.stage("turn-clustering", &format!("k={k}, final_assignment_diagnostic=true, changed_weight_fraction={:.9}, objective={:.9}", diagnostic.changed_weight_fraction, diagnostic.objective));
            }
            if let Some(termination) = turn_clusters.termination {
                log.stage(
                    "turn-clustering",
                    &format!(
                        "k={k}, termination={termination:?}, final_assignment_consistent={}",
                        turn_clusters.final_assignment_consistent
                    ),
                );
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
            let turn_ground_started = profile_flop_emd.then(Instant::now);
            let profiled_turn_ground = profile_flop_emd.then(|| {
                centroid_emd_matrix_profiled(&turn_clusters.centroids, k, &river_ground)
                    .unwrap_or_else(|error| abort_emd(&out, "turn-ground-matrix", error))
            });
            let (turn_ground, turn_ground_emd_calls) = match profiled_turn_ground {
                Some(profiled) => (profiled.matrix, Some(profiled.emd_calls)),
                None => (
                    centroid_emd_matrix(&turn_clusters.centroids, k, &river_ground)
                        .unwrap_or_else(|error| abort_emd(&out, "turn-ground-matrix", error)),
                    None,
                ),
            };
            let turn_ground_profile = turn_ground_started
                .zip(turn_ground_emd_calls)
                .map(|(started, emd_calls)| (started.elapsed(), emd_calls));
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
            let profiled_flop_clusters = kmeans_emd_profiled(
                &flop.distribution,
                k,
                &flop.weight,
                &turn_ground,
                k,
                seed,
                flop_iterations,
                Some(flop_progress),
                diagnose_final_assignments,
                profile_flop_emd,
            )
            .unwrap_or_else(|error| abort_emd(&out, "flop-clustering", error));
            let flop_profile = profiled_flop_clusters.profile;
            let flop_clusters = profiled_flop_clusters.clustering;
            for (iteration, diagnostic) in flop_clusters.diagnostics.iter().enumerate() {
                log.stage("flop-clustering", &format!("k={k}, iteration={}, objective_before={:.9}, objective_after={:.9}, moved_weight_fraction={:.9}, occupied_buckets={}", iteration + 1, diagnostic.objective_before_update, diagnostic.objective_after_update, diagnostic.moved_weight_fraction, diagnostic.occupied_buckets));
            }
            if let Some(termination) = flop_clusters.termination {
                log.stage(
                    "flop-clustering",
                    &format!(
                        "k={k}, termination={termination:?}, final_assignment_consistent={}",
                        flop_clusters.final_assignment_consistent
                    ),
                );
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
            if let (Some((turn_ground_wall_clock, turn_ground_emd_calls)), Some(flop_profile)) =
                (turn_ground_profile, flop_profile)
            {
                let iterations = flop_profile
                    .iterations
                    .iter()
                    .enumerate()
                    .map(|(index, iteration)| format!(
                        "{{\"iteration\": {}, \"assignment_wall_seconds\": {:.9}, \"assignment_emd_calls\": {}, \"centroid_update_wall_seconds\": {:.9}, \"objective_after_update_wall_seconds\": {:.9}, \"objective_after_update_emd_calls\": {}}}",
                        index + 1,
                        iteration.assignment_wall_clock.as_secs_f64(),
                        iteration.assignment_emd_calls,
                        iteration.centroid_update_wall_clock.as_secs_f64(),
                        iteration.objective_after_update_wall_clock.as_secs_f64(),
                        iteration.objective_after_update_emd_calls,
                    ))
                    .collect::<Vec<_>>()
                    .join(", ");
                let final_diagnostic = match (
                    flop_profile.final_assignment_diagnostic_wall_clock,
                    flop_profile.final_assignment_diagnostic_emd_calls,
                ) {
                    (Some(wall_clock), Some(emd_calls)) => format!(
                        "{{\"wall_seconds\": {:.9}, \"emd_calls\": {emd_calls}}}",
                        wall_clock.as_secs_f64(),
                    ),
                    _ => "null".to_owned(),
                };
                let final_assignment_gate = match (
                    flop_profile.final_assignment_gate_wall_clock,
                    flop_profile.final_assignment_gate_emd_calls,
                ) {
                    (Some(wall_clock), Some(emd_calls)) => format!(
                        "{{\"wall_seconds\": {:.9}, \"emd_calls\": {emd_calls}}}",
                        wall_clock.as_secs_f64(),
                    ),
                    _ => "null".to_owned(),
                };
                profiling_parts.push(format!(
                    "{{\"k\": {k}, \"turn_ground_matrix\": {{\"wall_seconds\": {:.9}, \"emd_calls\": {turn_ground_emd_calls}}}, \"flop_clustering_wall_seconds\": {:.9}, \"flop_iterations\": [{iterations}], \"final_assignment_gate\": {final_assignment_gate}, \"final_assignment_diagnostic\": {final_diagnostic}}}",
                    turn_ground_wall_clock.as_secs_f64(),
                    flop_profile.wall_clock.as_secs_f64(),
                ));
            }
            let flop_termination = flop_clusters
                .termination
                .map(|termination| format!("{termination:?}"))
                .unwrap_or_else(|| "Unrecorded".to_owned());
            parts.push(format!("{{\"k\": {k}, \"river_classes\": {}, \"turn_classes\": {}, \"flop_classes\": {}, \"river_objective\": {:?}, \"turn_objective\": {:?}, \"flop_objective\": {:?}, \"flop_termination\": {:?}, \"flop_final_assignment_consistent\": {}}}", river.keys.len(), turn.keys.len(), flop.keys.len(), river_clusters.objective, turn_clusters.objective, flop_clusters.objective, flop_termination, flop_clusters.final_assignment_consistent));
        }
        summary.push(format!("\"tables\": [{}]", parts.join(", ")));
        if !profiling_parts.is_empty() {
            summary.push(format!("\"profiling\": [{}]", profiling_parts.join(", ")));
        }
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
        Some("verify-libratus") => {
            use sha2::{Digest, Sha256};
            let table_path = args.get(2).expect("нужен путь к таблице");
            let manifest_path = args.get(3).expect("нужен путь к манифесту");
            let table =
                LibratusBucketTable::read_verified(Path::new(table_path), Path::new(manifest_path))
                    .expect("проверка таблицы и манифеста");
            println!("{:x}", Sha256::digest(table.serialize()));
        }
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
    fn iteration_limits_preserve_turn_default_without_override() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--iterations".to_owned(),
            "10".to_owned(),
        ];

        assert_eq!(iteration_limits(&args), (10, 10, 10, false, false));
    }

    #[test]
    fn iteration_limits_apply_turn_override_without_changing_global_budget() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--iterations".to_owned(),
            "10".to_owned(),
            "--turn-iterations".to_owned(),
            "50".to_owned(),
        ];

        assert_eq!(iteration_limits(&args), (10, 50, 10, true, false));
    }

    #[test]
    fn iteration_limits_preserve_flop_default_without_override() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--iterations".to_owned(),
            "10".to_owned(),
            "--turn-iterations".to_owned(),
            "50".to_owned(),
        ];

        assert_eq!(iteration_limits(&args), (10, 50, 10, true, false));
    }

    #[test]
    fn iteration_limits_apply_flop_override_without_changing_river_or_turn() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--iterations".to_owned(),
            "10".to_owned(),
            "--turn-iterations".to_owned(),
            "50".to_owned(),
            "--flop-iterations".to_owned(),
            "30".to_owned(),
        ];

        assert_eq!(iteration_limits(&args), (10, 50, 30, true, true));
    }

    #[test]
    #[should_panic(expected = "--flop-iterations must be a positive integer")]
    fn iteration_limits_reject_zero_flop_override() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--flop-iterations".to_owned(),
            "0".to_owned(),
        ];

        iteration_limits(&args);
    }

    #[test]
    #[should_panic(expected = "--flop-iterations must be a positive integer")]
    fn iteration_limits_reject_non_numeric_flop_override() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--flop-iterations".to_owned(),
            "many".to_owned(),
        ];

        iteration_limits(&args);
    }

    #[test]
    #[should_panic(expected = "--flop-iterations requires a value")]
    fn iteration_limits_reject_missing_flop_override_value() {
        let args = vec![
            "hu20-buckets".to_owned(),
            "build".to_owned(),
            "--flop-iterations".to_owned(),
        ];

        iteration_limits(&args);
    }

    #[test]
    #[should_panic(expected = "--flop-iterations requires --abstraction potential")]
    fn flop_override_rejects_equity_abstraction() {
        validate_flop_iterations_abstraction("equity", true);
    }

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
    fn git_metadata_requires_verified_revision_and_status() {
        let mut responses = [
            Ok("0123456789abcdef0123456789abcdef01234567\n".to_owned()),
            Ok(String::new()),
        ]
        .into_iter();

        let metadata = git_metadata_with(|_| responses.next().expect("expected git command"));

        assert_eq!(
            metadata.expect("verified metadata"),
            ("0123456789abcdef0123456789abcdef01234567".to_owned(), false)
        );
    }

    #[test]
    fn git_metadata_rejects_failed_status_check() {
        let mut responses = [
            Ok("0123456789abcdef0123456789abcdef01234567\n".to_owned()),
            Err("git status failed".to_owned()),
        ]
        .into_iter();

        let error = git_metadata_with(|_| responses.next().expect("expected git command"))
            .expect_err("failed git status must not become clean metadata");

        assert_eq!(error, "git status failed");
    }

    #[test]
    fn emd_failure_diagnostic_preserves_context_without_complete_status() {
        let directory =
            std::env::temp_dir().join(format!("hu20-emd-failure-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        atomic_write(
            &directory.join("status.json"),
            b"{\"stage\":\"flop-clustering\"}\n",
        )
        .unwrap();
        let error = EmdContextError {
            phase: "assignment",
            point_index: 7,
            centroid_index: 3,
            left: vec![1.0, 0.0],
            right: vec![0.0, 1.0],
            ground: vec![0.0, 1.0, 1.0, 0.0],
            source: EmdError::DisconnectedTransportGraph,
        };
        let diagnostic = write_emd_diagnostic(&directory, "flop-clustering", &error).unwrap();
        let text = fs::read_to_string(diagnostic).unwrap();
        assert!(text.contains("stage=flop-clustering"));
        assert!(text.contains("point_index: 7"));
        assert!(text.contains("centroid_index: 3"));
        assert!(text.contains("DisconnectedTransportGraph"));
        assert_ne!(
            fs::read_to_string(directory.join("status.json")).unwrap(),
            "{\"stage\":\"complete\"}\n"
        );
        assert!(!directory.join("summary.json").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}
