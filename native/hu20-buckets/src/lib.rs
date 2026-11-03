//! Equity-distribution card abstraction tables for heads-up Hold'em.
//!
//! Cards are `rank * 4 + suit`, ranks 0..13 for 2..A and suits 0..4 for c, d, h, s.
//! A situation is a two-card holding plus a board. Suit-isomorphic situations
//! share one class, keyed by the sorted per-suit (holding mask, board mask)
//! signatures, so every class is computed once and weighted by its orbit size.

use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const RANKS: &[u8] = b"23456789TJQKA";
pub const SUITS: &[u8] = b"cdhs";

pub fn parse_card(text: &str) -> u8 {
    let bytes = text.as_bytes();
    assert!(bytes.len() == 2, "card {text}");
    let rank = RANKS.iter().position(|&r| r == bytes[0]).expect("rank") as u8;
    let suit = SUITS.iter().position(|&s| s == bytes[1]).expect("suit") as u8;
    rank * 4 + suit
}

pub fn card_name(card: u8) -> String {
    format!(
        "{}{}",
        RANKS[(card / 4) as usize] as char,
        SUITS[(card % 4) as usize] as char
    )
}

/// Cards whose rank is among the top `ranks` ranks; 13 is the full deck.
pub fn deck(ranks: u8) -> Vec<u8> {
    (0..52u8).filter(|c| c / 4 >= 13 - ranks).collect()
}

fn straight_high(mask: u16) -> Option<u32> {
    for high in (4..13u32).rev() {
        let window = 0b11111u16 << (high - 4);
        if mask & window == window {
            return Some(high);
        }
    }
    if mask & 0x100F == 0x100F {
        Some(3)
    } else {
        None
    }
}

fn top(mask: u16, n: usize) -> u32 {
    let mut value = 0u32;
    let mut taken = 0;
    for rank in (0..13u32).rev() {
        if taken == n {
            break;
        }
        if mask & (1 << rank) != 0 {
            value = (value << 4) | rank;
            taken += 1;
        }
    }
    value
}

/// Hand strength of five to seven cards; larger is better, equal is a tie.
pub fn evaluate(cards: &[u8]) -> u32 {
    let mut suits = [0u16; 4];
    let mut counts = [0u8; 13];
    let mut all = 0u16;
    for &card in cards {
        let (rank, suit) = ((card / 4) as usize, (card % 4) as usize);
        suits[suit] |= 1 << rank;
        counts[rank] += 1;
        all |= 1 << rank;
    }
    // A flush excludes quads and full houses within seven cards.
    for mask in suits {
        if mask.count_ones() >= 5 {
            return match straight_high(mask) {
                Some(high) => (8 << 20) | high,
                None => (5 << 20) | top(mask, 5),
            };
        }
    }
    let (mut quads, mut trips, mut pairs) = (None, 0u16, 0u16);
    for rank in 0..13 {
        match counts[rank] {
            4 => quads = Some(rank as u32),
            3 => trips |= 1 << rank,
            2 => pairs |= 1 << rank,
            _ => {}
        }
    }
    if let Some(q) = quads {
        return (7 << 20) | (q << 4) | top(all & !(1 << q), 1);
    }
    if trips != 0 {
        let t = 15 - trips.leading_zeros();
        let rest = (trips & !(1 << t)) | pairs;
        if rest != 0 {
            return (6 << 20) | (t << 4) | (15 - rest.leading_zeros());
        }
    }
    if let Some(high) = straight_high(all) {
        return (4 << 20) | high;
    }
    if trips != 0 {
        let t = 15 - trips.leading_zeros();
        return (3 << 20) | (t << 8) | top(all & !(1 << t), 2);
    }
    if pairs.count_ones() >= 2 {
        let high = 15 - pairs.leading_zeros();
        let low = 15 - (pairs & !(1 << high)).leading_zeros();
        return (2 << 20) | (high << 8) | (low << 4) | top(all & !(1 << high) & !(1 << low), 1);
    }
    if pairs != 0 {
        let p = 15 - pairs.leading_zeros();
        return (1 << 20) | (p << 12) | top(all & !(1 << p), 3);
    }
    top(all, 5)
}

pub fn category(value: u32) -> u32 {
    value >> 20
}

fn signatures(hole: &[u8], board: &[u8]) -> [u32; 4] {
    let mut sig = [0u32; 4];
    for &card in hole {
        sig[(card % 4) as usize] |= 1 << (13 + card / 4);
    }
    for &card in board {
        sig[(card % 4) as usize] |= 1 << (card / 4);
    }
    sig
}

/// Suit-isomorphism class key: per-suit 26-bit signatures, sorted, packed.
pub fn class_key(hole: &[u8], board: &[u8]) -> u128 {
    let mut sig = signatures(hole, board);
    sig.sort_unstable_by(|a, b| b.cmp(a));
    sig.iter().fold(0u128, |key, &s| (key << 26) | s as u128)
}

/// Raw situations in the class: 24 suit permutations over the stabilizer size.
pub fn orbit_size(hole: &[u8], board: &[u8]) -> u64 {
    let mut sig = signatures(hole, board);
    sig.sort_unstable();
    let mut stabilizer = 1u64;
    let mut run = 1u64;
    for i in 1..4 {
        if sig[i] == sig[i - 1] {
            run += 1;
            stabilizer *= run;
        } else {
            run = 1;
        }
    }
    24 / stabilizer
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// The 64-bit table key; the build refuses any collision among its classes.
pub fn hash_key(key: u128) -> u64 {
    splitmix(((key >> 64) as u64) ^ splitmix(key as u64))
}

/// One street's table file (`HU20BKT1`): street code (1 flop, 2 turn, 3 river), K, count,
/// sorted 64-bit class hashes, then their u16 bucket ids, all little-endian.
pub struct BucketTable {
    pub street: u32,
    pub k: u32,
    hashes: Vec<u64>,
    buckets: Vec<u16>,
}

impl BucketTable {
    const MAGIC: &'static [u8; 8] = b"HU20BKT1";

    pub fn new(street: u32, k: u32, keys: &[u128], assignment: &[u16]) -> BucketTable {
        let mut rows: Vec<(u64, u16)> = keys
            .iter()
            .map(|&key| hash_key(key))
            .zip(assignment.iter().copied())
            .collect();
        rows.sort_unstable_by_key(|r| r.0);
        assert!(
            rows.windows(2).all(|w| w[0].0 != w[1].0),
            "64-bit class hash collision"
        );
        BucketTable {
            street,
            k,
            hashes: rows.iter().map(|r| r.0).collect(),
            buckets: rows.iter().map(|r| r.1).collect(),
        }
    }

    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut out = Vec::with_capacity(24 + self.hashes.len() * 10);
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&self.street.to_le_bytes());
        out.extend_from_slice(&self.k.to_le_bytes());
        out.extend_from_slice(&(self.hashes.len() as u64).to_le_bytes());
        for h in &self.hashes {
            out.extend_from_slice(&h.to_le_bytes());
        }
        for b in &self.buckets {
            out.extend_from_slice(&b.to_le_bytes());
        }
        std::fs::write(path, out)
    }

    /// Reads and validates a table without holding a second copy of it.
    pub fn read(path: &std::path::Path) -> std::io::Result<BucketTable> {
        use std::io::{Error, ErrorKind, Read};
        let invalid =
            |why: &str| Error::new(ErrorKind::InvalidData, format!("{}: {why}", path.display()));
        let file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let mut input = std::io::BufReader::with_capacity(1 << 20, file);
        let mut header = [0u8; 24];
        input.read_exact(&mut header)?;
        if &header[..8] != Self::MAGIC {
            return Err(invalid("not an HU20 bucket table"));
        }
        let word = |i: usize| u32::from_le_bytes(header[i..i + 4].try_into().unwrap());
        let (street, k) = (word(8), word(12));
        let count = u64::from_le_bytes(header[16..24].try_into().unwrap());
        if !(1..=3).contains(&street)
            || k == 0
            || k > u16::MAX as u32 + 1
            || size != 24 + 10 * count
        {
            return Err(invalid("inconsistent header"));
        }
        let count = count as usize;
        let mut hashes = Vec::with_capacity(count);
        let mut buckets = Vec::with_capacity(count);
        let mut chunk = vec![0u8; 1 << 16];
        while hashes.len() < count {
            let n = (count - hashes.len()).min(chunk.len() / 8);
            input.read_exact(&mut chunk[..8 * n])?;
            hashes.extend(
                chunk[..8 * n]
                    .chunks_exact(8)
                    .map(|b| u64::from_le_bytes(b.try_into().unwrap())),
            );
        }
        while buckets.len() < count {
            let n = (count - buckets.len()).min(chunk.len() / 2);
            input.read_exact(&mut chunk[..2 * n])?;
            buckets.extend(
                chunk[..2 * n]
                    .chunks_exact(2)
                    .map(|b| u16::from_le_bytes(b.try_into().unwrap())),
            );
        }
        if !hashes.windows(2).all(|w| w[0] < w[1]) || buckets.iter().any(|&b| b as u32 >= k) {
            return Err(invalid("unsorted hashes or bucket out of range"));
        }
        Ok(BucketTable {
            street,
            k,
            hashes,
            buckets,
        })
    }

    /// The bucket of a holding on this street's board, or None if its class is absent.
    pub fn bucket(&self, hole: &[u8], board: &[u8]) -> Option<u16> {
        assert_eq!(
            board.len() as u32,
            self.street + 2,
            "board size belongs to another street's table"
        );
        let target = hash_key(class_key(hole, board));
        self.hashes
            .binary_search(&target)
            .ok()
            .map(|i| self.buckets[i])
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }
}

pub fn combinations(cards: &[u8], k: usize) -> Vec<Vec<u8>> {
    fn go(cards: &[u8], k: usize, start: usize, current: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
        if current.len() == k {
            out.push(current.clone());
            return;
        }
        for i in start..cards.len() {
            if cards.len() - i < k - current.len() {
                break;
            }
            current.push(cards[i]);
            go(cards, k, i + 1, current, out);
            current.pop();
        }
    }
    let mut out = Vec::new();
    go(cards, k, 0, &mut Vec::new(), &mut out);
    out
}

/// One representative board per suit-isomorphism class.
pub fn canonical_boards(deck: &[u8], size: usize) -> Vec<Vec<u8>> {
    let mut keyed: Vec<(u128, Vec<u8>)> = combinations(deck, size)
        .into_iter()
        .map(|b| (class_key(&[], &b), b))
        .collect();
    keyed.par_sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    keyed.dedup_by(|a, b| a.0 == b.0);
    keyed.into_iter().map(|(_, b)| b).collect()
}

/// Equity of every holding on a complete board against a uniform random
/// opponent holding, with exact card removal; ties count one half.
pub fn river_equities(board: &[u8], deck: &[u8]) -> Vec<([u8; 2], f32)> {
    let rest: Vec<u8> = deck
        .iter()
        .copied()
        .filter(|c| !board.contains(c))
        .collect();
    let mut hands: Vec<([u8; 2], u32)> = Vec::with_capacity(rest.len() * rest.len() / 2);
    let mut cards = [0u8; 7];
    cards[2..].copy_from_slice(board);
    for i in 0..rest.len() {
        for j in i + 1..rest.len() {
            cards[0] = rest[i];
            cards[1] = rest[j];
            hands.push(([rest[i], rest[j]], evaluate(&cards)));
        }
    }
    hands.sort_unstable_by_key(|h| h.1);
    let opponents = ((rest.len() - 2) * (rest.len() - 3) / 2) as f32;
    let mut lower = [0u32; 52];
    let mut lower_total = 0u32;
    let mut out = Vec::with_capacity(hands.len());
    let mut start = 0;
    while start < hands.len() {
        let mut end = start;
        while end < hands.len() && hands[end].1 == hands[start].1 {
            end += 1;
        }
        let mut group = [0u32; 52];
        for h in &hands[start..end] {
            group[h.0[0] as usize] += 1;
            group[h.0[1] as usize] += 1;
        }
        let size = (end - start) as u32;
        for h in &hands[start..end] {
            let (a, b) = (h.0[0] as usize, h.0[1] as usize);
            let wins = lower_total - lower[a] - lower[b];
            let ties = size + 1 - group[a] - group[b];
            out.push((h.0, (wins as f32 + 0.5 * ties as f32) / opponents));
        }
        for h in &hands[start..end] {
            lower[h.0[0] as usize] += 1;
            lower[h.0[1] as usize] += 1;
        }
        lower_total += size;
        start = end;
    }
    out
}

/// River classes, sorted by key, with equity and orbit weight.
pub struct RiverTable {
    pub keys: Vec<u128>,
    pub equity: Vec<f32>,
    pub weight: Vec<u64>,
}

impl RiverTable {
    pub fn build(deck: &[u8]) -> RiverTable {
        let boards = canonical_boards(deck, 5);
        let mut rows: Vec<(u128, f32, u64)> = boards
            .par_iter()
            .flat_map_iter(|board| {
                river_equities(board, deck)
                    .into_iter()
                    .map(move |(hole, eq)| (class_key(&hole, board), eq, orbit_size(&hole, board)))
            })
            .collect();
        rows.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        rows.dedup_by(|a, b| {
            if a.0 == b.0 {
                assert!(a.1 == b.1, "isomorphic river situations disagree on equity");
                true
            } else {
                false
            }
        });
        RiverTable {
            keys: rows.iter().map(|r| r.0).collect(),
            equity: rows.iter().map(|r| r.1).collect(),
            weight: rows.iter().map(|r| r.2).collect(),
        }
    }

    pub fn equity_of(&self, hole: &[u8], board: &[u8]) -> f32 {
        let key = class_key(hole, board);
        let index = self.keys.binary_search(&key).expect("river class missing");
        self.equity[index]
    }
}

fn bin(equity: f32, bins: usize) -> usize {
    ((equity * bins as f32) as usize).min(bins - 1)
}

/// Classes of an earlier street with an equity histogram over all runouts.
pub struct StreetFeatures {
    pub keys: Vec<u128>,
    pub weight: Vec<u64>,
    pub mean_equity: Vec<f32>,
    /// Row-major cumulative histograms (CDFs), `bins` values per class.
    pub cdf: Vec<f32>,
    pub bins: usize,
}

impl StreetFeatures {
    pub fn build(
        deck: &[u8],
        board_size: usize,
        bins: usize,
        river: &RiverTable,
    ) -> StreetFeatures {
        let boards = canonical_boards(deck, board_size);
        let mut reps: Vec<(u128, [u8; 2], Vec<u8>)> = boards
            .par_iter()
            .flat_map_iter(|board| {
                let rest: Vec<u8> = deck
                    .iter()
                    .copied()
                    .filter(|c| !board.contains(c))
                    .collect();
                let mut found = Vec::new();
                for i in 0..rest.len() {
                    for j in i + 1..rest.len() {
                        let hole = [rest[i], rest[j]];
                        found.push((class_key(&hole, board), hole, board.clone()));
                    }
                }
                found
            })
            .collect();
        reps.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        reps.dedup_by(|a, b| a.0 == b.0);
        let rows: Vec<(u64, f32, Vec<f32>)> = reps
            .par_iter()
            .map(|(_, hole, board)| {
                let rest: Vec<u8> = deck
                    .iter()
                    .copied()
                    .filter(|c| !board.contains(c) && !hole.contains(c))
                    .collect();
                let mut counts = vec![0u32; bins];
                let (mut total, mut sum) = (0u32, 0f64);
                for runout in combinations(&rest, 5 - board_size) {
                    let mut full = board.clone();
                    full.extend_from_slice(&runout);
                    let eq = river.equity_of(hole, &full);
                    counts[bin(eq, bins)] += 1;
                    total += 1;
                    sum += eq as f64;
                }
                let mut running = 0u32;
                let cdf = counts
                    .iter()
                    .map(|&c| {
                        running += c;
                        running as f32 / total as f32
                    })
                    .collect();
                (orbit_size(hole, board), (sum / total as f64) as f32, cdf)
            })
            .collect();
        StreetFeatures {
            keys: reps.iter().map(|r| r.0).collect(),
            weight: rows.iter().map(|r| r.0).collect(),
            mean_equity: rows.iter().map(|r| r.1).collect(),
            cdf: rows.into_iter().flat_map(|r| r.2).collect(),
            bins,
        }
    }
}

/// Классы улицы как распределения по бакетам следующей улицы.
pub struct PotentialFeatures {
    pub keys: Vec<u128>,
    pub weight: Vec<u64>,
    /// Построчные нормированные распределения; в строке `dimension` масс.
    pub distribution: Vec<f32>,
    pub dimension: usize,
}

impl PotentialFeatures {
    pub fn build(
        deck: &[u8],
        board_size: usize,
        next_keys: &[u128],
        next_assignment: &[u16],
        dimension: usize,
    ) -> PotentialFeatures {
        assert!(board_size == 3 || board_size == 4);
        assert!(next_keys.len() == next_assignment.len());
        assert!(next_keys.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(next_assignment
            .iter()
            .all(|&bucket| (bucket as usize) < dimension));
        let boards = canonical_boards(deck, board_size);
        let mut reps: Vec<(u128, [u8; 2], Vec<u8>)> = boards
            .par_iter()
            .flat_map_iter(|board| {
                let rest: Vec<u8> = deck
                    .iter()
                    .copied()
                    .filter(|card| !board.contains(card))
                    .collect();
                let mut found = Vec::new();
                for i in 0..rest.len() {
                    for j in i + 1..rest.len() {
                        let hole = [rest[i], rest[j]];
                        found.push((class_key(&hole, board), hole, board.clone()));
                    }
                }
                found
            })
            .collect();
        reps.par_sort_unstable_by(|left, right| left.0.cmp(&right.0));
        reps.dedup_by(|left, right| left.0 == right.0);
        let processed = Arc::new(AtomicUsize::new(0));
        let last_report = Arc::new(Mutex::new(Instant::now()));
        let progress_started = Instant::now();
        let total = reps.len();
        let rows: Vec<(u64, Vec<f32>)> = reps
            .par_iter()
            .map(|(_, hole, board)| {
                let rest: Vec<u8> = deck.iter().copied().filter(|card| !board.contains(card) && !hole.contains(card)).collect();
                let mut distribution = vec![0.0; dimension];
                let row = if board_size == 4 {
                    for river_card in &rest {
                        let mut river_board = board.clone();
                        river_board.push(*river_card);
                        let key = class_key(hole, &river_board);
                        let index = next_keys.binary_search(&key).expect("river class missing");
                        distribution[next_assignment[index] as usize] += 1.0;
                    }
                    let total = rest.len() as f32;
                    for value in &mut distribution { *value /= total; }
                    (orbit_size(hole, board), distribution)
                } else {
                    for turn_index in 0..rest.len() {
                        let turn_card = rest[turn_index];
                        let mut turn_board = board.clone();
                        turn_board.push(turn_card);
                        let key = class_key(hole, &turn_board);
                        let index = next_keys.binary_search(&key).expect("turn class missing");
                        distribution[next_assignment[index] as usize] += 1.0;
                    }
                    for value in &mut distribution { *value /= rest.len() as f32; }
                    (orbit_size(hole, board), distribution)
                };
                let completed = processed.fetch_add(1, Ordering::Relaxed) + 1;
                let mut last = last_report.lock().expect("progress log");
                if completed == total || last.elapsed() >= Duration::from_secs(3) {
                    eprintln!("potential-features board_size={board_size}: {completed}/{total}, elapsed_seconds={:.1}", progress_started.elapsed().as_secs_f64());
                    *last = Instant::now();
                }
                row
            })
            .collect();
        PotentialFeatures {
            keys: reps.iter().map(|row| row.0).collect(),
            weight: rows.iter().map(|row| row.0).collect(),
            distribution: rows.into_iter().flat_map(|row| row.1).collect(),
            dimension,
        }
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = splitmix(self.0);
        self.0
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn l1(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
}

#[derive(Clone)]
struct FlowEdge {
    to: usize,
    reverse: usize,
    capacity: f64,
    cost: f64,
}

/// Причина, по которой точный транспортный solver не может безопасно вернуть EMD.
#[derive(Debug, Clone, PartialEq)]
pub enum EmdError {
    NonFiniteMass { side: &'static str, index: usize },
    NegativeMass { side: &'static str, index: usize },
    NonPositiveTotalMass { side: &'static str },
    NonFiniteGroundDistance { index: usize },
    NegativeGroundDistance { index: usize },
    DisconnectedTransportGraph,
    PredecessorCycle { node: usize },
    MissingPredecessor { node: usize },
    InvalidAugmentation,
    NoProgress,
    IterationLimit,
    IncompleteTransport,
    NonFiniteResult,
}

/// Контекст пары распределений, для которой общий EMD не смог вернуть результат.
#[derive(Debug, Clone, PartialEq)]
pub struct EmdContextError {
    pub phase: &'static str,
    pub point_index: usize,
    pub centroid_index: usize,
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub ground: Vec<f32>,
    pub source: EmdError,
}

fn emd_with_context(
    phase: &'static str,
    point_index: usize,
    centroid_index: usize,
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
) -> Result<f32, EmdContextError> {
    try_emd(left, right, ground, dimension).map_err(|source| EmdContextError {
        phase,
        point_index,
        centroid_index,
        left: left.to_vec(),
        right: right.to_vec(),
        ground: ground.to_vec(),
        source,
    })
}

/// Выбирает первый минимум, не материализуя временный список расстояний.
fn emd_with_context_with_scratch(
    phase: &'static str,
    point_index: usize,
    centroid_index: usize,
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
    scratch: &mut EmdScratch,
) -> Result<f32, EmdContextError> {
    try_emd_with_scratch(left, right, ground, dimension, scratch).map_err(|source| {
        EmdContextError {
            phase,
            point_index,
            centroid_index,
            left: left.to_vec(),
            right: right.to_vec(),
            ground: ground.to_vec(),
            source,
        }
    })
}

fn first_minimum_emd<I>(mut candidates: I) -> Result<(u16, f32), EmdContextError>
where
    I: Iterator<Item = Result<(u16, f32), EmdContextError>>,
{
    let mut best = candidates
        .next()
        .expect("EMD assignment requires at least one centroid")?;
    for candidate in candidates {
        let candidate = candidate?;
        if candidate.1 < best.1 {
            best = candidate;
        }
    }
    Ok(best)
}

#[cfg(test)]
fn first_minimum_emd_reference(candidates: Vec<(u16, f32)>) -> (u16, f32) {
    candidates
        .into_iter()
        .min_by(|left, right| left.1.partial_cmp(&right.1).unwrap())
        .expect("reference candidates must not be empty")
}

fn add_flow_edge(graph: &mut [Vec<FlowEdge>], from: usize, to: usize, capacity: f64, cost: f64) {
    let forward = FlowEdge {
        to,
        reverse: graph[to].len(),
        capacity,
        cost,
    };
    let backward = FlowEdge {
        to: from,
        reverse: graph[from].len(),
        capacity: 0.0,
        cost: -cost,
    };
    graph[from].push(forward);
    graph[to].push(backward);
}

fn transport_residuals(
    graph: &[Vec<FlowEdge>],
    source: usize,
    sink: usize,
    dimension: usize,
) -> (f64, f64) {
    let source_residual = graph[source].iter().map(|edge| edge.capacity).sum();
    let demand_residual = (0..dimension)
        .map(|index| {
            graph[dimension + index]
                .iter()
                .find(|edge| edge.to == sink)
                .expect("demand node must connect to sink")
                .capacity
        })
        .sum();
    (source_residual, demand_residual)
}

#[cfg(test)]
fn reconstruct_path(
    previous: &[Option<(usize, usize)>],
    source: usize,
    sink: usize,
) -> Result<Vec<(usize, usize)>, EmdError> {
    let mut path = Vec::with_capacity(previous.len());
    let mut seen = vec![false; previous.len()];
    let mut node = sink;
    while node != source {
        if seen[node] {
            return Err(EmdError::PredecessorCycle { node });
        }
        seen[node] = true;
        let (from, edge) = previous[node].ok_or(EmdError::MissingPredecessor { node })?;
        path.push((from, edge));
        node = from;
    }
    Ok(path)
}

fn reconstruct_path_into(
    previous: &[Option<(usize, usize)>],
    source: usize,
    sink: usize,
    path: &mut Vec<(usize, usize)>,
    seen: &mut [bool],
) -> Result<(), EmdError> {
    path.clear();
    seen.fill(false);
    let mut node = sink;
    while node != source {
        if seen[node] {
            return Err(EmdError::PredecessorCycle { node });
        }
        seen[node] = true;
        let (from, edge) = previous[node].ok_or(EmdError::MissingPredecessor { node })?;
        path.push((from, edge));
        node = from;
    }
    Ok(())
}

/// Временные буферы точного EMD. Содержимое всегда сбрасывается перед расчётом;
/// переиспользуется только выделенная память.
#[derive(Default)]
struct EmdScratch {
    normalized_left: Vec<f64>,
    normalized_right: Vec<f64>,
    graph: Vec<Vec<FlowEdge>>,
    potential: Vec<f64>,
    distance: Vec<f64>,
    previous: Vec<Option<(usize, usize)>>,
    used: Vec<bool>,
    path: Vec<(usize, usize)>,
    seen: Vec<bool>,
}

impl EmdScratch {
    fn reset(&mut self, dimension: usize) {
        let nodes = 2 * dimension + 2;
        self.normalized_left.clear();
        self.normalized_right.clear();
        self.graph.resize_with(nodes, Vec::new);
        self.graph.truncate(nodes);
        for edges in &mut self.graph {
            edges.clear();
        }
        self.potential.clear();
        self.potential.resize(nodes, 0.0);
        self.distance.clear();
        self.distance.resize(nodes, f64::INFINITY);
        self.previous.clear();
        self.previous.resize(nodes, None);
        self.used.clear();
        self.used.resize(nodes, false);
        self.path.clear();
        self.seen.clear();
        self.seen.resize(nodes, false);
    }
}

/// Точная стоимость переноса массы между двумя нормированными распределениями.
///
/// Ошибки входа и нарушения численных инвариантов возвращаются явно, чтобы
/// кластеризация не могла незаметно использовать подменённое расстояние.
#[cfg(test)]
// Снимок дорефакторингового ядра commit bbca55febbcf63c483ac829b8241685b751a3779.
fn try_emd_pre_scratch_reference(
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
) -> Result<f32, EmdError> {
    assert!(left.len() == dimension && right.len() == dimension);
    assert!(ground.len() == dimension * dimension);
    for (side, distribution) in [("left", left), ("right", right)] {
        for (index, &mass) in distribution.iter().enumerate() {
            if !mass.is_finite() {
                return Err(EmdError::NonFiniteMass { side, index });
            }
            if mass < 0.0 {
                return Err(EmdError::NegativeMass { side, index });
            }
        }
    }
    for (index, &distance) in ground.iter().enumerate() {
        if !distance.is_finite() {
            return Err(EmdError::NonFiniteGroundDistance { index });
        }
        if distance < 0.0 {
            return Err(EmdError::NegativeGroundDistance { index });
        }
    }
    let left_mass: f64 = left.iter().map(|&x| x as f64).sum();
    let right_mass: f64 = right.iter().map(|&x| x as f64).sum();
    if left_mass <= 0.0 {
        return Err(EmdError::NonPositiveTotalMass { side: "left" });
    }
    if right_mass <= 0.0 {
        return Err(EmdError::NonPositiveTotalMass { side: "right" });
    }
    let normalized_left: Vec<f64> = left.iter().map(|&value| value as f64 / left_mass).collect();
    let normalized_right: Vec<f64> = right
        .iter()
        .map(|&value| value as f64 / right_mass)
        .collect();
    let transport_mass = normalized_left
        .iter()
        .sum::<f64>()
        .min(normalized_right.iter().sum::<f64>());
    let source = 2 * dimension;
    let sink = source + 1;
    let mut graph = vec![Vec::new(); sink + 1];
    for i in 0..dimension {
        let source_mass = normalized_left[i];
        add_flow_edge(&mut graph, source, i, source_mass, 0.0);
        add_flow_edge(&mut graph, dimension + i, sink, normalized_right[i], 0.0);
        for j in 0..dimension {
            add_flow_edge(
                &mut graph,
                i,
                dimension + j,
                1.0,
                ground[i * dimension + j] as f64,
            );
        }
    }
    // Распределения нормированы к масштабу единицы; допуск покрывает только
    // накопление операций f64, а не разрешает отбросить значимую массу.
    let completion_tolerance = 64.0 * f64::EPSILON * dimension as f64;
    let iteration_limit = 4usize.saturating_mul(graph.len().saturating_mul(graph.len()));
    let mut potential = vec![0.0; graph.len()];
    let mut total_cost = 0.0;
    let mut delivered = 0.0;
    let mut complete = false;
    for _ in 0..iteration_limit {
        let (source_residual, demand_residual) =
            transport_residuals(&graph, source, sink, dimension);
        if source_residual <= completion_tolerance && demand_residual <= completion_tolerance {
            complete = true;
            break;
        }
        let mut distance = vec![f64::INFINITY; graph.len()];
        let mut previous = vec![None; graph.len()];
        let mut used = vec![false; graph.len()];
        distance[source] = 0.0;
        for _ in 0..graph.len() {
            let next = (0..graph.len())
                .filter(|&v| !used[v])
                .min_by(|&a, &b| distance[a].partial_cmp(&distance[b]).unwrap())
                .unwrap();
            if !distance[next].is_finite() {
                break;
            }
            used[next] = true;
            if next == sink {
                continue;
            }
            for (edge_index, edge) in graph[next].iter().enumerate() {
                if edge.capacity <= 0.0 || used[edge.to] {
                    continue;
                }
                let candidate = distance[next] + edge.cost + potential[next] - potential[edge.to];
                if candidate < distance[edge.to] {
                    distance[edge.to] = candidate;
                    previous[edge.to] = Some((next, edge_index));
                }
            }
        }
        if !used[sink] {
            return Err(EmdError::DisconnectedTransportGraph);
        }
        for v in 0..graph.len() {
            if used[v] {
                potential[v] += distance[v];
            }
        }
        let path = reconstruct_path(&previous, source, sink)?;
        let mut amount = f64::INFINITY;
        for &(from, edge) in &path {
            amount = amount.min(graph[from][edge].capacity);
        }
        if !amount.is_finite() || amount <= 0.0 {
            return Err(EmdError::InvalidAugmentation);
        }
        let mut residual_capacity_changed = false;
        let mut node = sink;
        for &(from, edge) in &path {
            let reverse = graph[from][edge].reverse;
            total_cost += amount * graph[from][edge].cost;
            let forward_capacity_before = graph[from][edge].capacity;
            let reverse_capacity_before = graph[node][reverse].capacity;
            graph[from][edge].capacity -= amount;
            graph[node][reverse].capacity += amount;
            residual_capacity_changed |= graph[from][edge].capacity != forward_capacity_before
                || graph[node][reverse].capacity != reverse_capacity_before;
            node = from;
        }
        delivered += amount;
        if !delivered.is_finite() || !residual_capacity_changed {
            return Err(EmdError::NoProgress);
        }
    }
    if !complete {
        let (source_residual, demand_residual) =
            transport_residuals(&graph, source, sink, dimension);
        if source_residual <= completion_tolerance && demand_residual <= completion_tolerance {
            complete = true;
        }
    }
    if !complete {
        return Err(EmdError::IterationLimit);
    }
    let (source_residual, demand_residual) = transport_residuals(&graph, source, sink, dimension);
    if source_residual > completion_tolerance || demand_residual > completion_tolerance {
        return Err(EmdError::IncompleteTransport);
    }
    let delivered_by_residual = transport_mass - source_residual;
    if !delivered_by_residual.is_finite()
        || (transport_mass - delivered_by_residual).abs() > completion_tolerance
    {
        return Err(EmdError::IncompleteTransport);
    }
    if !total_cost.is_finite() {
        return Err(EmdError::NonFiniteResult);
    }
    let result = total_cost as f32;
    if !result.is_finite() {
        return Err(EmdError::NonFiniteResult);
    }
    Ok(result)
}

/// Точная стоимость переноса массы между двумя нормированными распределениями.
///
/// Публичный контракт сохраняет независимость вызова; горячий путь
/// кластеризации использует тот же solver с локально переиспользуемым scratch.
pub fn try_emd(
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
) -> Result<f32, EmdError> {
    let mut scratch = EmdScratch::default();
    try_emd_with_scratch(left, right, ground, dimension, &mut scratch)
}

fn try_emd_with_scratch(
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
    scratch: &mut EmdScratch,
) -> Result<f32, EmdError> {
    assert!(left.len() == dimension && right.len() == dimension);
    assert!(ground.len() == dimension * dimension);
    for (side, distribution) in [("left", left), ("right", right)] {
        for (index, &mass) in distribution.iter().enumerate() {
            if !mass.is_finite() {
                return Err(EmdError::NonFiniteMass { side, index });
            }
            if mass < 0.0 {
                return Err(EmdError::NegativeMass { side, index });
            }
        }
    }
    for (index, &distance) in ground.iter().enumerate() {
        if !distance.is_finite() {
            return Err(EmdError::NonFiniteGroundDistance { index });
        }
        if distance < 0.0 {
            return Err(EmdError::NegativeGroundDistance { index });
        }
    }
    let left_mass: f64 = left.iter().map(|&x| x as f64).sum();
    let right_mass: f64 = right.iter().map(|&x| x as f64).sum();
    if left_mass <= 0.0 {
        return Err(EmdError::NonPositiveTotalMass { side: "left" });
    }
    if right_mass <= 0.0 {
        return Err(EmdError::NonPositiveTotalMass { side: "right" });
    }

    scratch.reset(dimension);
    scratch
        .normalized_left
        .extend(left.iter().map(|&value| value as f64 / left_mass));
    scratch
        .normalized_right
        .extend(right.iter().map(|&value| value as f64 / right_mass));
    let transport_mass = scratch
        .normalized_left
        .iter()
        .sum::<f64>()
        .min(scratch.normalized_right.iter().sum::<f64>());
    let source = 2 * dimension;
    let sink = source + 1;
    for i in 0..dimension {
        let source_mass = scratch.normalized_left[i];
        add_flow_edge(&mut scratch.graph, source, i, source_mass, 0.0);
        add_flow_edge(
            &mut scratch.graph,
            dimension + i,
            sink,
            scratch.normalized_right[i],
            0.0,
        );
        for j in 0..dimension {
            add_flow_edge(
                &mut scratch.graph,
                i,
                dimension + j,
                1.0,
                ground[i * dimension + j] as f64,
            );
        }
    }
    let completion_tolerance = 64.0 * f64::EPSILON * dimension as f64;
    let iteration_limit =
        4usize.saturating_mul(scratch.graph.len().saturating_mul(scratch.graph.len()));
    let mut total_cost = 0.0;
    let mut delivered = 0.0;
    let mut complete = false;
    for _ in 0..iteration_limit {
        let (source_residual, demand_residual) =
            transport_residuals(&scratch.graph, source, sink, dimension);
        if source_residual <= completion_tolerance && demand_residual <= completion_tolerance {
            complete = true;
            break;
        }
        scratch.distance.fill(f64::INFINITY);
        scratch.previous.fill(None);
        scratch.used.fill(false);
        scratch.distance[source] = 0.0;
        for _ in 0..scratch.graph.len() {
            let next = (0..scratch.graph.len())
                .filter(|&v| !scratch.used[v])
                .min_by(|&a, &b| {
                    scratch.distance[a]
                        .partial_cmp(&scratch.distance[b])
                        .unwrap()
                })
                .unwrap();
            if !scratch.distance[next].is_finite() {
                break;
            }
            scratch.used[next] = true;
            if next == sink {
                continue;
            }
            for (edge_index, edge) in scratch.graph[next].iter().enumerate() {
                if edge.capacity <= 0.0 || scratch.used[edge.to] {
                    continue;
                }
                let candidate = scratch.distance[next] + edge.cost + scratch.potential[next]
                    - scratch.potential[edge.to];
                if candidate < scratch.distance[edge.to] {
                    scratch.distance[edge.to] = candidate;
                    scratch.previous[edge.to] = Some((next, edge_index));
                }
            }
        }
        if !scratch.used[sink] {
            return Err(EmdError::DisconnectedTransportGraph);
        }
        for v in 0..scratch.graph.len() {
            if scratch.used[v] {
                scratch.potential[v] += scratch.distance[v];
            }
        }
        reconstruct_path_into(
            &scratch.previous,
            source,
            sink,
            &mut scratch.path,
            &mut scratch.seen,
        )?;
        let mut amount = f64::INFINITY;
        for &(from, edge) in &scratch.path {
            amount = amount.min(scratch.graph[from][edge].capacity);
        }
        if !amount.is_finite() || amount <= 0.0 {
            return Err(EmdError::InvalidAugmentation);
        }
        let mut residual_capacity_changed = false;
        let mut node = sink;
        for &(from, edge) in &scratch.path {
            let reverse = scratch.graph[from][edge].reverse;
            total_cost += amount * scratch.graph[from][edge].cost;
            let forward_capacity_before = scratch.graph[from][edge].capacity;
            let reverse_capacity_before = scratch.graph[node][reverse].capacity;
            scratch.graph[from][edge].capacity -= amount;
            scratch.graph[node][reverse].capacity += amount;
            residual_capacity_changed |= scratch.graph[from][edge].capacity
                != forward_capacity_before
                || scratch.graph[node][reverse].capacity != reverse_capacity_before;
            node = from;
        }
        delivered += amount;
        if !delivered.is_finite() || !residual_capacity_changed {
            return Err(EmdError::NoProgress);
        }
    }
    if !complete {
        let (source_residual, demand_residual) =
            transport_residuals(&scratch.graph, source, sink, dimension);
        if source_residual <= completion_tolerance && demand_residual <= completion_tolerance {
            complete = true;
        }
    }
    if !complete {
        return Err(EmdError::IterationLimit);
    }
    let (source_residual, demand_residual) =
        transport_residuals(&scratch.graph, source, sink, dimension);
    if source_residual > completion_tolerance || demand_residual > completion_tolerance {
        return Err(EmdError::IncompleteTransport);
    }
    let delivered_by_residual = transport_mass - source_residual;
    if !delivered_by_residual.is_finite()
        || (transport_mass - delivered_by_residual).abs() > completion_tolerance
    {
        return Err(EmdError::IncompleteTransport);
    }
    if !total_cost.is_finite() {
        return Err(EmdError::NonFiniteResult);
    }
    let result = total_cost as f32;
    if !result.is_finite() {
        return Err(EmdError::NonFiniteResult);
    }
    Ok(result)
}

/// Совместимая обёртка для существующих вызовов без контекста задачи.
pub fn emd(left: &[f32], right: &[f32], ground: &[f32], dimension: usize) -> f32 {
    try_emd(left, right, ground, dimension).unwrap_or_else(|error| panic!("EMD failed: {error:?}"))
}

/// Точная Wasserstein-1 для распределений на одномерных equity-координатах.
///
/// Массы нормируются так же, как в [`emd`]. Порядок компонент входа может не
/// совпадать с порядком координат; равные координаты не добавляют стоимость.
pub fn wasserstein_1d(left: &[f32], right: &[f32], coordinates: &[f32]) -> f32 {
    assert!(left.len() == right.len() && left.len() == coordinates.len());
    assert!(coordinates.iter().all(|coordinate| coordinate.is_finite()));
    let left_mass: f64 = left.iter().map(|&value| value as f64).sum();
    let right_mass: f64 = right.iter().map(|&value| value as f64).sum();
    assert!(
        left_mass > 0.0 && right_mass > 0.0,
        "Wasserstein-1 requires positive mass"
    );

    let mut order: Vec<usize> = (0..coordinates.len()).collect();
    order.sort_unstable_by(|&left_index, &right_index| {
        coordinates[left_index]
            .partial_cmp(&coordinates[right_index])
            .unwrap()
    });

    let mut imbalance = 0.0f64;
    let mut cost = 0.0f64;
    for pair in order.windows(2) {
        let index = pair[0];
        imbalance += left[index] as f64 / left_mass - right[index] as f64 / right_mass;
        let interval = (coordinates[pair[1]] - coordinates[index]) as f64;
        cost += imbalance.abs() * interval;
    }
    cost as f32
}

pub struct Clustering {
    pub assignment: Vec<u16>,
    pub centroids: Vec<f32>,
    pub objective: Vec<f64>,
    pub diagnostics: Vec<ClusteringIteration>,
    /// Проверка согласованности возвращённых назначений с финальными центрами.
    /// Заполняется только по явному запросу, потому что требует ещё одного
    /// полного прохода assignment.
    pub final_assignment_diagnostic: Option<FinalAssignmentDiagnostic>,
}

/// Результат EMD-кластеризации с необязательными измерениями стоимости.
pub struct ProfiledEmdClustering {
    pub clustering: Clustering,
    pub profile: Option<EmdClusteringProfile>,
}

/// Измерения одной итерации EMD-кластеризации.
pub struct EmdIterationProfile {
    pub assignment_wall_clock: Duration,
    pub assignment_emd_calls: usize,
    pub centroid_update_wall_clock: Duration,
    pub objective_after_update_wall_clock: Duration,
    pub objective_after_update_emd_calls: usize,
}

/// Опциональный профиль точной EMD-кластеризации.
pub struct EmdClusteringProfile {
    pub wall_clock: Duration,
    pub iterations: Vec<EmdIterationProfile>,
    pub final_assignment_diagnostic_wall_clock: Option<Duration>,
    pub final_assignment_diagnostic_emd_calls: Option<usize>,
}

/// Результат проверки финальных назначений без изменения сохраняемых таблиц.
pub struct FinalAssignmentDiagnostic {
    pub changed_weight_fraction: f64,
    pub objective: f64,
}

/// Состояние одной итерации EMD-кластеризации.
pub struct ClusteringIteration {
    pub objective_before_update: f64,
    pub objective_after_update: f64,
    /// Цель после нового assignment к центрам предыдущего update.
    /// Для последней итерации отсутствует, пока отдельная проверка не запрошена.
    pub objective_after_reassignment: Option<f64>,
    pub moved_weight_fraction: f64,
    pub occupied_buckets: usize,
}

/// Событие живого прогресса EMD-кластеризации.
#[derive(Debug, Clone, PartialEq)]
pub enum EmdProgressEvent {
    IterationStart {
        iteration: usize,
    },
    AssignmentProgress {
        iteration: usize,
        processed: usize,
        total: usize,
        emd_calls: usize,
    },
    IterationEnd {
        iteration: usize,
        objective_before_update: f64,
        objective_after_update: f64,
    },
}

pub type EmdProgressCallback = Arc<dyn Fn(EmdProgressEvent) + Send + Sync>;

#[derive(Clone, Copy)]
enum EmdAssignmentSelection {
    Streaming,
    #[cfg(test)]
    VecReference,
}

#[derive(Clone, Copy)]
enum EmdKernel {
    Scratch,
    #[cfg(test)]
    PreScratchReference,
}

fn emd_with_context_for_kernel(
    kernel: EmdKernel,
    phase: &'static str,
    point_index: usize,
    centroid_index: usize,
    left: &[f32],
    right: &[f32],
    ground: &[f32],
    dimension: usize,
    scratch: &mut EmdScratch,
) -> Result<f32, EmdContextError> {
    match kernel {
        EmdKernel::Scratch => emd_with_context_with_scratch(
            phase,
            point_index,
            centroid_index,
            left,
            right,
            ground,
            dimension,
            scratch,
        ),
        #[cfg(test)]
        EmdKernel::PreScratchReference => {
            try_emd_pre_scratch_reference(left, right, ground, dimension).map_err(|source| {
                EmdContextError {
                    phase,
                    point_index,
                    centroid_index,
                    left: left.to_vec(),
                    right: right.to_vec(),
                    ground: ground.to_vec(),
                    source,
                }
            })
        }
    }
}

/// Weighted k-means under L1 between rows (earth mover's distance for CDFs),
/// with k-means++ seeding on a weighted sample and mean centroid updates.
pub fn kmeans(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    k: usize,
    seed: u64,
    max_iterations: usize,
) -> Clustering {
    let n = weight.len();
    assert!(points.len() == n * dim && k <= n && k <= u16::MAX as usize);
    let row = |i: usize| &points[i * dim..(i + 1) * dim];
    let mut rng = Rng(seed);
    let total: f64 = weight.iter().map(|&w| w as f64).sum();
    // Weighted sample of candidates for seeding.
    let cumulative: Vec<f64> = weight
        .iter()
        .scan(0f64, |acc, &w| {
            *acc += w as f64;
            Some(*acc)
        })
        .collect();
    let draw = |rng: &mut Rng| -> usize {
        let target = rng.unit() * total;
        cumulative.partition_point(|&c| c <= target).min(n - 1)
    };
    let sample: Vec<usize> = (0..n.min(200_000)).map(|_| draw(&mut rng)).collect();
    let mut centroids: Vec<f32> = row(sample[0]).to_vec();
    let mut nearest: Vec<f32> = sample
        .iter()
        .map(|&i| l1(row(i), &centroids[0..dim]))
        .collect();
    while centroids.len() < k * dim {
        let sum: f64 = nearest.iter().map(|&d| d as f64).sum();
        let pick = if sum > 0.0 {
            let target = rng.unit() * sum;
            let mut acc = 0f64;
            let mut chosen = sample.len() - 1;
            for (s, &d) in nearest.iter().enumerate() {
                acc += d as f64;
                if acc > target {
                    chosen = s;
                    break;
                }
            }
            chosen
        } else {
            (rng.next() % sample.len() as u64) as usize
        };
        let start = centroids.len();
        centroids.extend_from_slice(row(sample[pick]));
        let fresh = centroids[start..start + dim].to_vec();
        nearest
            .par_iter_mut()
            .zip(&sample)
            .for_each(|(d, &i)| *d = d.min(l1(row(i), &fresh)));
    }
    let mut assignment = vec![u16::MAX; n];
    let mut objective = Vec::new();
    for _ in 0..max_iterations {
        let assigned: Vec<(u16, f32)> = (0..n)
            .into_par_iter()
            .map(|i| {
                let p = row(i);
                let mut best = (0u16, f32::INFINITY);
                for c in 0..k {
                    let d = l1(p, &centroids[c * dim..(c + 1) * dim]);
                    if d < best.1 {
                        best = (c as u16, d);
                    }
                }
                best
            })
            .collect();
        let moved: f64 = assigned
            .iter()
            .zip(&assignment)
            .zip(weight)
            .filter(|((a, old), _)| a.0 != **old)
            .map(|(_, &w)| w as f64)
            .sum();
        objective.push(
            assigned
                .iter()
                .zip(weight)
                .map(|(a, &w)| a.1 as f64 * w as f64)
                .sum::<f64>()
                / total,
        );
        for (slot, a) in assignment.iter_mut().zip(&assigned) {
            *slot = a.0;
        }
        let mut sums = vec![0f64; k * dim];
        let mut mass = vec![0f64; k];
        for i in 0..n {
            let c = assignment[i] as usize;
            let w = weight[i] as f64;
            mass[c] += w;
            for (s, &x) in sums[c * dim..(c + 1) * dim].iter_mut().zip(row(i)) {
                *s += w * x as f64;
            }
        }
        for c in 0..k {
            if mass[c] > 0.0 {
                for d in 0..dim {
                    centroids[c * dim + d] = (sums[c * dim + d] / mass[c]) as f32;
                }
            }
        }
        if moved / total < 1e-4 {
            break;
        }
    }
    Clustering {
        assignment,
        centroids,
        objective,
        diagnostics: Vec::new(),
        final_assignment_diagnostic: None,
    }
}

/// Взвешенный k-means для распределений с EMD и заданной ground matrix.
pub fn kmeans_emd(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<Clustering, EmdContextError> {
    kmeans_emd_with_progress(points, dim, weight, ground, k, seed, max_iterations, None)
}

/// Взвешенный k-means с живыми событиями начала итерации, assignment и её завершения.
pub fn kmeans_emd_with_progress(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    progress: Option<EmdProgressCallback>,
) -> Result<Clustering, EmdContextError> {
    kmeans_emd_with_diagnostics(
        points,
        dim,
        weight,
        ground,
        k,
        seed,
        max_iterations,
        progress,
        false,
    )
}

/// Взвешенный k-means с необязательной проверкой финальных назначений.
/// Проверка не меняет ни assignment, ни центроиды, возвращаемые кластеризацией.
pub fn kmeans_emd_with_diagnostics(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    progress: Option<EmdProgressCallback>,
    diagnose_final_assignment: bool,
) -> Result<Clustering, EmdContextError> {
    Ok(kmeans_emd_profiled(
        points,
        dim,
        weight,
        ground,
        k,
        seed,
        max_iterations,
        progress,
        diagnose_final_assignment,
        false,
    )?
    .clustering)
}

/// Взвешенный k-means с opt-in измерениями стоимости точных EMD-фаз.
pub fn kmeans_emd_profiled(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    progress: Option<EmdProgressCallback>,
    diagnose_final_assignment: bool,
    profile_enabled: bool,
) -> Result<ProfiledEmdClustering, EmdContextError> {
    kmeans_emd_profiled_with_assignment(
        points,
        dim,
        weight,
        ground,
        k,
        seed,
        max_iterations,
        progress,
        diagnose_final_assignment,
        profile_enabled,
        EmdAssignmentSelection::Streaming,
        EmdKernel::Scratch,
    )
}

fn kmeans_emd_profiled_with_assignment(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    progress: Option<EmdProgressCallback>,
    diagnose_final_assignment: bool,
    profile_enabled: bool,
    assignment_selection: EmdAssignmentSelection,
    emd_kernel: EmdKernel,
) -> Result<ProfiledEmdClustering, EmdContextError> {
    let clustering_started = profile_enabled.then(Instant::now);
    let n = weight.len();
    assert!(
        points.len() == n * dim && ground.len() == dim * dim && k <= n && k <= u16::MAX as usize
    );
    let row = |index: usize| &points[index * dim..(index + 1) * dim];
    let mut rng = Rng(seed);
    let mut centroids = Vec::with_capacity(k * dim);
    for _ in 0..k {
        let index = (rng.next() % n as u64) as usize;
        centroids.extend_from_slice(row(index));
    }
    let total: f64 = weight.iter().map(|&value| value as f64).sum();
    let mut assignment = vec![u16::MAX; n];
    let mut objective = Vec::new();
    let mut diagnostics: Vec<ClusteringIteration> = Vec::new();
    let mut profile_iterations = Vec::new();
    for iteration in 1..=max_iterations {
        if let Some(callback) = &progress {
            callback(EmdProgressEvent::IterationStart { iteration });
        }
        let processed = Arc::new(AtomicUsize::new(0));
        let assignment_progress = progress.clone();
        let assignment_started = profile_enabled.then(Instant::now);
        let assigned: Vec<(u16, f32, usize)> = (0..n)
            .into_par_iter()
            .map_init(EmdScratch::default, |scratch, index| {
                let point = row(index);
                let mut emd_calls = 0usize;
                let candidates = (0..k).map(|cluster| {
                    let centroid = &centroids[cluster * dim..(cluster + 1) * dim];
                    emd_calls += 1;
                    emd_with_context_for_kernel(
                        emd_kernel,
                        "assignment",
                        index,
                        cluster,
                        point,
                        centroid,
                        ground,
                        dim,
                        scratch,
                    )
                    .map(|distance| (cluster as u16, distance))
                });
                let result = match assignment_selection {
                    EmdAssignmentSelection::Streaming => first_minimum_emd(candidates),
                    #[cfg(test)]
                    EmdAssignmentSelection::VecReference => candidates
                        .collect::<Result<Vec<_>, _>>()
                        .map(first_minimum_emd_reference),
                };
                let completed = processed.fetch_add(1, Ordering::Relaxed) + 1;
                if (completed == n || completed % 4096 == 0) && result.is_ok() {
                    if let Some(callback) = &assignment_progress {
                        callback(EmdProgressEvent::AssignmentProgress {
                            iteration,
                            processed: completed,
                            total: n,
                            emd_calls: completed * k,
                        });
                    }
                }
                result.map(|(cluster, distance)| (cluster, distance, emd_calls))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assignment_emd_calls = assigned.iter().map(|entry| entry.2).sum();
        let assignment_wall_clock = assignment_started.map(|started| started.elapsed());
        let moved: f64 = assigned
            .iter()
            .zip(&assignment)
            .zip(weight)
            .filter(|((next, old), _)| next.0 != **old)
            .map(|(_, &value)| value as f64)
            .sum();
        let objective_before_update = assigned
            .iter()
            .zip(weight)
            .map(|(entry, &value)| entry.1 as f64 * value as f64)
            .sum::<f64>()
            / total;
        if let Some(previous) = diagnostics.last_mut() {
            previous.objective_after_reassignment = Some(objective_before_update);
        }
        objective.push(objective_before_update);
        let centroid_update_started = profile_enabled.then(Instant::now);
        for (slot, entry) in assignment.iter_mut().zip(&assigned) {
            *slot = entry.0;
        }
        let mut sums = vec![0.0f64; k * dim];
        let mut mass = vec![0.0f64; k];
        for index in 0..n {
            let cluster = assignment[index] as usize;
            let row_weight = weight[index] as f64;
            mass[cluster] += row_weight;
            for (sum, &value) in sums[cluster * dim..(cluster + 1) * dim]
                .iter_mut()
                .zip(row(index))
            {
                *sum += row_weight * value as f64;
            }
        }
        for cluster in 0..k {
            if mass[cluster] > 0.0 {
                for offset in 0..dim {
                    centroids[cluster * dim + offset] =
                        (sums[cluster * dim + offset] / mass[cluster]) as f32;
                }
            }
        }
        let centroid_update_wall_clock = centroid_update_started.map(|started| started.elapsed());
        let objective_after_update_started = profile_enabled.then(Instant::now);
        let mut objective_after_update_scratch = EmdScratch::default();
        let objective_after_update_entries = (0..n)
            .map(|index| {
                let cluster = assignment[index] as usize;
                emd_with_context_for_kernel(
                    emd_kernel,
                    "objective-after-update",
                    index,
                    cluster,
                    row(index),
                    &centroids[cluster * dim..(cluster + 1) * dim],
                    ground,
                    dim,
                    &mut objective_after_update_scratch,
                )
                .map(|distance| (distance as f64 * weight[index] as f64, 1usize))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let objective_after_update_emd_calls = objective_after_update_entries
            .iter()
            .map(|entry| entry.1)
            .sum();
        let objective_after_update = objective_after_update_entries
            .into_iter()
            .map(|entry| entry.0)
            .sum::<f64>()
            / total;
        let objective_after_update_wall_clock =
            objective_after_update_started.map(|started| started.elapsed());
        let diagnostic = ClusteringIteration {
            objective_before_update,
            objective_after_update,
            objective_after_reassignment: None,
            moved_weight_fraction: moved / total,
            occupied_buckets: mass.iter().filter(|&&value| value > 0.0).count(),
        };
        if let Some(callback) = &progress {
            callback(EmdProgressEvent::IterationEnd {
                iteration,
                objective_before_update,
                objective_after_update,
            });
        }
        diagnostics.push(diagnostic);
        if let (
            Some(assignment_wall_clock),
            Some(centroid_update_wall_clock),
            Some(objective_after_update_wall_clock),
        ) = (
            assignment_wall_clock,
            centroid_update_wall_clock,
            objective_after_update_wall_clock,
        ) {
            profile_iterations.push(EmdIterationProfile {
                assignment_wall_clock,
                assignment_emd_calls,
                centroid_update_wall_clock,
                objective_after_update_wall_clock,
                objective_after_update_emd_calls,
            });
        }
        if moved / total < 1e-4 {
            break;
        }
    }
    let final_assignment_started = profile_enabled.then(Instant::now);
    let (final_assignment_diagnostic, final_assignment_diagnostic_profile) =
        if diagnose_final_assignment {
            let reassigned: Vec<(u16, f32, usize)> = (0..n)
                .into_par_iter()
                .map_init(EmdScratch::default, |scratch, index| {
                    let point = row(index);
                    let mut emd_calls = 0usize;
                    let distances: Result<Vec<(u16, f32)>, EmdContextError> = (0..k)
                        .map(|cluster| {
                            let centroid = &centroids[cluster * dim..(cluster + 1) * dim];
                            emd_calls += 1;
                            emd_with_context_for_kernel(
                                emd_kernel,
                                "final-assignment-diagnostic",
                                index,
                                cluster,
                                point,
                                centroid,
                                ground,
                                dim,
                                scratch,
                            )
                            .map(|distance| (cluster as u16, distance))
                        })
                        .collect();
                    distances
                        .map(|distances| {
                            distances
                                .into_iter()
                                .min_by(|left, right| left.1.partial_cmp(&right.1).unwrap())
                                .unwrap()
                        })
                        .map(|(cluster, distance)| (cluster, distance, emd_calls))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let changed_weight_fraction = reassigned
                .iter()
                .zip(&assignment)
                .zip(weight)
                .filter(|((next, current), _)| next.0 != **current)
                .map(|(_, &value)| value as f64)
                .sum::<f64>()
                / total;
            let objective = reassigned
                .iter()
                .zip(weight)
                .map(|(entry, &value)| entry.1 as f64 * value as f64)
                .sum::<f64>()
                / total;
            let profile = final_assignment_started.map(|started| {
                (
                    started.elapsed(),
                    reassigned.iter().map(|entry| entry.2).sum(),
                )
            });
            (
                Some(FinalAssignmentDiagnostic {
                    changed_weight_fraction,
                    objective,
                }),
                profile,
            )
        } else {
            (None, None)
        };
    Ok(ProfiledEmdClustering {
        clustering: Clustering {
            assignment,
            centroids,
            objective,
            diagnostics,
            final_assignment_diagnostic,
        },
        profile: clustering_started.map(|started| EmdClusteringProfile {
            wall_clock: started.elapsed(),
            iterations: profile_iterations,
            final_assignment_diagnostic_wall_clock: final_assignment_diagnostic_profile
                .map(|profile| profile.0),
            final_assignment_diagnostic_emd_calls: final_assignment_diagnostic_profile
                .map(|profile| profile.1),
        }),
    })
}

#[cfg(test)]
fn kmeans_emd_with_reference_assignment(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    diagnose_final_assignment: bool,
) -> Result<Clustering, EmdContextError> {
    Ok(kmeans_emd_profiled_with_assignment(
        points,
        dim,
        weight,
        ground,
        k,
        seed,
        max_iterations,
        None,
        diagnose_final_assignment,
        false,
        EmdAssignmentSelection::VecReference,
        EmdKernel::Scratch,
    )?
    .clustering)
}

#[cfg(test)]
fn kmeans_emd_with_pre_scratch_reference(
    points: &[f32],
    dim: usize,
    weight: &[u64],
    ground: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    diagnose_final_assignment: bool,
) -> Result<Clustering, EmdContextError> {
    Ok(kmeans_emd_profiled_with_assignment(
        points,
        dim,
        weight,
        ground,
        k,
        seed,
        max_iterations,
        None,
        diagnose_final_assignment,
        false,
        EmdAssignmentSelection::Streaming,
        EmdKernel::PreScratchReference,
    )?
    .clustering)
}

/// EMD-кластеризация для river ground вида `|equity_i - equity_j|`.
/// Координаты сортируются ровно один раз, а центроиды перед возвратом
/// восстанавливаются в исходном порядке компонент.
pub fn kmeans_wasserstein_1d(
    points: &[f32],
    weight: &[u64],
    coordinates: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
) -> Clustering {
    kmeans_wasserstein_1d_with_diagnostics(
        points,
        weight,
        coordinates,
        k,
        seed,
        max_iterations,
        false,
    )
}

/// Одномерная EMD-кластеризация с необязательной проверкой финальных назначений.
pub fn kmeans_wasserstein_1d_with_diagnostics(
    points: &[f32],
    weight: &[u64],
    coordinates: &[f32],
    k: usize,
    seed: u64,
    max_iterations: usize,
    diagnose_final_assignment: bool,
) -> Clustering {
    let dimension = coordinates.len();
    let n = weight.len();
    assert!(points.len() == n * dimension && k <= n && k <= u16::MAX as usize);
    let mut order: Vec<usize> = (0..dimension).collect();
    order.sort_unstable_by(|&left, &right| {
        coordinates[left].partial_cmp(&coordinates[right]).unwrap()
    });
    let sorted_coordinates: Vec<f32> = order.iter().map(|&index| coordinates[index]).collect();
    let mut sorted_points = vec![0.0; points.len()];
    for point in 0..n {
        for (sorted_index, &original_index) in order.iter().enumerate() {
            sorted_points[point * dimension + sorted_index] =
                points[point * dimension + original_index];
        }
    }
    let row = |index: usize| &sorted_points[index * dimension..(index + 1) * dimension];
    let mut rng = Rng(seed);
    let mut centroids = Vec::with_capacity(k * dimension);
    for _ in 0..k {
        centroids.extend_from_slice(row((rng.next() % n as u64) as usize));
    }
    let total: f64 = weight.iter().map(|&value| value as f64).sum();
    let mut assignment = vec![u16::MAX; n];
    let mut objective = Vec::new();
    let mut diagnostics = Vec::new();
    for _ in 0..max_iterations {
        let assigned: Vec<(u16, f32)> = (0..n)
            .into_par_iter()
            .map(|index| {
                let point = row(index);
                (0..k)
                    .map(|cluster| {
                        (
                            cluster as u16,
                            wasserstein_1d_sorted(
                                point,
                                &centroids[cluster * dimension..(cluster + 1) * dimension],
                                &sorted_coordinates,
                            ),
                        )
                    })
                    .min_by(|left, right| left.1.partial_cmp(&right.1).unwrap())
                    .unwrap()
            })
            .collect();
        let moved: f64 = assigned
            .iter()
            .zip(&assignment)
            .zip(weight)
            .filter(|((next, old), _)| next.0 != **old)
            .map(|(_, &value)| value as f64)
            .sum();
        let objective_before_update = assigned
            .iter()
            .zip(weight)
            .map(|(entry, &value)| entry.1 as f64 * value as f64)
            .sum::<f64>()
            / total;
        objective.push(objective_before_update);
        for (slot, entry) in assignment.iter_mut().zip(&assigned) {
            *slot = entry.0;
        }
        let mut sums = vec![0.0f64; k * dimension];
        let mut mass = vec![0.0f64; k];
        for index in 0..n {
            let cluster = assignment[index] as usize;
            let row_weight = weight[index] as f64;
            mass[cluster] += row_weight;
            for (sum, &value) in sums[cluster * dimension..(cluster + 1) * dimension]
                .iter_mut()
                .zip(row(index))
            {
                *sum += row_weight * value as f64;
            }
        }
        for cluster in 0..k {
            if mass[cluster] > 0.0 {
                for offset in 0..dimension {
                    centroids[cluster * dimension + offset] =
                        (sums[cluster * dimension + offset] / mass[cluster]) as f32;
                }
            }
        }
        let objective_after_update = (0..n)
            .map(|index| {
                let cluster = assignment[index] as usize;
                wasserstein_1d_sorted(
                    row(index),
                    &centroids[cluster * dimension..(cluster + 1) * dimension],
                    &sorted_coordinates,
                ) as f64
                    * weight[index] as f64
            })
            .sum::<f64>()
            / total;
        diagnostics.push(ClusteringIteration {
            objective_before_update,
            objective_after_update,
            objective_after_reassignment: None,
            moved_weight_fraction: moved / total,
            occupied_buckets: mass.iter().filter(|&&value| value > 0.0).count(),
        });
        if moved / total < 1e-4 {
            break;
        }
    }
    let mut original_centroids = vec![0.0; k * dimension];
    for cluster in 0..k {
        for (sorted_index, &original_index) in order.iter().enumerate() {
            original_centroids[cluster * dimension + original_index] =
                centroids[cluster * dimension + sorted_index];
        }
    }
    let final_assignment_diagnostic = if diagnose_final_assignment {
        let reassigned: Vec<(u16, f32)> = (0..n)
            .into_par_iter()
            .map(|index| {
                let point = row(index);
                (0..k)
                    .map(|cluster| {
                        (
                            cluster as u16,
                            wasserstein_1d_sorted(
                                point,
                                &centroids[cluster * dimension..(cluster + 1) * dimension],
                                &sorted_coordinates,
                            ),
                        )
                    })
                    .min_by(|left, right| left.1.partial_cmp(&right.1).unwrap())
                    .unwrap()
            })
            .collect();
        let changed_weight_fraction = reassigned
            .iter()
            .zip(&assignment)
            .zip(weight)
            .filter(|((next, current), _)| next.0 != **current)
            .map(|(_, &value)| value as f64)
            .sum::<f64>()
            / total;
        let objective = reassigned
            .iter()
            .zip(weight)
            .map(|(entry, &value)| entry.1 as f64 * value as f64)
            .sum::<f64>()
            / total;
        Some(FinalAssignmentDiagnostic {
            changed_weight_fraction,
            objective,
        })
    } else {
        None
    };
    Clustering {
        assignment,
        centroids: original_centroids,
        objective,
        diagnostics,
        final_assignment_diagnostic,
    }
}

fn wasserstein_1d_sorted(left: &[f32], right: &[f32], coordinates: &[f32]) -> f32 {
    let left_mass: f64 = left.iter().map(|&value| value as f64).sum();
    let right_mass: f64 = right.iter().map(|&value| value as f64).sum();
    let mut imbalance = 0.0f64;
    let mut cost = 0.0f64;
    for index in 0..coordinates.len().saturating_sub(1) {
        imbalance += left[index] as f64 / left_mass - right[index] as f64 / right_mass;
        cost += imbalance.abs() * (coordinates[index + 1] - coordinates[index]) as f64;
    }
    cost as f32
}

/// Матрица ground distances между центроидами улицы для предыдущей улицы.
pub fn centroid_emd_matrix(
    centroids: &[f32],
    dimension: usize,
    lower_ground: &[f32],
) -> Result<Vec<f32>, EmdContextError> {
    Ok(centroid_emd_matrix_profiled(centroids, dimension, lower_ground)?.matrix)
}

/// Матрица ground distances с числом фактически выполненных EMD-сравнений.
pub struct ProfiledCentroidEmdMatrix {
    pub matrix: Vec<f32>,
    pub emd_calls: usize,
}

pub fn centroid_emd_matrix_profiled(
    centroids: &[f32],
    dimension: usize,
    lower_ground: &[f32],
) -> Result<ProfiledCentroidEmdMatrix, EmdContextError> {
    assert!(centroids.len() % dimension == 0 && lower_ground.len() == dimension * dimension);
    let count = centroids.len() / dimension;
    let mut matrix = vec![0.0; count * count];
    let mut emd_calls = 0usize;
    for left in 0..count {
        for right in left + 1..count {
            emd_calls += 1;
            let distance = emd_with_context(
                "centroid-ground-matrix",
                left,
                right,
                &centroids[left * dimension..(left + 1) * dimension],
                &centroids[right * dimension..(right + 1) * dimension],
                lower_ground,
                dimension,
            )?;
            matrix[left * count + right] = distance;
            matrix[right * count + left] = distance;
        }
    }
    Ok(ProfiledCentroidEmdMatrix { matrix, emd_calls })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(text: &str) -> Vec<u8> {
        text.split_whitespace().map(parse_card).collect()
    }

    fn ranks5_disconnected_transport_fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let left = vec![
            0.13333334, 0.0, 0.06666667, 0.0, 0.0, 0.0, 0.4, 0.0, 0.26666668, 0.13333334,
        ];
        let right = vec![
            0.13333334, 0.0, 0.0, 0.0, 0.0, 0.0, 0.46666667, 0.0, 0.26666668, 0.13333334,
        ];
        let ground = vec![
            0.0,
            0.1547003,
            0.26344058,
            0.37948424,
            0.121828094,
            0.19579911,
            0.22736175,
            0.3203327,
            0.23452008,
            0.29428524,
            0.1547003,
            0.0,
            0.13274729,
            0.24522302,
            0.2564203,
            0.33043665,
            0.24524431,
            0.23298159,
            0.20729323,
            0.42887747,
            0.26344058,
            0.13274729,
            0.0,
            0.11604366,
            0.38526866,
            0.4591352,
            0.16985136,
            0.1178549,
            0.12155493,
            0.5577258,
            0.37948424,
            0.24522302,
            0.11604366,
            0.0,
            0.5013123,
            0.57517886,
            0.21012262,
            0.120126605,
            0.15476401,
            0.6737695,
            0.121828094,
            0.2564203,
            0.38526866,
            0.5013123,
            0.0,
            0.074084334,
            0.30677053,
            0.4421608,
            0.3563482,
            0.17245714,
            0.19579911,
            0.33043665,
            0.4591352,
            0.57517886,
            0.074084334,
            0.0,
            0.38063708,
            0.51602733,
            0.43021473,
            0.09859059,
            0.22736175,
            0.24524431,
            0.16985136,
            0.21012262,
            0.30677053,
            0.38063708,
            0.0,
            0.13539027,
            0.0666598,
            0.47922766,
            0.3203327,
            0.23298159,
            0.1178549,
            0.120126605,
            0.4421608,
            0.51602733,
            0.13539027,
            0.0,
            0.08581262,
            0.61461794,
            0.23452008,
            0.20729323,
            0.12155493,
            0.15476401,
            0.3563482,
            0.43021473,
            0.0666598,
            0.08581262,
            0.0,
            0.5288053,
            0.29428524,
            0.42887747,
            0.5577258,
            0.6737695,
            0.17245714,
            0.09859059,
            0.47922766,
            0.61461794,
            0.5288053,
            0.0,
        ];
        (left, right, ground)
    }

    fn flop_no_progress_fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let left = vec![
            0.18181819, 0.0, 0.0, 0.72727275, 0.0, 0.0, 0.0, 0.09090909, 0.0, 0.0,
        ];
        let right = vec![
            0.0, 0.0, 0.0, 0.0, 0.54545456, 0.0, 0.27272728, 0.0, 0.18181819, 0.0,
        ];
        let ground = vec![
            0.0,
            0.13629213,
            0.08587584,
            0.27648434,
            0.25827435,
            0.014638996,
            0.058641795,
            0.7000237,
            0.4346938,
            0.181192,
            0.13629213,
            0.0,
            0.050416287,
            0.19532846,
            0.12198224,
            0.12348225,
            0.07901462,
            0.5637316,
            0.29840168,
            0.08497865,
            0.08587584,
            0.050416287,
            0.0,
            0.24455096,
            0.17239852,
            0.07306597,
            0.11995916,
            0.6141479,
            0.34881797,
            0.1295547,
            0.27648434,
            0.19532846,
            0.24455096,
            0.0,
            0.11568272,
            0.26367447,
            0.21920684,
            0.4235394,
            0.15820947,
            0.13901122,
            0.25827435,
            0.12198224,
            0.17239852,
            0.11568272,
            0.0,
            0.24546447,
            0.20099685,
            0.44174936,
            0.17641945,
            0.0819604,
            0.014638996,
            0.12348225,
            0.07306597,
            0.26367447,
            0.24546447,
            0.0,
            0.05392626,
            0.68721384,
            0.42188394,
            0.16838212,
            0.058641795,
            0.07901462,
            0.11995916,
            0.21920684,
            0.20099685,
            0.05392626,
            0.0,
            0.6427462,
            0.3774163,
            0.123914495,
            0.7000237,
            0.5637316,
            0.6141479,
            0.4235394,
            0.44174936,
            0.68721384,
            0.6427462,
            0.0,
            0.26532993,
            0.51883173,
            0.4346938,
            0.29840168,
            0.34881797,
            0.15820947,
            0.17641945,
            0.42188394,
            0.3774163,
            0.26532993,
            0.0,
            0.2535018,
            0.181192,
            0.08497865,
            0.1295547,
            0.13901122,
            0.0819604,
            0.16838212,
            0.123914495,
            0.51883173,
            0.2535018,
            0.0,
        ];
        (left, right, ground)
    }

    #[derive(Debug)]
    struct ReducedCostEdgeTrace {
        search_index: usize,
        from: usize,
        to: usize,
        capacity: f64,
        from_potential: f64,
        to_potential: f64,
        reduced_cost: f64,
        from_reachable_from_source: bool,
        from_is_relaxable: bool,
    }

    #[derive(Debug)]
    struct SearchInvariantTrace {
        search_index: usize,
        sink_reachable_from_source: bool,
        sink_outgoing_eligible_edges: usize,
        negative_eligible_reduced_cost_edges: Vec<ReducedCostEdgeTrace>,
    }

    #[derive(Debug)]
    struct EmdFailureTrace {
        transport_mass: f64,
        delivered: f64,
        augmentations: usize,
        reachable_supply: Vec<usize>,
        reachable_demand: Vec<usize>,
        source_residual: f64,
        demand_residual: f64,
        eligible_demand_edges: usize,
        minimum_reduced_cost: f64,
        first_negative_eligible_reduced_cost: Option<ReducedCostEdgeTrace>,
        search_invariants: Vec<SearchInvariantTrace>,
    }

    fn reachable_with_eligible_edges(
        graph: &[Vec<FlowEdge>],
        source: usize,
        sink: usize,
    ) -> Vec<bool> {
        let mut reachable = vec![false; graph.len()];
        let mut pending = std::collections::VecDeque::from([source]);
        reachable[source] = true;
        while let Some(from) = pending.pop_front() {
            if from == sink {
                continue;
            }
            for edge in &graph[from] {
                if edge.capacity > 1e-9 && !reachable[edge.to] {
                    reachable[edge.to] = true;
                    pending.push_back(edge.to);
                }
            }
        }
        reachable
    }

    fn trace_disconnected_transport(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
    ) -> EmdFailureTrace {
        let left_mass: f64 = left.iter().map(|&value| value as f64).sum();
        let right_mass: f64 = right.iter().map(|&value| value as f64).sum();
        let normalized_left: Vec<f64> =
            left.iter().map(|&value| value as f64 / left_mass).collect();
        let normalized_right: Vec<f64> = right
            .iter()
            .map(|&value| value as f64 / right_mass)
            .collect();
        let transport_mass = normalized_left
            .iter()
            .sum::<f64>()
            .min(normalized_right.iter().sum::<f64>());
        let source = 2 * dimension;
        let sink = source + 1;
        let mut graph = vec![Vec::new(); sink + 1];
        for index in 0..dimension {
            add_flow_edge(&mut graph, source, index, normalized_left[index], 0.0);
            add_flow_edge(
                &mut graph,
                dimension + index,
                sink,
                normalized_right[index],
                0.0,
            );
            for target in 0..dimension {
                add_flow_edge(
                    &mut graph,
                    index,
                    dimension + target,
                    1.0,
                    ground[index * dimension + target] as f64,
                );
            }
        }
        let mut potential = vec![0.0; graph.len()];
        let mut delivered = 0.0;
        let mut augmentations = 0usize;
        let mut search_invariants = Vec::new();
        loop {
            let reachable_before_search = reachable_with_eligible_edges(&graph, source, sink);
            let mut negative_eligible_reduced_cost_edges = Vec::new();
            for (from, edges) in graph.iter().enumerate() {
                for edge in edges {
                    let reduced_cost = edge.cost + potential[from] - potential[edge.to];
                    if edge.capacity > 1e-9 && reduced_cost < 0.0 {
                        negative_eligible_reduced_cost_edges.push(ReducedCostEdgeTrace {
                            search_index: augmentations,
                            from,
                            to: edge.to,
                            capacity: edge.capacity,
                            from_potential: potential[from],
                            to_potential: potential[edge.to],
                            reduced_cost,
                            from_reachable_from_source: reachable_before_search[from],
                            from_is_relaxable: reachable_before_search[from] && from != sink,
                        });
                    }
                }
            }
            let sink_outgoing_eligible_edges = graph[sink]
                .iter()
                .filter(|edge| edge.capacity > 1e-9)
                .count();
            search_invariants.push(SearchInvariantTrace {
                search_index: augmentations,
                sink_reachable_from_source: reachable_before_search[sink],
                sink_outgoing_eligible_edges,
                negative_eligible_reduced_cost_edges,
            });
            let mut distance = vec![f64::INFINITY; graph.len()];
            let mut previous = vec![None; graph.len()];
            let mut used = vec![false; graph.len()];
            distance[source] = 0.0;
            for _ in 0..graph.len() {
                let next = (0..graph.len())
                    .filter(|&vertex| !used[vertex])
                    .min_by(|&left, &right| distance[left].partial_cmp(&distance[right]).unwrap())
                    .unwrap();
                if !distance[next].is_finite() {
                    break;
                }
                used[next] = true;
                if next == sink {
                    continue;
                }
                for (edge_index, edge) in graph[next].iter().enumerate() {
                    if edge.capacity <= 1e-9 || used[edge.to] {
                        continue;
                    }
                    let candidate =
                        distance[next] + edge.cost + potential[next] - potential[edge.to];
                    if candidate < distance[edge.to] {
                        distance[edge.to] = candidate;
                        previous[edge.to] = Some((next, edge_index));
                    }
                }
            }
            if !used[sink] {
                let mut minimum_reduced_cost = f64::INFINITY;
                for (from, edges) in graph.iter().enumerate() {
                    for edge in edges {
                        if edge.capacity > 1e-9 {
                            minimum_reduced_cost = minimum_reduced_cost
                                .min(edge.cost + potential[from] - potential[edge.to]);
                        }
                    }
                }
                let demand_residuals: Vec<f64> = (0..dimension)
                    .map(|index| {
                        graph[dimension + index]
                            .iter()
                            .find(|edge| edge.to == sink)
                            .expect("demand node must connect to sink")
                            .capacity
                    })
                    .collect();
                return EmdFailureTrace {
                    transport_mass,
                    delivered,
                    augmentations,
                    reachable_supply: (0..dimension).filter(|&index| used[index]).collect(),
                    reachable_demand: (0..dimension)
                        .filter(|&index| used[dimension + index])
                        .collect(),
                    source_residual: graph[source]
                        .iter()
                        .map(|edge| edge.capacity)
                        .filter(|&capacity| capacity > 1e-9)
                        .sum(),
                    demand_residual: (0..dimension).map(|index| demand_residuals[index]).sum(),
                    eligible_demand_edges: demand_residuals
                        .iter()
                        .filter(|&&capacity| capacity > 1e-9)
                        .count(),
                    minimum_reduced_cost,
                    first_negative_eligible_reduced_cost: search_invariants
                        .iter()
                        .flat_map(|search| &search.negative_eligible_reduced_cost_edges)
                        .next()
                        .map(|edge| ReducedCostEdgeTrace {
                            search_index: edge.search_index,
                            from: edge.from,
                            to: edge.to,
                            capacity: edge.capacity,
                            from_potential: edge.from_potential,
                            to_potential: edge.to_potential,
                            reduced_cost: edge.reduced_cost,
                            from_reachable_from_source: edge.from_reachable_from_source,
                            from_is_relaxable: edge.from_is_relaxable,
                        }),
                    search_invariants,
                };
            }
            for vertex in 0..graph.len() {
                if used[vertex] {
                    potential[vertex] += distance[vertex];
                }
            }
            let mut amount = transport_mass - delivered;
            let path = reconstruct_path(&previous, source, sink).expect("reachable sink has path");
            for &(from, edge) in &path {
                amount = amount.min(graph[from][edge].capacity);
            }
            let mut node = sink;
            for &(from, edge) in &path {
                let reverse = graph[from][edge].reverse;
                graph[from][edge].capacity -= amount;
                graph[node][reverse].capacity += amount;
                node = from;
            }
            delivered += amount;
            augmentations += 1;
        }
    }

    fn independent_transport_cost(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
    ) -> f64 {
        const TOLERANCE: f64 = 1e-12;

        fn solve_linear_system(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
            for pivot_column in 0..rhs.len() {
                let pivot_row = (pivot_column..rhs.len()).max_by(|&left, &right| {
                    matrix[left][pivot_column]
                        .abs()
                        .partial_cmp(&matrix[right][pivot_column].abs())
                        .unwrap()
                })?;
                if matrix[pivot_row][pivot_column].abs() <= TOLERANCE {
                    return None;
                }
                matrix.swap(pivot_column, pivot_row);
                rhs.swap(pivot_column, pivot_row);
                let pivot = matrix[pivot_column][pivot_column];
                for column in pivot_column..rhs.len() {
                    matrix[pivot_column][column] /= pivot;
                }
                rhs[pivot_column] /= pivot;
                for row in 0..rhs.len() {
                    if row == pivot_column {
                        continue;
                    }
                    let factor = matrix[row][pivot_column];
                    for column in pivot_column..rhs.len() {
                        matrix[row][column] -= factor * matrix[pivot_column][column];
                    }
                    rhs[row] -= factor * rhs[pivot_column];
                }
            }
            Some(rhs)
        }

        fn evaluate_basis(
            selected: &[usize],
            edges: &[(usize, usize)],
            sources: &[usize],
            targets: &[usize],
            supply: &[f64],
            demand: &[f64],
            ground: &[f32],
            dimension: usize,
        ) -> Option<f64> {
            let equations = selected.len();
            let mut matrix = vec![vec![0.0; equations]; equations];
            let mut rhs = Vec::with_capacity(equations);
            for (row, &mass) in supply.iter().enumerate() {
                for (column, &edge) in selected.iter().enumerate() {
                    matrix[row][column] = (edges[edge].0 == sources[row]) as u8 as f64;
                }
                rhs.push(mass);
            }
            for (offset, &mass) in demand.iter().take(demand.len() - 1).enumerate() {
                let row = supply.len() + offset;
                for (column, &edge) in selected.iter().enumerate() {
                    matrix[row][column] = (edges[edge].1 == targets[offset]) as u8 as f64;
                }
                rhs.push(mass);
            }
            let flow = solve_linear_system(matrix, rhs)?;
            if flow.iter().any(|&mass| mass < -TOLERANCE) {
                return None;
            }
            let mut delivered = vec![0.0; demand.len()];
            let mut cost = 0.0;
            for (&edge, &mass) in selected.iter().zip(&flow) {
                let (source, target) = edges[edge];
                let mass = mass.max(0.0);
                let compact_target = targets
                    .iter()
                    .position(|&candidate| candidate == target)
                    .expect("basis target must be present in the demand support");
                delivered[compact_target] += mass;
                cost += mass * ground[source * dimension + target] as f64;
            }
            (delivered
                .iter()
                .zip(demand)
                .all(|(&actual, &expected)| (actual - expected).abs() <= TOLERANCE))
            .then_some(cost)
        }

        fn enumerate_bases(
            edge_index: usize,
            selected: &mut Vec<usize>,
            edges: &[(usize, usize)],
            sources: &[usize],
            targets: &[usize],
            basis_size: usize,
            supply: &[f64],
            demand: &[f64],
            ground: &[f32],
            dimension: usize,
            best: &mut f64,
        ) {
            if selected.len() == basis_size {
                if let Some(cost) = evaluate_basis(
                    selected, edges, sources, targets, supply, demand, ground, dimension,
                ) {
                    *best = best.min(cost);
                }
                return;
            }
            if edges.len() - edge_index < basis_size - selected.len() {
                return;
            }
            for next in edge_index..edges.len() {
                selected.push(next);
                enumerate_bases(
                    next + 1,
                    selected,
                    edges,
                    sources,
                    targets,
                    basis_size,
                    supply,
                    demand,
                    ground,
                    dimension,
                    best,
                );
                selected.pop();
            }
        }

        let left_mass: f64 = left.iter().map(|&mass| mass as f64).sum();
        let right_mass: f64 = right.iter().map(|&mass| mass as f64).sum();
        let supply: Vec<f64> = left
            .iter()
            .filter_map(|&mass| (mass > 0.0).then_some(mass as f64 / left_mass))
            .collect();
        let demand: Vec<f64> = right
            .iter()
            .filter_map(|&mass| (mass > 0.0).then_some(mass as f64 / right_mass))
            .collect();
        let sources: Vec<usize> = left
            .iter()
            .enumerate()
            .filter_map(|(index, &mass)| (mass > 0.0).then_some(index))
            .collect();
        let targets: Vec<usize> = right
            .iter()
            .enumerate()
            .filter_map(|(index, &mass)| (mass > 0.0).then_some(index))
            .collect();
        let edges: Vec<(usize, usize)> = sources
            .iter()
            .flat_map(|&source| targets.iter().map(move |&target| (source, target)))
            .collect();
        let basis_size = supply.len() + demand.len() - 1;
        let mut best = f64::INFINITY;
        enumerate_bases(
            0,
            &mut Vec::with_capacity(basis_size),
            &edges,
            &sources,
            &targets,
            basis_size,
            &supply,
            &demand,
            ground,
            dimension,
            &mut best,
        );
        best
    }

    #[derive(Debug)]
    enum TestOnlyEmdOutcome {
        Complete {
            cost: f64,
            required_mass: f64,
            delivered_mass: f64,
            accounted_mass: f64,
            searches: Vec<SearchInvariantTrace>,
            augmentations: Vec<AugmentationTrace>,
        },
        NoPath,
        NoProgress,
        IterationLimit,
        InvalidInput,
    }

    #[derive(Clone)]
    struct TestOnlyEmdStepperSnapshot {
        assignment: Vec<u16>,
        centroids: Vec<f32>,
        rng_state: u64,
        objective: Vec<f64>,
        diagnostics: Vec<TestOnlyEmdIteration>,
        stabilized: bool,
    }

    #[derive(Clone)]
    struct TestOnlyEmdIteration {
        objective_before_update: f64,
        objective_after_update: f64,
        objective_after_reassignment: Option<f64>,
        moved_weight_fraction: f64,
        occupied_buckets: usize,
    }

    struct TestOnlyEmdStepper<'a> {
        points: &'a [f32],
        dim: usize,
        weight: &'a [u64],
        ground: &'a [f32],
        k: usize,
        assignment: Vec<u16>,
        centroids: Vec<f32>,
        rng: Rng,
        objective: Vec<f64>,
        diagnostics: Vec<TestOnlyEmdIteration>,
        stabilized: bool,
    }

    impl<'a> TestOnlyEmdStepper<'a> {
        fn new(
            points: &'a [f32],
            dim: usize,
            weight: &'a [u64],
            ground: &'a [f32],
            k: usize,
            seed: u64,
        ) -> Self {
            let n = weight.len();
            assert!(
                points.len() == n * dim
                    && ground.len() == dim * dim
                    && k <= n
                    && k <= u16::MAX as usize
            );
            let row = |index: usize| &points[index * dim..(index + 1) * dim];
            let mut rng = Rng(seed);
            let mut centroids = Vec::with_capacity(k * dim);
            for _ in 0..k {
                let index = (rng.next() % n as u64) as usize;
                centroids.extend_from_slice(row(index));
            }
            Self {
                points,
                dim,
                weight,
                ground,
                k,
                assignment: vec![u16::MAX; n],
                centroids,
                rng,
                objective: Vec::new(),
                diagnostics: Vec::new(),
                stabilized: false,
            }
        }

        fn from_snapshot(
            points: &'a [f32],
            dim: usize,
            weight: &'a [u64],
            ground: &'a [f32],
            k: usize,
            snapshot: TestOnlyEmdStepperSnapshot,
        ) -> Self {
            Self {
                points,
                dim,
                weight,
                ground,
                k,
                assignment: snapshot.assignment,
                centroids: snapshot.centroids,
                rng: Rng(snapshot.rng_state),
                objective: snapshot.objective,
                diagnostics: snapshot.diagnostics,
                stabilized: snapshot.stabilized,
            }
        }

        fn snapshot(&self) -> TestOnlyEmdStepperSnapshot {
            TestOnlyEmdStepperSnapshot {
                assignment: self.assignment.clone(),
                centroids: self.centroids.clone(),
                rng_state: self.rng.0,
                objective: self.objective.clone(),
                diagnostics: self.diagnostics.clone(),
                stabilized: self.stabilized,
            }
        }

        fn step(&mut self) -> Result<Option<TestOnlyEmdIteration>, EmdContextError> {
            self.step_impl(true)
        }

        fn step_without_early_stop(
            &mut self,
        ) -> Result<Option<TestOnlyEmdIteration>, EmdContextError> {
            self.step_impl(false)
        }

        fn step_impl(
            &mut self,
            stop_on_early_stabilization: bool,
        ) -> Result<Option<TestOnlyEmdIteration>, EmdContextError> {
            if stop_on_early_stabilization && self.stabilized {
                return Ok(None);
            }
            let n = self.weight.len();
            let row = |index: usize| &self.points[index * self.dim..(index + 1) * self.dim];
            let assigned: Vec<(u16, f32)> = (0..n)
                .into_par_iter()
                .map(|index| {
                    let point = row(index);
                    first_minimum_emd((0..self.k).map(|cluster| {
                        let centroid =
                            &self.centroids[cluster * self.dim..(cluster + 1) * self.dim];
                        emd_with_context(
                            "assignment",
                            index,
                            cluster,
                            point,
                            centroid,
                            self.ground,
                            self.dim,
                        )
                        .map(|distance| (cluster as u16, distance))
                    }))
                })
                .collect::<Result<_, _>>()?;
            let total: f64 = self.weight.iter().map(|&value| value as f64).sum();
            let moved: f64 = assigned
                .iter()
                .zip(&self.assignment)
                .zip(self.weight)
                .filter(|((next, old), _)| next.0 != **old)
                .map(|(_, &value)| value as f64)
                .sum();
            let objective_before_update = assigned
                .iter()
                .zip(self.weight)
                .map(|(entry, &value)| entry.1 as f64 * value as f64)
                .sum::<f64>()
                / total;
            if let Some(previous) = self.diagnostics.last_mut() {
                previous.objective_after_reassignment = Some(objective_before_update);
            }
            self.objective.push(objective_before_update);
            for (slot, entry) in self.assignment.iter_mut().zip(&assigned) {
                *slot = entry.0;
            }
            let mut sums = vec![0.0f64; self.k * self.dim];
            let mut mass = vec![0.0f64; self.k];
            for index in 0..n {
                let cluster = self.assignment[index] as usize;
                let row_weight = self.weight[index] as f64;
                mass[cluster] += row_weight;
                for (sum, &value) in sums[cluster * self.dim..(cluster + 1) * self.dim]
                    .iter_mut()
                    .zip(row(index))
                {
                    *sum += row_weight * value as f64;
                }
            }
            for cluster in 0..self.k {
                if mass[cluster] > 0.0 {
                    for offset in 0..self.dim {
                        self.centroids[cluster * self.dim + offset] =
                            (sums[cluster * self.dim + offset] / mass[cluster]) as f32;
                    }
                }
            }
            let objective_after_update = (0..n)
                .map(|index| {
                    let cluster = self.assignment[index] as usize;
                    emd_with_context(
                        "objective-after-update",
                        index,
                        cluster,
                        row(index),
                        &self.centroids[cluster * self.dim..(cluster + 1) * self.dim],
                        self.ground,
                        self.dim,
                    )
                    .map(|distance| distance as f64 * self.weight[index] as f64)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum::<f64>()
                / total;
            let diagnostic = TestOnlyEmdIteration {
                objective_before_update,
                objective_after_update,
                objective_after_reassignment: None,
                moved_weight_fraction: moved / total,
                occupied_buckets: mass.iter().filter(|&&value| value > 0.0).count(),
            };
            self.stabilized =
                stop_on_early_stabilization && diagnostic.moved_weight_fraction < 1e-4;
            self.diagnostics.push(diagnostic.clone());
            Ok(Some(diagnostic))
        }

        fn final_assignment_diagnostic(
            &self,
        ) -> Result<FinalAssignmentDiagnostic, EmdContextError> {
            let n = self.weight.len();
            let row = |index: usize| &self.points[index * self.dim..(index + 1) * self.dim];
            let reassigned: Vec<(u16, f32)> = (0..n)
                .into_par_iter()
                .map(|index| {
                    first_minimum_emd((0..self.k).map(|cluster| {
                        let centroid =
                            &self.centroids[cluster * self.dim..(cluster + 1) * self.dim];
                        emd_with_context(
                            "final-assignment-diagnostic",
                            index,
                            cluster,
                            row(index),
                            centroid,
                            self.ground,
                            self.dim,
                        )
                        .map(|distance| (cluster as u16, distance))
                    }))
                })
                .collect::<Result<_, _>>()?;
            let total: f64 = self.weight.iter().map(|&value| value as f64).sum();
            let changed_weight_fraction = reassigned
                .iter()
                .zip(&self.assignment)
                .zip(self.weight)
                .filter(|((next, current), _)| next.0 != **current)
                .map(|(_, &value)| value as f64)
                .sum::<f64>()
                / total;
            let objective = reassigned
                .iter()
                .zip(self.weight)
                .map(|(entry, &value)| entry.1 as f64 * value as f64)
                .sum::<f64>()
                / total;
            Ok(FinalAssignmentDiagnostic {
                changed_weight_fraction,
                objective,
            })
        }

        fn state_words(&self) -> Vec<u32> {
            self.assignment
                .iter()
                .map(|&assignment| assignment as u32)
                .chain(self.centroids.iter().map(|value| value.to_bits()))
                .chain([(self.rng.0 >> 32) as u32, self.rng.0 as u32])
                .collect()
        }
    }

    #[test]
    fn test_only_stepper_can_continue_past_early_stop() {
        let points = [1.0f32, 0.0, 0.0, 1.0];
        let weight = [1u64, 1];
        let ground = [0.0f32, 1.0, 1.0, 0.0];
        let mut stepper = TestOnlyEmdStepper::new(&points, 2, &weight, &ground, 1, 7);
        stepper.stabilized = true;

        let diagnostic = stepper
            .step_without_early_stop()
            .expect("valid test-only continuation");

        assert!(diagnostic.is_some());
        assert!(!stepper.stabilized);
    }

    fn test_only_state_fingerprint(words: &[u32]) -> u64 {
        words.iter().fold(0xcbf2_9ce4_8422_2325u64, |hash, &word| {
            (hash ^ word as u64).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    fn test_only_input_fingerprint(
        points: &[f32],
        dim: usize,
        weight: &[u64],
        ground: &[f32],
        k: usize,
    ) -> u64 {
        let words = points
            .iter()
            .map(|value| value.to_bits())
            .chain(ground.iter().map(|value| value.to_bits()))
            .chain(
                weight
                    .iter()
                    .flat_map(|&value| [value as u32, (value >> 32) as u32]),
            )
            .chain([dim as u32, (dim >> 32) as u32, k as u32, (k >> 32) as u32]);
        words.fold(0xcbf2_9ce4_8422_2325u64, |hash, word| {
            (hash ^ word as u64).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    struct TestOnlyEmdCheckpoint;

    impl TestOnlyEmdCheckpoint {
        const MAGIC: [u8; 8] = *b"HU20CP01";

        fn write_atomic(
            path: &std::path::Path,
            stepper: &TestOnlyEmdStepper<'_>,
        ) -> std::io::Result<()> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&Self::MAGIC);
            for value in [
                stepper.dim as u64,
                stepper.k as u64,
                stepper.weight.len() as u64,
                test_only_input_fingerprint(
                    stepper.points,
                    stepper.dim,
                    stepper.weight,
                    stepper.ground,
                    stepper.k,
                ),
                stepper.diagnostics.len() as u64,
                stepper.rng.0,
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes.push(u8::from(stepper.stabilized));
            Self::write_u16s(&mut bytes, &stepper.assignment);
            Self::write_f32s(&mut bytes, &stepper.centroids);
            Self::write_f64s(&mut bytes, &stepper.objective);
            bytes.extend_from_slice(&(stepper.diagnostics.len() as u64).to_le_bytes());
            for diagnostic in &stepper.diagnostics {
                bytes
                    .extend_from_slice(&diagnostic.objective_before_update.to_bits().to_le_bytes());
                bytes.extend_from_slice(&diagnostic.objective_after_update.to_bits().to_le_bytes());
                match diagnostic.objective_after_reassignment {
                    Some(value) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
                    }
                    None => bytes.push(0),
                }
                bytes.extend_from_slice(&diagnostic.moved_weight_fraction.to_bits().to_le_bytes());
                bytes.extend_from_slice(&(diagnostic.occupied_buckets as u64).to_le_bytes());
            }
            let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
            if temporary.exists() {
                std::fs::remove_file(&temporary)?;
            }
            std::fs::write(&temporary, bytes)?;
            match std::fs::rename(&temporary, path) {
                Ok(()) => Ok(()),
                Err(error) => {
                    let _ = std::fs::remove_file(&temporary);
                    Err(error)
                }
            }
        }

        fn load(
            path: &std::path::Path,
            points: &[f32],
            dim: usize,
            weight: &[u64],
            ground: &[f32],
            k: usize,
        ) -> Result<TestOnlyEmdStepperSnapshot, String> {
            Self::load_verified(
                path,
                dim,
                weight.len(),
                k,
                test_only_input_fingerprint(points, dim, weight, ground, k),
            )
        }

        fn load_verified(
            path: &std::path::Path,
            dim: usize,
            point_count: usize,
            k: usize,
            expected_input_fingerprint: u64,
        ) -> Result<TestOnlyEmdStepperSnapshot, String> {
            let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
            let mut offset = 0usize;
            let magic = Self::read_bytes(&bytes, &mut offset, 8)?;
            if magic != Self::MAGIC {
                return Err("неверный magic test-only checkpoint".to_owned());
            }
            let checkpoint_dim = Self::read_u64(&bytes, &mut offset)? as usize;
            let checkpoint_k = Self::read_u64(&bytes, &mut offset)? as usize;
            let checkpoint_n = Self::read_u64(&bytes, &mut offset)? as usize;
            let checkpoint_input = Self::read_u64(&bytes, &mut offset)?;
            let checkpoint_iterations = Self::read_u64(&bytes, &mut offset)? as usize;
            let rng_state = Self::read_u64(&bytes, &mut offset)?;
            let stabilized = match Self::read_u8(&bytes, &mut offset)? {
                0 => false,
                1 => true,
                _ => return Err("некорректный флаг стабилизации checkpoint".to_owned()),
            };
            if checkpoint_dim != dim || checkpoint_k != k || checkpoint_n != point_count {
                return Err("checkpoint несовместим с размерностями или K".to_owned());
            }
            if checkpoint_input != expected_input_fingerprint {
                return Err("checkpoint несовместим с точными входами".to_owned());
            }
            let assignment = Self::read_u16s(&bytes, &mut offset)?;
            let centroids = Self::read_f32s(&bytes, &mut offset)?;
            let objective = Self::read_f64s(&bytes, &mut offset)?;
            let diagnostics_len = Self::read_u64(&bytes, &mut offset)? as usize;
            if assignment.len() != checkpoint_n
                || centroids.len() != checkpoint_k * checkpoint_dim
                || objective.len() != checkpoint_iterations
                || diagnostics_len != checkpoint_iterations
            {
                return Err("checkpoint содержит несовместимые длины состояния".to_owned());
            }
            let mut diagnostics = Vec::with_capacity(diagnostics_len);
            for _ in 0..diagnostics_len {
                let objective_before_update = f64::from_bits(Self::read_u64(&bytes, &mut offset)?);
                let objective_after_update = f64::from_bits(Self::read_u64(&bytes, &mut offset)?);
                let objective_after_reassignment = match Self::read_u8(&bytes, &mut offset)? {
                    0 => None,
                    1 => Some(f64::from_bits(Self::read_u64(&bytes, &mut offset)?)),
                    _ => return Err("некорректный флаг reassignment checkpoint".to_owned()),
                };
                let moved_weight_fraction = f64::from_bits(Self::read_u64(&bytes, &mut offset)?);
                let occupied_buckets = Self::read_u64(&bytes, &mut offset)? as usize;
                diagnostics.push(TestOnlyEmdIteration {
                    objective_before_update,
                    objective_after_update,
                    objective_after_reassignment,
                    moved_weight_fraction,
                    occupied_buckets,
                });
            }
            if offset != bytes.len() {
                return Err("лишние байты в checkpoint".to_owned());
            }
            Ok(TestOnlyEmdStepperSnapshot {
                assignment,
                centroids,
                rng_state,
                objective,
                diagnostics,
                stabilized,
            })
        }

        fn write_u16s(bytes: &mut Vec<u8>, values: &[u16]) {
            bytes.extend_from_slice(&(values.len() as u64).to_le_bytes());
            for &value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }

        fn write_f32s(bytes: &mut Vec<u8>, values: &[f32]) {
            bytes.extend_from_slice(&(values.len() as u64).to_le_bytes());
            for &value in values {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }

        fn write_f64s(bytes: &mut Vec<u8>, values: &[f64]) {
            bytes.extend_from_slice(&(values.len() as u64).to_le_bytes());
            for &value in values {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }

        fn read_bytes<'b>(
            bytes: &'b [u8],
            offset: &mut usize,
            len: usize,
        ) -> Result<&'b [u8], String> {
            let end = offset.checked_add(len).ok_or("переполнение checkpoint")?;
            let result = bytes.get(*offset..end).ok_or("усечённый checkpoint")?;
            *offset = end;
            Ok(result)
        }

        fn read_u8(bytes: &[u8], offset: &mut usize) -> Result<u8, String> {
            Ok(Self::read_bytes(bytes, offset, 1)?[0])
        }

        fn read_u64(bytes: &[u8], offset: &mut usize) -> Result<u64, String> {
            let raw: [u8; 8] = Self::read_bytes(bytes, offset, 8)?.try_into().unwrap();
            Ok(u64::from_le_bytes(raw))
        }

        fn read_u16s(bytes: &[u8], offset: &mut usize) -> Result<Vec<u16>, String> {
            let len = Self::read_u64(bytes, offset)? as usize;
            let raw = Self::read_bytes(
                bytes,
                offset,
                len.checked_mul(2).ok_or("переполнение checkpoint")?,
            )?;
            Ok(raw
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect())
        }

        fn read_f32s(bytes: &[u8], offset: &mut usize) -> Result<Vec<f32>, String> {
            let len = Self::read_u64(bytes, offset)? as usize;
            let raw = Self::read_bytes(
                bytes,
                offset,
                len.checked_mul(4).ok_or("переполнение checkpoint")?,
            )?;
            Ok(raw
                .chunks_exact(4)
                .map(|chunk| f32::from_bits(u32::from_le_bytes(chunk.try_into().unwrap())))
                .collect())
        }

        fn read_f64s(bytes: &[u8], offset: &mut usize) -> Result<Vec<f64>, String> {
            let len = Self::read_u64(bytes, offset)? as usize;
            let raw = Self::read_bytes(
                bytes,
                offset,
                len.checked_mul(8).ok_or("переполнение checkpoint")?,
            )?;
            Ok(raw
                .chunks_exact(8)
                .map(|chunk| f64::from_bits(u64::from_le_bytes(chunk.try_into().unwrap())))
                .collect())
        }
    }

    fn test_only_record_step(
        trace: &mut std::fs::File,
        seen: &mut std::collections::HashMap<Vec<u32>, usize>,
        iteration: usize,
        stepper: &TestOnlyEmdStepper<'_>,
        diagnostic: &TestOnlyEmdIteration,
    ) -> Option<(usize, usize)> {
        use std::io::Write;

        let state = stepper.state_words();
        let fingerprint = test_only_state_fingerprint(&state);
        let repeated = seen
            .insert(state, iteration)
            .map(|first_seen_at| (first_seen_at, iteration));
        writeln!(
            trace,
            "iteration={iteration} objective_before={:.9} objective_after={:.9} moved_weight_fraction={:.9} occupied_buckets={} state_fingerprint={fingerprint:016X}",
            diagnostic.objective_before_update,
            diagnostic.objective_after_update,
            diagnostic.moved_weight_fraction,
            diagnostic.occupied_buckets,
        )
        .expect("write per-step flop probe trace");
        trace.flush().expect("flush per-step flop probe trace");
        repeated
    }

    fn test_only_write_checkpoint(
        directory: &std::path::Path,
        iteration: usize,
        stepper: &TestOnlyEmdStepper<'_>,
    ) {
        let path = directory.join(format!("checkpoint-{iteration:02}.bin"));
        assert!(
            !path.exists(),
            "checkpoint already exists: {}",
            path.display()
        );
        TestOnlyEmdCheckpoint::write_atomic(&path, stepper).expect("write atomic step checkpoint");
    }

    fn test_only_resume_from_checkpoint<'a>(
        path: &std::path::Path,
        points: &'a [f32],
        dim: usize,
        weight: &'a [u64],
        ground: &'a [f32],
        k: usize,
    ) -> Result<TestOnlyEmdStepper<'a>, String> {
        let snapshot = TestOnlyEmdCheckpoint::load(path, points, dim, weight, ground, k)?;
        Ok(TestOnlyEmdStepper::from_snapshot(
            points, dim, weight, ground, k, snapshot,
        ))
    }

    fn test_only_load_checkpoint_history(
        checkpoints: &[(usize, &std::path::Path)],
        points: &[f32],
        dim: usize,
        weight: &[u64],
        ground: &[f32],
        k: usize,
    ) -> Result<std::collections::HashMap<Vec<u32>, usize>, String> {
        let mut history = std::collections::HashMap::new();
        for &(iteration, path) in checkpoints {
            let stepper = test_only_resume_from_checkpoint(path, points, dim, weight, ground, k)?;
            if stepper.diagnostics.len() != iteration {
                return Err(format!(
                    "checkpoint {} содержит шаг {}, ожидался {iteration}",
                    path.display(),
                    stepper.diagnostics.len(),
                ));
            }
            let state = stepper.state_words();
            if let Some(previous) = history.insert(state, iteration) {
                return Err(format!(
                    "checkpoint {iteration} повторяет полное состояние checkpoint {previous}"
                ));
            }
        }
        Ok(history)
    }

    fn test_only_load_trace_metrics(
        path: &std::path::Path,
    ) -> Result<std::collections::HashMap<usize, (f64, f64, f64)>, String> {
        let trace = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        let mut result = std::collections::HashMap::new();
        for line in trace.lines().filter(|line| line.starts_with("iteration=")) {
            let fields = line
                .split_whitespace()
                .filter_map(|field| field.split_once('='))
                .collect::<std::collections::HashMap<_, _>>();
            let parse = |name| {
                fields
                    .get(name)
                    .ok_or_else(|| format!("trace lacks {name}"))?
                    .parse::<f64>()
                    .map_err(|error| error.to_string())
            };
            let iteration = fields
                .get("iteration")
                .ok_or_else(|| "trace lacks iteration".to_owned())?
                .parse::<usize>()
                .map_err(|error| error.to_string())?;
            let metrics = (
                parse("objective_before")?,
                parse("objective_after")?,
                parse("moved_weight_fraction")?,
            );
            if result.insert(iteration, metrics).is_some() {
                return Err(format!("duplicate trace iteration {iteration}"));
            }
        }
        Ok(result)
    }

    #[derive(Debug)]
    struct AugmentationTrace {
        amount: f64,
        delivered_before: f64,
        delivered_after: f64,
        forward_capacity_changed: bool,
        reverse_capacity_changed: bool,
    }

    #[derive(Debug)]
    struct ResidualChange {
        forward_capacity_changed: bool,
        reverse_capacity_changed: bool,
    }

    impl ResidualChange {
        fn changed(&self) -> bool {
            self.forward_capacity_changed || self.reverse_capacity_changed
        }
    }

    fn test_only_transport_residuals(
        graph: &[Vec<FlowEdge>],
        source: usize,
        sink: usize,
        dimension: usize,
    ) -> (f64, f64) {
        let source_residual = graph[source].iter().map(|edge| edge.capacity).sum();
        let demand_residual = (0..dimension)
            .map(|index| {
                graph[dimension + index]
                    .iter()
                    .find(|edge| edge.to == sink)
                    .expect("demand node must connect to sink")
                    .capacity
            })
            .sum();
        (source_residual, demand_residual)
    }

    fn test_only_transport_is_complete(
        graph: &[Vec<FlowEdge>],
        source: usize,
        sink: usize,
        dimension: usize,
    ) -> bool {
        let (source_residual, demand_residual) =
            test_only_transport_residuals(graph, source, sink, dimension);
        source_residual <= test_only_completion_tolerance(dimension)
            && demand_residual <= test_only_completion_tolerance(dimension)
    }

    fn test_only_apply_augmentation(
        graph: &mut [Vec<FlowEdge>],
        path: &[(usize, usize)],
        sink: usize,
        amount: f64,
    ) -> ResidualChange {
        let mut forward_capacity_changed = false;
        let mut reverse_capacity_changed = false;
        let mut node = sink;
        for &(from, edge) in path {
            let reverse = graph[from][edge].reverse;
            let forward_capacity_before = graph[from][edge].capacity;
            let reverse_capacity_before = graph[node][reverse].capacity;
            graph[from][edge].capacity -= amount;
            graph[node][reverse].capacity += amount;
            forward_capacity_changed |= graph[from][edge].capacity != forward_capacity_before;
            reverse_capacity_changed |= graph[node][reverse].capacity != reverse_capacity_before;
            node = from;
        }
        ResidualChange {
            forward_capacity_changed,
            reverse_capacity_changed,
        }
    }

    fn test_only_completion_tolerance(dimension: usize) -> f64 {
        64.0 * f64::EPSILON * dimension as f64
    }

    fn test_only_emd_without_capacity_threshold(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
    ) -> TestOnlyEmdOutcome {
        test_only_emd_without_capacity_threshold_with_limit(
            left,
            right,
            ground,
            dimension,
            4 * (2 * dimension + 2).pow(2),
        )
    }

    fn test_only_emd_without_capacity_threshold_with_limit(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
        iteration_limit: usize,
    ) -> TestOnlyEmdOutcome {
        test_only_emd_with_residual_bottleneck(
            left,
            right,
            ground,
            dimension,
            iteration_limit,
            true,
        )
    }

    fn test_only_emd_without_delivered_amount_cap(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
    ) -> TestOnlyEmdOutcome {
        test_only_emd_with_residual_bottleneck(
            left,
            right,
            ground,
            dimension,
            4 * (2 * dimension + 2).pow(2),
            false,
        )
    }

    fn test_only_emd_with_residual_bottleneck(
        left: &[f32],
        right: &[f32],
        ground: &[f32],
        dimension: usize,
        iteration_limit: usize,
        cap_amount_by_delivered_mass: bool,
    ) -> TestOnlyEmdOutcome {
        if left.len() != dimension
            || right.len() != dimension
            || ground.len() != dimension * dimension
            || left
                .iter()
                .chain(right)
                .any(|mass| !mass.is_finite() || *mass < 0.0)
            || ground.iter().any(|cost| !cost.is_finite() || *cost < 0.0)
        {
            return TestOnlyEmdOutcome::InvalidInput;
        }
        let left_mass: f64 = left.iter().map(|&mass| mass as f64).sum();
        let right_mass: f64 = right.iter().map(|&mass| mass as f64).sum();
        if left_mass <= 0.0 || right_mass <= 0.0 {
            return TestOnlyEmdOutcome::InvalidInput;
        }
        let normalized_left: Vec<f64> = left.iter().map(|&mass| mass as f64 / left_mass).collect();
        let normalized_right: Vec<f64> =
            right.iter().map(|&mass| mass as f64 / right_mass).collect();
        let required_mass = normalized_left
            .iter()
            .sum::<f64>()
            .min(normalized_right.iter().sum());
        let source = 2 * dimension;
        let sink = source + 1;
        let mut graph = vec![Vec::new(); sink + 1];
        for index in 0..dimension {
            add_flow_edge(&mut graph, source, index, normalized_left[index], 0.0);
            add_flow_edge(
                &mut graph,
                dimension + index,
                sink,
                normalized_right[index],
                0.0,
            );
            for target in 0..dimension {
                add_flow_edge(
                    &mut graph,
                    index,
                    dimension + target,
                    1.0,
                    ground[index * dimension + target] as f64,
                );
            }
        }
        let mut potential = vec![0.0; graph.len()];
        let mut delivered = 0.0;
        let mut total_cost = 0.0;
        let mut searches = Vec::new();
        let mut augmentations = Vec::new();
        for search_index in 0..iteration_limit {
            if test_only_transport_is_complete(&graph, source, sink, dimension) {
                let (source_residual, _) =
                    test_only_transport_residuals(&graph, source, sink, dimension);
                return TestOnlyEmdOutcome::Complete {
                    cost: total_cost,
                    required_mass,
                    delivered_mass: required_mass - source_residual,
                    accounted_mass: delivered,
                    searches,
                    augmentations,
                };
            }
            let mut reachable = vec![false; graph.len()];
            let mut pending = std::collections::VecDeque::from([source]);
            reachable[source] = true;
            while let Some(from) = pending.pop_front() {
                if from == sink {
                    continue;
                }
                for edge in &graph[from] {
                    if edge.capacity > 0.0 && !reachable[edge.to] {
                        reachable[edge.to] = true;
                        pending.push_back(edge.to);
                    }
                }
            }
            let mut negative_eligible_reduced_cost_edges = Vec::new();
            for (from, edges) in graph.iter().enumerate() {
                for edge in edges {
                    let reduced_cost = edge.cost + potential[from] - potential[edge.to];
                    if edge.capacity > 0.0 && reduced_cost < 0.0 {
                        negative_eligible_reduced_cost_edges.push(ReducedCostEdgeTrace {
                            search_index,
                            from,
                            to: edge.to,
                            capacity: edge.capacity,
                            from_potential: potential[from],
                            to_potential: potential[edge.to],
                            reduced_cost,
                            from_reachable_from_source: reachable[from],
                            from_is_relaxable: reachable[from] && from != sink,
                        });
                    }
                }
            }
            searches.push(SearchInvariantTrace {
                search_index,
                sink_reachable_from_source: reachable[sink],
                sink_outgoing_eligible_edges: graph[sink]
                    .iter()
                    .filter(|edge| edge.capacity > 0.0)
                    .count(),
                negative_eligible_reduced_cost_edges,
            });
            let mut distance = vec![f64::INFINITY; graph.len()];
            let mut previous = vec![None; graph.len()];
            let mut used = vec![false; graph.len()];
            distance[source] = 0.0;
            for _ in 0..graph.len() {
                let next = (0..graph.len())
                    .filter(|&vertex| !used[vertex])
                    .min_by(|&left, &right| distance[left].partial_cmp(&distance[right]).unwrap())
                    .expect("graph must contain an unused vertex");
                if !distance[next].is_finite() {
                    break;
                }
                used[next] = true;
                if next == sink {
                    continue;
                }
                for (edge_index, edge) in graph[next].iter().enumerate() {
                    if edge.capacity <= 0.0 || used[edge.to] {
                        continue;
                    }
                    let candidate =
                        distance[next] + edge.cost + potential[next] - potential[edge.to];
                    if candidate < distance[edge.to] {
                        distance[edge.to] = candidate;
                        previous[edge.to] = Some((next, edge_index));
                    }
                }
            }
            if !used[sink] {
                return TestOnlyEmdOutcome::NoPath;
            }
            for vertex in 0..graph.len() {
                if used[vertex] {
                    potential[vertex] += distance[vertex];
                }
            }
            let path = match reconstruct_path(&previous, source, sink) {
                Ok(path) => path,
                Err(_) => return TestOnlyEmdOutcome::NoProgress,
            };
            let initial_amount = cap_amount_by_delivered_mass
                .then_some(required_mass - delivered)
                .unwrap_or(f64::INFINITY);
            let amount = path.iter().fold(initial_amount, |amount, &(from, edge)| {
                amount.min(graph[from][edge].capacity)
            });
            if !amount.is_finite() || amount <= 0.0 {
                return TestOnlyEmdOutcome::NoProgress;
            }
            for &(from, edge) in &path {
                total_cost += amount * graph[from][edge].cost;
            }
            let delivered_before = delivered;
            let residual_change = test_only_apply_augmentation(&mut graph, &path, sink, amount);
            delivered += amount;
            augmentations.push(AugmentationTrace {
                amount,
                delivered_before,
                delivered_after: delivered,
                forward_capacity_changed: residual_change.forward_capacity_changed,
                reverse_capacity_changed: residual_change.reverse_capacity_changed,
            });
            if !residual_change.changed() {
                return TestOnlyEmdOutcome::NoProgress;
            }
        }
        TestOnlyEmdOutcome::IterationLimit
    }

    #[test]
    fn ranks5_failure_fixture_production_solver_matches_independent_cost() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();

        let actual = try_emd(&left, &right, &ground, 10)
            .expect("captured ranks=5 transport must now complete");

        assert!(
            (actual as f64 - independent_transport_cost(&left, &right, &ground, 10)).abs() < 1e-8
        );
    }

    #[test]
    fn ranks5_failure_fixture_reports_residual_graph_state() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();
        let trace = trace_disconnected_transport(&left, &right, &ground, 10);

        println!("{trace:#?}");
        assert!(trace.delivered + 1e-9 < trace.transport_mass);
        assert_eq!(trace.augmentations, 6);
        assert_eq!(trace.reachable_supply, vec![0, 2, 6, 8, 9]);
        assert_eq!(trace.reachable_demand, (0..10).collect::<Vec<_>>());
        assert!(trace.source_residual > 1e-9);
        assert!(trace.demand_residual > 1e-9);
        assert_eq!(trace.eligible_demand_edges, 0);
        assert!(trace.minimum_reduced_cost.is_sign_negative());
    }

    #[test]
    fn ranks5_failure_fixture_identifies_negative_eligible_reduced_cost_before_search() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();
        let trace = trace_disconnected_transport(&left, &right, &ground, 10);

        let edge = trace
            .first_negative_eligible_reduced_cost
            .expect("fixture must identify the first negative eligible reduced-cost edge");
        assert!(edge.reduced_cost < 0.0);
        assert_eq!(edge.search_index, 1);
        assert!(edge.capacity > 1e-9);
        assert!(edge.from_potential.is_finite());
        assert!(edge.to_potential.is_finite());
        assert!(!edge.from_reachable_from_source);
    }

    #[test]
    fn ranks5_failure_fixture_has_no_negative_reduced_cost_on_relaxable_edges() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();
        let trace = trace_disconnected_transport(&left, &right, &ground, 10);

        assert!(trace.search_invariants.iter().all(|search| {
            search
                .negative_eligible_reduced_cost_edges
                .iter()
                .all(|edge| !edge.from_is_relaxable)
        }));
        assert!(trace
            .search_invariants
            .iter()
            .enumerate()
            .all(|(index, search)| search.search_index == index));
        assert!(trace
            .search_invariants
            .iter()
            .any(|search| search.sink_outgoing_eligible_edges > 0));
        assert!(trace
            .search_invariants
            .iter()
            .any(|search| !search.sink_reachable_from_source));
    }

    #[test]
    fn ranks5_failure_fixture_has_finite_independent_transport_cost() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();

        let cost = independent_transport_cost(&left, &right, &ground, 10);

        assert!(cost.is_finite());
        assert!(
            (cost - 0.011_323_424_731_416).abs() < 1e-12,
            "cost={cost:.15}"
        );
    }

    #[test]
    fn no_threshold_candidate_completes_ranks5_fixture_at_independent_cost() {
        let (left, right, ground) = ranks5_disconnected_transport_fixture();

        let outcome = test_only_emd_without_capacity_threshold(&left, &right, &ground, 10);

        let TestOnlyEmdOutcome::Complete {
            cost,
            required_mass,
            delivered_mass,
            searches,
            ..
        } = outcome
        else {
            panic!("candidate must deliver the full captured transport");
        };
        assert!((required_mass - delivered_mass).abs() <= test_only_completion_tolerance(10));
        assert!((cost - independent_transport_cost(&left, &right, &ground, 10)).abs() < 1e-12);
        assert!(searches.iter().all(|search| {
            search
                .negative_eligible_reduced_cost_edges
                .iter()
                .all(|edge| !edge.from_is_relaxable)
        }));
    }

    #[test]
    fn captured_flop_no_progress_fixture_production_solver_completes_at_oracle_cost() {
        let (left, right, ground) = flop_no_progress_fixture();

        let actual = try_emd(&left, &right, &ground, 10)
            .expect("captured point=1, centroid=9 transport must complete");

        assert!(
            (actual as f64 - independent_transport_cost(&left, &right, &ground, 10)).abs() < 1e-8
        );
    }

    #[test]
    fn no_threshold_candidate_completes_captured_flop_no_progress_fixture() {
        let (left, right, ground) = flop_no_progress_fixture();

        let outcome = test_only_emd_without_capacity_threshold(&left, &right, &ground, 10);

        let TestOnlyEmdOutcome::Complete {
            cost,
            required_mass,
            delivered_mass,
            accounted_mass,
            augmentations,
            ..
        } = outcome
        else {
            panic!("captured point=1, centroid=9 transport must complete");
        };
        assert!((required_mass - delivered_mass).abs() <= test_only_completion_tolerance(10));
        assert!((required_mass - accounted_mass).abs() <= test_only_completion_tolerance(10));
        assert!(augmentations.iter().any(|augmentation| {
            augmentation.amount > 0.0
                && augmentation.delivered_before == augmentation.delivered_after
                && (augmentation.forward_capacity_changed || augmentation.reverse_capacity_changed)
        }));
        let oracle_cost = independent_transport_cost(&left, &right, &ground, 10);
        assert!((oracle_cost - 0.132_193_286_987_868_22).abs() < 1e-12);
        assert!((cost - oracle_cost).abs() < 1e-12);
    }

    #[test]
    fn residual_bottleneck_candidate_completes_captured_flop_fixture_at_oracle_cost() {
        let (left, right, ground) = flop_no_progress_fixture();

        let outcome = test_only_emd_without_delivered_amount_cap(&left, &right, &ground, 10);

        let TestOnlyEmdOutcome::Complete {
            cost,
            required_mass,
            delivered_mass,
            augmentations,
            ..
        } = outcome
        else {
            panic!("residual-bottleneck candidate must complete captured transport");
        };
        assert!((required_mass - delivered_mass).abs() <= test_only_completion_tolerance(10));
        assert!(augmentations.iter().any(|augmentation| {
            augmentation.amount > 0.0
                && augmentation.delivered_before == augmentation.delivered_after
                && (augmentation.forward_capacity_changed || augmentation.reverse_capacity_changed)
        }));
        assert!((cost - independent_transport_cost(&left, &right, &ground, 10)).abs() < 1e-12);
    }

    #[test]
    fn no_threshold_candidate_matches_oracle_on_small_residual_and_sparse_cases() {
        let cases = [
            (
                vec![1.0, 2e-9],
                vec![1.0 - 2e-9, 4e-9],
                vec![0.0, 3.0, 3.0, 0.0],
            ),
            (
                vec![0.0, 7.0, 0.0, 3.0],
                vec![0.0, 2.0, 0.0, 8.0],
                vec![
                    0.0, 0.0, 4.0, 4.0, 0.0, 0.0, 4.0, 4.0, 4.0, 4.0, 0.0, 0.0, 4.0, 4.0, 0.0, 0.0,
                ],
            ),
        ];
        for (left, right, ground) in cases {
            let dimension = left.len();
            let outcome =
                test_only_emd_without_capacity_threshold(&left, &right, &ground, dimension);
            let TestOnlyEmdOutcome::Complete { cost, .. } = outcome else {
                panic!("candidate must complete valid case");
            };
            assert!(
                (cost - independent_transport_cost(&left, &right, &ground, dimension)).abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn no_threshold_candidate_rejects_invalid_input_without_success() {
        assert!(matches!(
            test_only_emd_without_capacity_threshold(&[0.0, 0.0], &[1.0, 0.0], &[0.0; 4], 2),
            TestOnlyEmdOutcome::InvalidInput
        ));
    }

    #[test]
    fn no_threshold_candidate_reports_iteration_limit_without_partial_success() {
        let ground = [0.0, 1.0, 1.0, 0.0];

        assert!(matches!(
            test_only_emd_without_capacity_threshold_with_limit(
                &[1.0, 0.0],
                &[0.0, 1.0],
                &ground,
                2,
                0,
            ),
            TestOnlyEmdOutcome::IterationLimit
        ));
    }

    #[test]
    fn incomplete_residual_graph_is_not_a_successful_transport() {
        let source = 4;
        let sink = 5;
        let mut graph = vec![Vec::new(); sink + 1];
        add_flow_edge(&mut graph, source, 0, 0.0, 0.0);
        add_flow_edge(&mut graph, 2, sink, 0.25, 0.0);
        add_flow_edge(&mut graph, 3, sink, 0.0, 0.0);

        assert!(!test_only_transport_is_complete(&graph, source, sink, 2));
    }

    #[test]
    fn test_only_augmentation_reports_no_progress_when_residual_graph_is_unchanged() {
        let mut graph = vec![Vec::new(); 2];
        add_flow_edge(&mut graph, 0, 1, f64::MAX, 0.0);
        graph[1][0].capacity = f64::MAX;

        assert!(!test_only_apply_augmentation(&mut graph, &[(0, 0)], 1, 1.0).changed());
    }

    #[test]
    fn independent_transport_oracle_finds_optimal_non_greedy_split() {
        let left = [8.0, 1.0, 3.0];
        let right = [5.0, 5.0, 2.0];
        let ground = [0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 100.0, 0.0, 0.0];

        assert_eq!(independent_transport_cost(&left, &right, &ground, 3), 0.0);
    }

    #[test]
    fn streaming_emd_minimum_matches_reference_for_k1_ties_and_signed_zeroes() {
        let cases = [
            vec![(0, 1.25)],
            vec![(0, 0.0), (1, 0.0), (2, 0.5)],
            vec![(0, 3.0), (1, 1.0), (2, 2.0), (3, 1.0)],
            vec![(0, -0.0), (1, 0.0), (2, -0.0)],
            vec![(0, 0.0), (1, -0.0), (2, 0.0)],
        ];

        for candidates in cases {
            let expected = first_minimum_emd_reference(candidates.clone());
            let actual = first_minimum_emd(candidates.into_iter().map(Ok))
                .expect("all candidates are valid");

            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1.to_bits(), expected.1.to_bits());
        }
    }

    #[test]
    fn streaming_emd_minimum_returns_the_same_first_error_as_reference_assignment() {
        let point = [1.0, 0.0];
        let centroids = [1.0, 0.0, -1.0, 2.0, 0.0, 1.0];
        let ground = [0.0, 1.0, 1.0, 0.0];
        let reference: Result<Vec<(u16, f32)>, EmdContextError> = (0..3)
            .map(|cluster| {
                emd_with_context(
                    "assignment",
                    7,
                    cluster,
                    &point,
                    &centroids[cluster * 2..(cluster + 1) * 2],
                    &ground,
                    2,
                )
                .map(|distance| (cluster as u16, distance))
            })
            .collect();
        let evaluated = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&evaluated);
        let actual = first_minimum_emd((0..3).map(move |cluster| {
            observed.fetch_add(1, Ordering::Relaxed);
            emd_with_context(
                "assignment",
                7,
                cluster,
                &point,
                &centroids[cluster * 2..(cluster + 1) * 2],
                &ground,
                2,
            )
            .map(|distance| (cluster as u16, distance))
        }));

        let reference = reference.expect_err("second centroid must fail");
        let actual = actual.expect_err("second centroid must fail");
        assert_eq!(actual.phase, reference.phase);
        assert_eq!(actual.point_index, reference.point_index);
        assert_eq!(actual.centroid_index, reference.centroid_index);
        assert_eq!(evaluated.load(Ordering::Relaxed), 2);
        assert!(matches!(
            actual.source,
            EmdError::NegativeMass {
                side: "right",
                index: 0
            }
        ));
    }

    fn brute_force_transport_cost(
        supply: &[u8],
        demand: &[u8],
        ground: &[f32],
        dimension: usize,
    ) -> f64 {
        fn assign(
            source: usize,
            remaining: u8,
            supply: &[u8],
            demand: &mut [u8],
            ground: &[f32],
            dimension: usize,
        ) -> f64 {
            if source == dimension {
                return if demand.iter().all(|&mass| mass == 0) {
                    0.0
                } else {
                    f64::INFINITY
                };
            }
            if remaining == 0 {
                return assign(
                    source + 1,
                    supply.get(source + 1).copied().unwrap_or(0),
                    supply,
                    demand,
                    ground,
                    dimension,
                );
            }
            let mut best = f64::INFINITY;
            for target in 0..dimension {
                if demand[target] == 0 {
                    continue;
                }
                demand[target] -= 1;
                best = best.min(
                    ground[source * dimension + target] as f64
                        + assign(source, remaining - 1, supply, demand, ground, dimension),
                );
                demand[target] += 1;
            }
            best
        }

        assert_eq!(supply.iter().sum::<u8>(), demand.iter().sum::<u8>());
        let total = supply.iter().sum::<u8>() as f64;
        let mut demand = demand.to_vec();
        assign(0, supply[0], supply, &mut demand, ground, dimension) / total
    }

    #[test]
    fn evaluator_orders_categories_and_kickers() {
        let order = [
            "2c 3d 5h 7s 9c Jd Kh", // high card
            "2c 2d 5h 7s 9c Jd Kh", // pair
            "2c 2d 5h 5s 9c Jd Kh", // two pair
            "2c 2d 2h 7s 9c Jd Kh", // trips
            "Ac 2d 3h 4s 5c Jd Kh", // wheel
            "6c 2d 3h 4s 5c Jd Kh", // six-high straight
            "2c 4c 6c 8c Tc Jd Kh", // flush
            "2c 2d 2h 7s 7c Jd Kh", // full house
            "2c 2d 2h 2s 7c Jd Kh", // quads
            "Ac 2c 3c 4c 5c Jd Kh", // steel wheel
            "Tc Jc Qc Kc Ac 2d 3h", // royal
        ];
        let values: Vec<u32> = order.iter().map(|h| evaluate(&cards(h))).collect();
        for pair in values.windows(2) {
            assert!(pair[0] < pair[1], "{values:?}");
        }
        // Kickers and board plays.
        assert!(
            evaluate(&cards("Ac Ad Kh 7s 5c 3d 2h")) > evaluate(&cards("Ac Ad Qh 7s 5c 3d 2h"))
        );
        assert_eq!(
            evaluate(&cards("2c 3d Ah Ks Qc Jd Th")),
            evaluate(&cards("4c 5d Ah Ks Qc Jd Th"))
        );
        // Two trips make a full house with the higher trips.
        assert!(
            evaluate(&cards("Kc Kd Kh 9s 9c 9d 2h")) > evaluate(&cards("Qc Qd Qh As Ac 3d 2h"))
        );
    }

    /// Exact category counts over all 133,784,560 seven-card hands. Slow; release only.
    #[test]
    #[ignore]
    fn evaluator_matches_all_seven_card_category_counts() {
        let expected = [
            23_294_460u64,
            58_627_800,
            31_433_400,
            6_461_620,
            6_180_020,
            4_047_644,
            3_473_184,
            224_848,
            41_584,
        ];
        let counts = (0..52u8)
            .into_par_iter()
            .map(|a| {
                let mut local = [0u64; 9];
                for b in a + 1..52 {
                    for c in b + 1..52 {
                        for d in c + 1..52 {
                            for e in d + 1..52 {
                                for f in e + 1..52 {
                                    for g in f + 1..52 {
                                        local[category(evaluate(&[a, b, c, d, e, f, g]))
                                            as usize] += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                local
            })
            .reduce(
                || [0u64; 9],
                |mut x, y| {
                    for i in 0..9 {
                        x[i] += y[i];
                    }
                    x
                },
            );
        assert_eq!(counts, expected);
    }

    #[test]
    fn tables_round_trip_and_reject_corruption() {
        let board = cards("2h 7c 9d Ks");
        let holes = [cards("Ah Kh"), cards("As Kd"), cards("Qc Qd")];
        let keys: Vec<u128> = holes.iter().map(|h| class_key(h, &board)).collect();
        let table = BucketTable::new(2, 50, &keys, &[3, 49, 0]);
        let path =
            std::env::temp_dir().join(format!("hu20-bucket-table-{}.bin", std::process::id()));
        table.write(&path).unwrap();
        let read = BucketTable::read(&path).unwrap();
        assert_eq!((read.street, read.k, read.len()), (2, 50, 3));
        for (hole, bucket) in holes.iter().zip([3, 49, 0]) {
            assert_eq!(read.bucket(hole, &board), Some(bucket));
        }
        // A suit-isomorphic holding shares its class; an absent class has no bucket.
        assert_eq!(read.bucket(&cards("Ad Kd"), &cards("2d 7c 9h Ks")), Some(3));
        assert_eq!(read.bucket(&cards("Jc Tc"), &board), None);
        let mut bytes = std::fs::read(&path).unwrap();
        let last = bytes.len() - 2;
        bytes[last] = 50; // a bucket id equal to K
        std::fs::write(&path, &bytes).unwrap();
        assert!(BucketTable::read(&path).is_err());
        std::fs::write(&path, &bytes[..bytes.len() - 1]).unwrap();
        assert!(BucketTable::read(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn class_keys_respect_suit_symmetry_only() {
        let a = class_key(&cards("Ah Kh"), &cards("2h 7c 9d"));
        let b = class_key(&cards("As Ks"), &cards("2s 7d 9c"));
        let c = class_key(&cards("Ah Kd"), &cards("2h 7c 9d"));
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(orbit_size(&cards("Ah Kh"), &cards("2h 7c 9d")), 24);
        assert_eq!(orbit_size(&cards("Ah Ad"), &[]), 6);
    }

    #[test]
    fn river_equity_matches_brute_force() {
        let deck = deck(13);
        let board = cards("Ah Kd 7c 7s 2h");
        let table = river_equities(&board, &deck);
        for (hole, eq) in table.iter().step_by(97) {
            let mine = evaluate(&[
                hole[0], hole[1], board[0], board[1], board[2], board[3], board[4],
            ]);
            let (mut score, mut count) = (0f64, 0f64);
            let rest: Vec<u8> = deck
                .iter()
                .copied()
                .filter(|c| !board.contains(c) && !hole.contains(c))
                .collect();
            for o in combinations(&rest, 2) {
                let theirs =
                    evaluate(&[o[0], o[1], board[0], board[1], board[2], board[3], board[4]]);
                score += if mine > theirs {
                    1.0
                } else if mine == theirs {
                    0.5
                } else {
                    0.0
                };
                count += 1.0;
            }
            assert!((score / count - *eq as f64).abs() < 1e-6, "{hole:?}");
        }
    }

    #[test]
    fn small_deck_orbits_cover_every_raw_situation() {
        let deck = deck(5);
        let river = RiverTable::build(&deck);
        let n = deck.len() as u64;
        let choose = |n: u64, k: u64| (0..k).fold(1u64, |acc, i| acc * (n - i) / (i + 1));
        assert_eq!(
            river.weight.iter().sum::<u64>(),
            choose(n, 5) * choose(n - 5, 2)
        );
        let turn = StreetFeatures::build(&deck, 4, 10, &river);
        assert_eq!(
            turn.weight.iter().sum::<u64>(),
            choose(n, 4) * choose(n - 4, 2)
        );
        let flop = StreetFeatures::build(&deck, 3, 10, &river);
        assert_eq!(
            flop.weight.iter().sum::<u64>(),
            choose(n, 3) * choose(n - 3, 2)
        );
        let clusters = kmeans(&turn.cdf, turn.bins, &turn.weight, 8, 1, 50);
        assert_eq!(clusters.assignment.len(), turn.keys.len());
        assert!(clusters.assignment.iter().all(|&c| c < 8));
        assert!(clusters.objective.iter().all(|o| o.is_finite()));
        // Distinct classes must not collide in the 64-bit table key.
        let mut hashes: Vec<u64> = river.keys.iter().map(|&k| hash_key(k)).collect();
        hashes.sort_unstable();
        assert!(hashes.windows(2).all(|w| w[0] != w[1]));
    }

    #[test]
    fn emd_uses_ground_distance_for_mass_transfer() {
        let ground = vec![0.0, 2.0, 2.0, 0.0];
        assert!((emd(&[1.0, 0.0], &[0.0, 1.0], &ground, 2) - 2.0).abs() < 1e-6);
        assert!((emd(&[0.5, 0.5], &[0.0, 1.0], &ground, 2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn emd_scratch_matches_pre_refactor_reference_after_reuse_and_error() {
        let ground2 = [0.0, 1.0, 1.0, 0.0];
        let ground3 = [0.0, 1.0, 2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0];
        let mut scratch = EmdScratch::default();

        let cases = [
            (&[1.0, 0.0][..], &[0.25, 0.75][..], &ground2[..], 2usize),
            (
                &[0.0, 0.3, 0.7][..],
                &[0.5, 0.5, 0.0][..],
                &ground3[..],
                3usize,
            ),
            (&[0.2, 0.8][..], &[0.2, 0.8][..], &ground2[..], 2usize),
        ];
        for (left, right, ground, dimension) in cases {
            let reference = try_emd_pre_scratch_reference(left, right, ground, dimension);
            let actual = try_emd_with_scratch(left, right, ground, dimension, &mut scratch);
            let wrapper = try_emd(left, right, ground, dimension);
            let expected_bits = reference.unwrap().to_bits();
            assert_eq!(actual.unwrap().to_bits(), expected_bits);
            assert_eq!(wrapper.unwrap().to_bits(), expected_bits);
        }

        let expected_error = try_emd_pre_scratch_reference(&[-1.0, 2.0], &[0.5, 0.5], &ground2, 2)
            .expect_err("negative mass must fail in reference");
        let actual_error =
            try_emd_with_scratch(&[-1.0, 2.0], &[0.5, 0.5], &ground2, 2, &mut scratch)
                .expect_err("negative mass must fail with reused scratch");
        assert_eq!(actual_error, expected_error);

        let expected_context = emd_with_context(
            "scratch-error-context",
            3,
            5,
            &[0.5, 0.5],
            &[0.5, 0.5],
            &[0.0, f32::NAN, 1.0, 0.0],
            2,
        )
        .expect_err("reference context must fail");
        let actual_context = emd_with_context_with_scratch(
            "scratch-error-context",
            3,
            5,
            &[0.5, 0.5],
            &[0.5, 0.5],
            &[0.0, f32::NAN, 1.0, 0.0],
            2,
            &mut scratch,
        )
        .expect_err("scratch context must fail");
        assert_eq!(actual_context.phase, expected_context.phase);
        assert_eq!(actual_context.point_index, expected_context.point_index);
        assert_eq!(
            actual_context.centroid_index,
            expected_context.centroid_index
        );
        assert_eq!(actual_context.left, expected_context.left);
        assert_eq!(actual_context.right, expected_context.right);
        assert_eq!(actual_context.source, expected_context.source);
        assert_eq!(actual_context.ground.len(), expected_context.ground.len());
        for (actual, expected) in actual_context.ground.iter().zip(&expected_context.ground) {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }

        let reference = try_emd_pre_scratch_reference(&[0.9, 0.1], &[0.4, 0.6], &ground2, 2);
        let actual = try_emd_with_scratch(&[0.9, 0.1], &[0.4, 0.6], &ground2, 2, &mut scratch);
        assert_eq!(actual.unwrap().to_bits(), reference.unwrap().to_bits());
    }

    #[test]
    #[ignore = "ручной release microbenchmark EmdScratch; не запускает генератор"]
    fn test_only_emd_scratch_microbenchmark() {
        use std::hint::black_box;

        const DIMENSION: usize = 20;
        const WARMUP_CALLS: usize = 128;
        const CALLS_PER_SAMPLE: usize = 5_000;
        const SAMPLES: usize = 3;
        let timeout = Duration::from_secs(20);
        let left: Vec<f32> = (0..DIMENSION)
            .map(|index| ((index * 7 % 19) + 1) as f32)
            .collect();
        let right: Vec<f32> = (0..DIMENSION)
            .map(|index| ((index * 11 % 17) + 1) as f32)
            .collect();
        let ground: Vec<f32> = (0..DIMENSION)
            .flat_map(|left| (0..DIMENSION).map(move |right| left.abs_diff(right) as f32))
            .collect();
        let deadline = Instant::now() + timeout;
        let mut scratch = EmdScratch::default();
        for _ in 0..WARMUP_CALLS {
            let reference = try_emd_pre_scratch_reference(&left, &right, &ground, DIMENSION)
                .expect("reference warmup");
            let actual = try_emd_with_scratch(&left, &right, &ground, DIMENSION, &mut scratch)
                .expect("scratch warmup");
            assert_eq!(actual.to_bits(), reference.to_bits());
        }

        let mut reference_elapsed = Duration::ZERO;
        let mut scratch_elapsed = Duration::ZERO;
        let mut reference_calls = 0usize;
        let mut scratch_calls = 0usize;
        'samples: for _ in 0..SAMPLES {
            let started = Instant::now();
            for _ in 0..CALLS_PER_SAMPLE {
                if Instant::now() >= deadline {
                    break 'samples;
                }
                black_box(
                    try_emd_pre_scratch_reference(
                        black_box(&left),
                        black_box(&right),
                        black_box(&ground),
                        DIMENSION,
                    )
                    .expect("reference benchmark"),
                );
                reference_calls += 1;
            }
            reference_elapsed += started.elapsed();

            let started = Instant::now();
            for _ in 0..CALLS_PER_SAMPLE {
                if Instant::now() >= deadline {
                    break 'samples;
                }
                black_box(
                    try_emd_with_scratch(
                        black_box(&left),
                        black_box(&right),
                        black_box(&ground),
                        DIMENSION,
                        &mut scratch,
                    )
                    .expect("scratch benchmark"),
                );
                scratch_calls += 1;
            }
            scratch_elapsed += started.elapsed();
        }
        eprintln!(
            "emd-scratch-microbenchmark dimension={DIMENSION} reference_calls={reference_calls} reference_seconds={:.9} reference_ns_per_call={:.3} scratch_calls={scratch_calls} scratch_seconds={:.9} scratch_ns_per_call={:.3} timeout_seconds={}",
            reference_elapsed.as_secs_f64(),
            reference_elapsed.as_nanos() as f64 / reference_calls.max(1) as f64,
            scratch_elapsed.as_secs_f64(),
            scratch_elapsed.as_nanos() as f64 / scratch_calls.max(1) as f64,
            timeout.as_secs(),
        );
        assert!(
            reference_calls > 0 && scratch_calls > 0,
            "timeout before measurement"
        );
    }

    #[test]
    fn emd_matches_independent_discrete_transport_reference() {
        let ground = vec![1.0, 2.0, 2.0, 100.0];
        let expected = brute_force_transport_cost(&[1, 1], &[1, 1], &ground, 2);
        let actual =
            try_emd(&[0.5, 0.5], &[0.5, 0.5], &ground, 2).expect("valid transport problem");
        assert!(
            (actual - expected as f32).abs() < 1e-6,
            "actual={actual}, expected={expected}"
        );
    }

    fn equity_ground(coordinates: &[f32]) -> Vec<f32> {
        coordinates
            .iter()
            .flat_map(|&left| coordinates.iter().map(move |&right| (left - right).abs()))
            .collect()
    }

    #[test]
    fn wasserstein_1d_is_zero_for_identical_distributions() {
        let coordinates = [0.1, 0.5, 0.9];
        assert_eq!(
            wasserstein_1d(&[0.2, 0.3, 0.5], &[0.2, 0.3, 0.5], &coordinates),
            0.0
        );
    }

    #[test]
    fn wasserstein_1d_charges_equity_difference_for_unit_mass() {
        let coordinates = [0.8, 0.2, 0.5];
        assert!(
            (wasserstein_1d(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0], &coordinates) - 0.6).abs() < 1e-6
        );
    }

    #[test]
    fn wasserstein_1d_is_symmetric_and_ignores_zero_components() {
        let coordinates = [0.7, 0.0, 0.4, 1.0];
        let left = [0.0, 0.25, 0.75, 0.0];
        let right = [0.5, 0.0, 0.0, 0.5];
        let forward = wasserstein_1d(&left, &right, &coordinates);
        let backward = wasserstein_1d(&right, &left, &coordinates);
        assert!((forward - backward).abs() < 1e-6);
        assert!(forward.is_finite());
    }

    #[test]
    fn wasserstein_1d_handles_uneven_intervals_and_equal_coordinates() {
        let coordinates = [0.7, 0.0, 0.0, 1.0];
        let left = [0.2, 0.3, 0.1, 0.4];
        let right = [0.1, 0.0, 0.4, 0.5];
        let expected = emd(
            &left,
            &right,
            &equity_ground(&coordinates),
            coordinates.len(),
        );
        let actual = wasserstein_1d(&left, &right, &coordinates);
        assert!(
            (actual - expected).abs() < 5e-6,
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn wasserstein_1d_matches_general_emd_for_normalized_distributions() {
        let coordinates = [0.91, 0.12, 0.55, 0.31, 0.77];
        let cases = [
            ([0.05, 0.15, 0.2, 0.25, 0.35], [0.4, 0.1, 0.2, 0.1, 0.2]),
            ([0.2, 0.2, 0.2, 0.2, 0.2], [0.0, 0.3, 0.1, 0.4, 0.2]),
        ];
        let ground = equity_ground(&coordinates);
        for (left, right) in cases {
            let expected = emd(&left, &right, &ground, coordinates.len());
            let actual = wasserstein_1d(&left, &right, &coordinates);
            assert!(
                (actual - expected).abs() < 5e-6,
                "actual={actual}, expected={expected}"
            );
        }
    }

    #[test]
    fn potential_turn_features_are_normalized_river_bucket_distributions() {
        let deck = deck(5);
        let river = RiverTable::build(&deck);
        let river_assignment: Vec<u16> = (0..river.keys.len()).map(|i| (i % 3) as u16).collect();
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_assignment, 3);
        assert_eq!(turn.dimension, 3);
        assert_eq!(turn.distribution.len(), turn.keys.len() * 3);
        for row in turn.distribution.chunks_exact(3) {
            assert!((row.iter().sum::<f32>() - 1.0).abs() < 1e-6, "{row:?}");
        }
    }

    #[test]
    fn potential_flop_features_are_normalized_turn_bucket_distributions() {
        let deck = deck(5);
        let river = RiverTable::build(&deck);
        let river_assignment: Vec<u16> = (0..river.keys.len()).map(|i| (i % 3) as u16).collect();
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_assignment, 3);
        let turn_assignment: Vec<u16> = (0..turn.keys.len()).map(|i| (i % 4) as u16).collect();
        let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_assignment, 4);
        assert_eq!(flop.dimension, 4);
        for row in flop.distribution.chunks_exact(4) {
            assert!((row.iter().sum::<f32>() - 1.0).abs() < 1e-6, "{row:?}");
        }
    }

    /// Один согласованный test-only probe продолжает turn после нового
    /// проверяемого checkpoint шага 28 для `ranks=7, K=10`.
    #[test]
    #[ignore = "ручной test-only continuation turn ranks=7; запускать с --ignored --nocapture под общим timeout 300 с"]
    fn test_only_ranks7_turn_k10_continues_after_verified_step_28() {
        use std::io::Write;

        const K: usize = 10;
        const SEED: u64 = 202_610_050_003;
        const CONTROL_TURN_OBJECTIVES: [f64; 28] = [
            0.093_878_107_531_247_96,
            0.069_619_094_619_986_57,
            0.066_461_444_756_355,
            0.065_835_569_059_757_3,
            0.064_993_761_144_054_38,
            0.064_219_256_308_054_95,
            0.063_890_307_745_594_61,
            0.063_956_841_499_195_64,
            0.063_920_528_245_297_27,
            0.063_861_846_133_701_18,
            0.063_772_262_941_226,
            0.063_619_773_161_908_26,
            0.063_557_347_155_343_34,
            0.063_541_175_612_322_13,
            0.063_443_046_939_590_09,
            0.063_327_102_121_507_26,
            0.063_217_400_356_864_03,
            0.063_063_193_480_681_87,
            0.062_966_368_040_303_13,
            0.062_887_448_183_352_66,
            0.062_763_998_038_317_99,
            0.062_641_214_205_662_13,
            0.062_617_736_939_631_93,
            0.062_619_913_559_020_3,
            0.062_627_927_249_839_41,
            0.062_623_934_115_557_54,
            0.062_617_296_211_079_56,
            0.062_618_406_466_629_56,
        ];
        let trace_path = std::path::PathBuf::from(
            std::env::var("HU20_R7_TURN_PROBE_TRACE_PATH")
                .expect("HU20_R7_TURN_PROBE_TRACE_PATH is required"),
        );
        let checkpoint_directory = std::path::PathBuf::from(
            std::env::var("HU20_R7_TURN_PROBE_CHECKPOINT_DIR")
                .expect("HU20_R7_TURN_PROBE_CHECKPOINT_DIR is required"),
        );
        assert!(
            checkpoint_directory.is_dir(),
            "checkpoint directory must exist"
        );
        assert!(
            std::fs::read_dir(&checkpoint_directory)
                .expect("read checkpoint directory")
                .next()
                .is_none(),
            "checkpoint directory must be empty"
        );
        assert!(!trace_path.exists(), "trace path must be new");
        let mut trace = std::fs::File::create(trace_path).expect("create turn trace");

        let deck = deck(7);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let mut river_ground = vec![0.0; K * K];
        for left in 0..K {
            for right in 0..K {
                river_ground[left * K + right] =
                    (river_clusters.centroids[left] - river_clusters.centroids[right]).abs();
            }
        }
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);
        let mut stepper =
            TestOnlyEmdStepper::new(&turn.distribution, K, &turn.weight, &river_ground, K, SEED);
        let mut seen = std::collections::HashMap::new();
        for iteration in 1..=28 {
            let diagnostic = stepper
                .step()
                .expect("valid production-order prefix step")
                .expect("prefix must return a diagnostic");
            assert_eq!(
                stepper.objective[iteration - 1],
                CONTROL_TURN_OBJECTIVES[iteration - 1],
                "turn objective mismatch at iteration {iteration}"
            );
            assert_eq!(
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic),
                None,
                "prefix repeated a complete algorithmic state"
            );
        }
        assert!(
            stepper.stabilized,
            "step 28 must hit the historical early threshold"
        );
        assert_eq!(stepper.objective, CONTROL_TURN_OBJECTIVES);
        let state_before_diagnostic = stepper.state_words();
        let step_28_diagnostic = stepper
            .final_assignment_diagnostic()
            .expect("valid step-28 final reassignment");
        assert_eq!(state_before_diagnostic, stepper.state_words());
        assert_eq!(
            format!("{:.9}", step_28_diagnostic.changed_weight_fraction),
            "0.000019111"
        );
        assert_eq!(
            format!("{:.9}", step_28_diagnostic.objective),
            "0.062617997"
        );
        writeln!(
            trace,
            "final iteration=28 objective={:.17} changed_weight_fraction={:.17}",
            step_28_diagnostic.objective, step_28_diagnostic.changed_weight_fraction,
        )
        .expect("write step-28 final diagnostic");
        trace.flush().expect("flush step-28 final diagnostic");
        test_only_write_checkpoint(&checkpoint_directory, 28, &stepper);
        let resumed = test_only_resume_from_checkpoint(
            &checkpoint_directory.join("checkpoint-28.bin"),
            &turn.distribution,
            K,
            &turn.weight,
            &river_ground,
            K,
        )
        .expect("load verified new step-28 checkpoint");
        assert_eq!(resumed.state_words(), stepper.state_words());
        assert_eq!(resumed.objective, stepper.objective);

        let mut terminal_reason = "budget";
        for iteration in 29..=50 {
            let diagnostic = stepper
                .step_without_early_stop()
                .expect("valid test-only continuation step")
                .expect("test-only continuation must not stop early");
            test_only_write_checkpoint(&checkpoint_directory, iteration, &stepper);
            let state_before_diagnostic = stepper.state_words();
            let final_diagnostic = stepper
                .final_assignment_diagnostic()
                .expect("valid continuation final reassignment");
            assert_eq!(state_before_diagnostic, stepper.state_words());
            writeln!(
                trace,
                "final iteration={iteration} objective={:.17} changed_weight_fraction={:.17}",
                final_diagnostic.objective, final_diagnostic.changed_weight_fraction,
            )
            .expect("write continuation final diagnostic");
            trace.flush().expect("flush continuation final diagnostic");
            if let Some((first_seen_at, repeated_at)) =
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic)
            {
                terminal_reason = "repeated-state";
                writeln!(trace, "terminal reason={terminal_reason} first={first_seen_at} repeated={repeated_at}")
                    .expect("write repeated state");
                break;
            }
            if final_diagnostic.changed_weight_fraction == 0.0 {
                terminal_reason = "zero-final-assignment-change";
                writeln!(
                    trace,
                    "terminal reason={terminal_reason} iteration={iteration}"
                )
                .expect("write zero diagnostic");
                break;
            }
        }
        writeln!(
            trace,
            "complete iterations={} terminal_reason={terminal_reason}",
            stepper.diagnostics.len(),
        )
        .expect("write probe completion");
        trace.flush().expect("flush probe completion");
    }

    /// Одноразовый test-only probe для контроля `ranks=5, K=20`.
    ///
    /// Не вызывается production-кодом и не меняет сохраняемые таблицы: повторно
    /// строит только river/turn, сверяет первые 10 итераций с зафиксированным
    /// контролем, затем наблюдает ту же детерминированную траекторию до 50.
    #[test]
    #[ignore = "ручное test-only расследование траектории turn; запускать с --ignored --nocapture"]
    fn test_only_ranks5_turn_k20_trajectory_to_fifty_iterations() {
        const K: usize = 20;
        const SEED: u64 = 202_610_050_003;
        const FIRST_CONTROL_TURN_OBJECTIVE: f64 = 0.033_152_026_385_421_3;
        const FIRST_CONTROL_FINAL_OBJECTIVE: f64 = 0.032_777_321;
        const FIRST_CONTROL_CHANGED_WEIGHT: f64 = 0.008_957_688;

        let deck = deck(5);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);

        let ten_iterations = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            10,
            true,
        );
        assert_eq!(ten_iterations.diagnostics.len(), 10);
        assert!(
            (ten_iterations
                .objective
                .last()
                .copied()
                .expect("10 objectives")
                - FIRST_CONTROL_TURN_OBJECTIVE)
                .abs()
                < 1e-12
        );
        let ten_diagnostic = ten_iterations
            .final_assignment_diagnostic
            .expect("requested final turn diagnostic");
        assert!((ten_diagnostic.objective - FIRST_CONTROL_FINAL_OBJECTIVE).abs() < 1e-8);
        assert!(
            (ten_diagnostic.changed_weight_fraction - FIRST_CONTROL_CHANGED_WEIGHT).abs() < 1e-8
        );

        let fifty_iterations = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            50,
            true,
        );
        for (iteration, diagnostic) in fifty_iterations.diagnostics.iter().enumerate() {
            println!(
                "turn-trajectory iteration={} objective_before={:.9} objective_after={:.9} moved_weight_fraction={:.9} occupied_buckets={}",
                iteration + 1,
                diagnostic.objective_before_update,
                diagnostic.objective_after_update,
                diagnostic.moved_weight_fraction,
                diagnostic.occupied_buckets,
            );
        }
        let fifty_diagnostic = fifty_iterations
            .final_assignment_diagnostic
            .expect("requested final turn diagnostic");
        assert_eq!(fifty_iterations.diagnostics.len(), 17);
        assert_eq!(fifty_diagnostic.changed_weight_fraction, 0.0);
        assert!((fifty_diagnostic.objective - 0.032_710_700).abs() < 1e-8);
        println!(
            "turn-trajectory final changed_weight_fraction={:.9} objective={:.9}",
            fifty_diagnostic.changed_weight_fraction, fifty_diagnostic.objective,
        );

        let mut seen = std::collections::HashMap::<Vec<u32>, usize>::new();
        let mut repeated_state = None;
        for iteration_limit in 1..=50 {
            let clustering = kmeans_wasserstein_1d_with_diagnostics(
                &turn.distribution,
                &turn.weight,
                &river_clusters.centroids,
                K,
                SEED,
                iteration_limit,
                false,
            );
            let state = clustering
                .assignment
                .iter()
                .map(|&assignment| assignment as u32)
                .chain(clustering.centroids.iter().map(|value| value.to_bits()))
                .collect::<Vec<_>>();
            if let Some(first_seen_at) = seen.insert(state, iteration_limit) {
                repeated_state = Some((first_seen_at, iteration_limit));
                break;
            }
        }
        match repeated_state {
            Some((first_seen_at, repeated_at)) if repeated_at == first_seen_at + 1 => println!(
                "turn-trajectory stabilization first_seen_at={} repeated_at={}",
                first_seen_at, repeated_at
            ),
            Some((first_seen_at, repeated_at)) => println!(
                "turn-trajectory cycle first_seen_at={} repeated_at={}",
                first_seen_at, repeated_at
            ),
            None => println!("turn-trajectory budget_exhausted iterations=50"),
        }
        assert_eq!(repeated_state, Some((16, 17)));
    }

    /// Изолированное расследование траектории flop для `ranks=6, K=20`.
    ///
    /// Probe намеренно воспроизводит весь зависимый префикс, но не вызывает
    /// launcher и не записывает таблицы. Сверяет контрольные 10 итераций;
    /// continuation допустим только через отдельный test-only stepper,
    /// чтобы не пересчитывать этот префикс повторно в пределах одного лимита.
    #[test]
    #[ignore = "ручная test-only сверка flop-контроля; запускать с --ignored --nocapture под timeout 300 с"]
    fn test_only_ranks6_flop_k20_reproduces_ten_iteration_control() {
        const K: usize = 20;
        const SEED: u64 = 202_610_050_003;
        const CONTROL_FLOP_OBJECTIVES: [f64; 10] = [
            0.052_218_402_081_649_31,
            0.048_975_848_067_693_31,
            0.047_125_931_493_033_9,
            0.045_908_284_084_046_23,
            0.044_481_157_472_547_085,
            0.043_480_023_156_214_17,
            0.042_986_778_310_103_405,
            0.043_031_078_124_860_24,
            0.043_051_403_845_207_614,
            0.043_068_505_771_261_47,
        ];
        const CONTROL_FINAL_OBJECTIVE: f64 = 0.043_054_945;
        const CONTROL_CHANGED_WEIGHT: f64 = 0.005_881_799;

        let deck = deck(6);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let river_ground = equity_ground(&river_clusters.centroids);
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);
        let turn_clusters = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            50,
            true,
        );
        assert_eq!(turn_clusters.diagnostics.len(), 21);
        assert_eq!(
            turn_clusters
                .final_assignment_diagnostic
                .as_ref()
                .expect("requested final turn diagnostic")
                .changed_weight_fraction,
            0.0
        );
        let turn_ground = centroid_emd_matrix(&turn_clusters.centroids, K, &river_ground)
            .expect("valid reproduced turn ground matrix");
        let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_clusters.assignment, K);

        let ten_iterations = kmeans_emd_with_diagnostics(
            &flop.distribution,
            K,
            &flop.weight,
            &turn_ground,
            K,
            SEED,
            10,
            None,
            true,
        )
        .expect("valid reproduced 10-iteration flop clustering");
        assert_eq!(
            ten_iterations.objective.len(),
            CONTROL_FLOP_OBJECTIVES.len()
        );
        for (iteration, (&actual, &expected)) in ten_iterations
            .objective
            .iter()
            .zip(CONTROL_FLOP_OBJECTIVES.iter())
            .enumerate()
        {
            assert!(
                (actual - expected).abs() < 1e-12,
                "control mismatch at flop iteration {}: actual={actual:.15}, expected={expected:.15}",
                iteration + 1,
            );
        }
        let ten_diagnostic = ten_iterations
            .final_assignment_diagnostic
            .expect("requested final flop diagnostic");
        assert!((ten_diagnostic.objective - CONTROL_FINAL_OBJECTIVE).abs() < 1e-8);
        assert!((ten_diagnostic.changed_weight_fraction - CONTROL_CHANGED_WEIGHT).abs() < 1e-8);
        println!(
            "flop-trajectory control iterations=10 final_objective={:.9} changed_weight_fraction={:.9}",
            ten_diagnostic.objective, ten_diagnostic.changed_weight_fraction,
        );
    }

    /// Единственный release-probe продолжает flop после 10-го update без
    /// повторной оплаты первых 10 итераций; путь trace задаётся окружением.
    #[test]
    #[ignore = "ручное test-only расследование траектории flop; запускать с --ignored --nocapture под timeout 300 с"]
    fn test_only_ranks6_flop_k20_stateful_trajectory_to_twenty_iterations() {
        use std::io::Write;

        const K: usize = 20;
        const SEED: u64 = 202_610_050_003;
        const CONTROL_FLOP_OBJECTIVES: [f64; 10] = [
            0.052_218_402_081_649_31,
            0.048_975_848_067_693_31,
            0.047_125_931_493_033_9,
            0.045_908_284_084_046_23,
            0.044_481_157_472_547_085,
            0.043_480_023_156_214_17,
            0.042_986_778_310_103_405,
            0.043_031_078_124_860_24,
            0.043_051_403_845_207_614,
            0.043_068_505_771_261_47,
        ];
        const CONTROL_FINAL_OBJECTIVE: f64 = 0.043_054_945;
        const CONTROL_CHANGED_WEIGHT: f64 = 0.005_881_799;

        let trace_path = std::env::var("HU20_FLOP_PROBE_TRACE_PATH")
            .expect("HU20_FLOP_PROBE_TRACE_PATH is required for the test-only probe");
        let checkpoint_directory = std::path::PathBuf::from(
            std::env::var("HU20_FLOP_PROBE_CHECKPOINT_DIR")
                .expect("HU20_FLOP_PROBE_CHECKPOINT_DIR is required for the test-only probe"),
        );
        assert!(
            checkpoint_directory.is_dir(),
            "checkpoint directory must exist"
        );
        assert!(
            std::fs::read_dir(&checkpoint_directory)
                .expect("read checkpoint directory")
                .next()
                .is_none(),
            "checkpoint directory must be empty"
        );
        let previous_metrics = test_only_load_trace_metrics(&std::path::PathBuf::from(
            std::env::var("HU20_FLOP_PREVIOUS_TRACE_PATH")
                .expect("HU20_FLOP_PREVIOUS_TRACE_PATH is required for the test-only probe"),
        ))
        .expect("load previous flop trajectory trace");
        for iteration in 11..=18 {
            assert!(
                previous_metrics.contains_key(&iteration),
                "previous trace lacks iteration {iteration}"
            );
        }
        let mut trace = std::fs::File::create(trace_path).expect("create flop probe trace");
        let deck = deck(6);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let river_ground = equity_ground(&river_clusters.centroids);
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);
        let turn_clusters = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            50,
            true,
        );
        assert_eq!(turn_clusters.diagnostics.len(), 21);
        assert_eq!(
            turn_clusters
                .final_assignment_diagnostic
                .as_ref()
                .expect("requested final turn diagnostic")
                .changed_weight_fraction,
            0.0
        );
        let turn_ground = centroid_emd_matrix(&turn_clusters.centroids, K, &river_ground)
            .expect("valid reproduced turn ground matrix");
        let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_clusters.assignment, K);
        let mut stepper =
            TestOnlyEmdStepper::new(&flop.distribution, K, &flop.weight, &turn_ground, K, SEED);
        let mut seen = std::collections::HashMap::new();
        for iteration in 1..=10 {
            let diagnostic = stepper
                .step()
                .expect("valid control step")
                .expect("control must not stabilize before iteration 10");
            test_only_write_checkpoint(&checkpoint_directory, iteration, &stepper);
            assert_eq!(
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic),
                None
            );
        }
        for (&actual, &expected) in stepper.objective.iter().zip(CONTROL_FLOP_OBJECTIVES.iter()) {
            assert!(
                (actual - expected).abs() < 1e-12,
                "actual={actual:.15}, expected={expected:.15}"
            );
        }
        let state_before_control_diagnostic = stepper.state_words();
        let control_diagnostic = stepper
            .final_assignment_diagnostic()
            .expect("valid control final reassignment");
        assert_eq!(state_before_control_diagnostic, stepper.state_words());
        assert!((control_diagnostic.objective - CONTROL_FINAL_OBJECTIVE).abs() < 1e-8);
        assert!((control_diagnostic.changed_weight_fraction - CONTROL_CHANGED_WEIGHT).abs() < 1e-8);
        writeln!(
            trace,
            "control_final_diagnostic objective={:.9} changed_weight_fraction={:.9}",
            control_diagnostic.objective, control_diagnostic.changed_weight_fraction,
        )
        .expect("write control diagnostic");
        trace.flush().expect("flush control diagnostic");

        let mut repeated_state = None;
        for iteration in 11..=20 {
            let Some(diagnostic) = stepper.step().expect("valid continuation step") else {
                break;
            };
            test_only_write_checkpoint(&checkpoint_directory, iteration, &stepper);
            if let Some(&(objective_before, objective_after, moved_weight_fraction)) =
                previous_metrics.get(&iteration)
            {
                assert!(
                    (diagnostic.objective_before_update - objective_before).abs() <= 5e-10
                        && (diagnostic.objective_after_update - objective_after).abs() <= 5e-10
                        && (diagnostic.moved_weight_fraction - moved_weight_fraction).abs()
                            <= 5e-10,
                    "previous trace metrics differ at iteration {iteration}"
                );
            }
            repeated_state =
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic);
            if repeated_state.is_some() {
                break;
            }
        }
        let final_diagnostic = stepper
            .final_assignment_diagnostic()
            .expect("valid final reassignment");
        writeln!(
            trace,
            "final iterations={} changed_weight_fraction={:.9} objective={:.9} repeated_state={repeated_state:?}",
            stepper.diagnostics.len(),
            final_diagnostic.changed_weight_fraction,
            final_diagnostic.objective,
        )
        .expect("write final diagnostic");
        trace.flush().expect("flush final diagnostic");
    }

    #[test]
    #[ignore = "ручное test-only возобновление flop из checkpoint; запускать с --ignored --nocapture под timeout 300 с"]
    fn test_only_ranks6_flop_k20_resume_from_checkpoint_to_twenty_iterations() {
        use std::io::Write;
        use std::time::Instant;

        const K: usize = 20;
        const SEED: u64 = 202_610_050_003;
        const CHECKPOINT_STEP: usize = 18;
        const INPUT_FINGERPRINT: u64 = 0xDF7E_8949_91B2_639B;
        let checkpoint_path =
            std::path::PathBuf::from(std::env::var("HU20_FLOP_RESUME_CHECKPOINT_PATH").expect(
                "HU20_FLOP_RESUME_CHECKPOINT_PATH is required for the test-only resume probe",
            ));
        let checkpoint_directory =
            std::path::PathBuf::from(std::env::var("HU20_FLOP_RESUME_CHECKPOINT_DIR").expect(
                "HU20_FLOP_RESUME_CHECKPOINT_DIR is required for the test-only resume probe",
            ));
        let trace_path = std::path::PathBuf::from(
            std::env::var("HU20_FLOP_RESUME_TRACE_PATH")
                .expect("HU20_FLOP_RESUME_TRACE_PATH is required for the test-only resume probe"),
        );
        assert!(
            checkpoint_directory.is_dir(),
            "checkpoint directory must exist"
        );
        assert!(
            std::fs::read_dir(&checkpoint_directory)
                .expect("read checkpoint directory")
                .next()
                .is_none(),
            "checkpoint directory must be empty"
        );
        assert!(!trace_path.exists(), "resume trace must not exist");
        let header =
            TestOnlyEmdCheckpoint::load_verified(&checkpoint_path, K, 21_150, K, INPUT_FINGERPRINT)
                .expect("compatible saved checkpoint before input recovery");
        assert_eq!(header.diagnostics.len(), CHECKPOINT_STEP);
        assert!(!header.stabilized, "checkpoint must require continuation");

        let recovery_started = Instant::now();
        let deck = deck(6);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let river_ground = equity_ground(&river_clusters.centroids);
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);
        let turn_clusters = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            50,
            true,
        );
        assert_eq!(turn_clusters.diagnostics.len(), 21);
        let turn_ground = centroid_emd_matrix(&turn_clusters.centroids, K, &river_ground)
            .expect("valid reproduced turn ground matrix");
        let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_clusters.assignment, K);
        let mut stepper = test_only_resume_from_checkpoint(
            &checkpoint_path,
            &flop.distribution,
            K,
            &flop.weight,
            &turn_ground,
            K,
        )
        .expect("checkpoint must match reconstructed exact inputs");
        let recovery_seconds = recovery_started.elapsed().as_secs_f64();
        assert_eq!(stepper.diagnostics.len(), CHECKPOINT_STEP);
        assert_eq!(stepper.objective.len(), CHECKPOINT_STEP);
        assert!(!stepper.stabilized, "checkpoint must require continuation");

        let mut trace = std::fs::File::create(&trace_path).expect("create resume probe trace");
        writeln!(
            trace,
            "input_recovery_seconds={recovery_seconds:.3} checkpoint_step={CHECKPOINT_STEP} state_history=checkpoint-18-only"
        )
        .expect("write recovery metadata");
        trace.flush().expect("flush recovery metadata");
        let mut seen = std::collections::HashMap::new();
        seen.insert(stepper.state_words(), CHECKPOINT_STEP);
        let mut repeated_state = None;
        for iteration in (CHECKPOINT_STEP + 1)..=20 {
            let Some(diagnostic) = stepper.step().expect("valid resumed continuation step") else {
                break;
            };
            test_only_write_checkpoint(&checkpoint_directory, iteration, &stepper);
            repeated_state =
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic);
            if repeated_state.is_some() {
                break;
            }
        }
        let state_before_final_diagnostic = stepper.state_words();
        let final_diagnostic = stepper
            .final_assignment_diagnostic()
            .expect("valid final reassignment");
        assert_eq!(state_before_final_diagnostic, stepper.state_words());
        writeln!(
            trace,
            "final iterations={} changed_weight_fraction={:.9} objective={:.9} repeated_state={repeated_state:?}",
            stepper.diagnostics.len(),
            final_diagnostic.changed_weight_fraction,
            final_diagnostic.objective,
        )
        .expect("write final resume diagnostic");
        trace.flush().expect("flush final resume diagnostic");
    }

    #[test]
    #[ignore = "ручное test-only возобновление flop из checkpoint-20; запускать с --ignored --nocapture под timeout 300 с"]
    fn test_only_ranks6_flop_k20_resume_from_checkpoint_twenty_to_thirty_iterations() {
        use std::io::Write;
        use std::time::Instant;

        const K: usize = 20;
        const SEED: u64 = 202_610_050_003;
        const CHECKPOINT_STEP: usize = 20;
        const INPUT_FINGERPRINT: u64 = 0xDF7E_8949_91B2_639B;
        let checkpoint_path =
            std::path::PathBuf::from(std::env::var("HU20_FLOP_RESUME_CHECKPOINT_PATH").expect(
                "HU20_FLOP_RESUME_CHECKPOINT_PATH is required for the test-only resume probe",
            ));
        let history_paths = [18, 19, 20].map(|iteration| {
            std::path::PathBuf::from(
                std::env::var(format!("HU20_FLOP_RESUME_HISTORY_{iteration}_PATH"))
                    .unwrap_or_else(|_| checkpoint_path.to_string_lossy().into_owned()),
            )
        });
        let checkpoint_directory =
            std::path::PathBuf::from(std::env::var("HU20_FLOP_RESUME_CHECKPOINT_DIR").expect(
                "HU20_FLOP_RESUME_CHECKPOINT_DIR is required for the test-only resume probe",
            ));
        let trace_path = std::path::PathBuf::from(
            std::env::var("HU20_FLOP_RESUME_TRACE_PATH")
                .expect("HU20_FLOP_RESUME_TRACE_PATH is required for the test-only resume probe"),
        );
        assert!(
            checkpoint_directory.is_dir(),
            "checkpoint directory must exist"
        );
        assert!(
            std::fs::read_dir(&checkpoint_directory)
                .expect("read checkpoint directory")
                .next()
                .is_none(),
            "checkpoint directory must be empty"
        );
        assert!(!trace_path.exists(), "resume trace must not exist");
        for (&iteration, path) in [18, 19, 20].iter().zip(&history_paths) {
            let header =
                TestOnlyEmdCheckpoint::load_verified(path, K, 21_150, K, INPUT_FINGERPRINT)
                    .expect("compatible history checkpoint before input recovery");
            assert_eq!(header.diagnostics.len(), iteration);
        }

        let recovery_started = Instant::now();
        let deck = deck(6);
        let river = RiverTable::build(&deck);
        let river_clusters = kmeans(&river.equity, 1, &river.weight, K, SEED, 10);
        let river_ground = equity_ground(&river_clusters.centroids);
        let turn = PotentialFeatures::build(&deck, 4, &river.keys, &river_clusters.assignment, K);
        let turn_clusters = kmeans_wasserstein_1d_with_diagnostics(
            &turn.distribution,
            &turn.weight,
            &river_clusters.centroids,
            K,
            SEED,
            50,
            true,
        );
        assert_eq!(turn_clusters.diagnostics.len(), 21);
        let turn_ground = centroid_emd_matrix(&turn_clusters.centroids, K, &river_ground)
            .expect("valid reproduced turn ground matrix");
        let flop = PotentialFeatures::build(&deck, 3, &turn.keys, &turn_clusters.assignment, K);
        let history_entries = [
            (18, history_paths[0].as_path()),
            (19, history_paths[1].as_path()),
            (20, history_paths[2].as_path()),
        ];
        let mut seen = test_only_load_checkpoint_history(
            &history_entries,
            &flop.distribution,
            K,
            &flop.weight,
            &turn_ground,
            K,
        )
        .expect("full checkpoint history must not contain a cycle");
        let mut stepper = test_only_resume_from_checkpoint(
            &checkpoint_path,
            &flop.distribution,
            K,
            &flop.weight,
            &turn_ground,
            K,
        )
        .expect("checkpoint must match reconstructed exact inputs");
        let recovery_seconds = recovery_started.elapsed().as_secs_f64();
        assert_eq!(stepper.diagnostics.len(), CHECKPOINT_STEP);
        assert!(!stepper.stabilized, "checkpoint must require continuation");

        let mut trace = std::fs::File::create(&trace_path).expect("create resume probe trace");
        writeln!(
            trace,
            "input_recovery_seconds={recovery_seconds:.3} checkpoint_step={CHECKPOINT_STEP} state_history=checkpoints-18-20"
        )
        .expect("write recovery metadata");
        trace.flush().expect("flush recovery metadata");
        let mut repeated_state = None;
        for iteration in (CHECKPOINT_STEP + 1)..=30 {
            let Some(diagnostic) = stepper.step().expect("valid resumed continuation step") else {
                break;
            };
            test_only_write_checkpoint(&checkpoint_directory, iteration, &stepper);
            repeated_state =
                test_only_record_step(&mut trace, &mut seen, iteration, &stepper, &diagnostic);
            if repeated_state.is_some() {
                break;
            }
        }
        let state_before_final_diagnostic = stepper.state_words();
        let final_diagnostic = stepper
            .final_assignment_diagnostic()
            .expect("valid final reassignment");
        assert_eq!(state_before_final_diagnostic, stepper.state_words());
        writeln!(
            trace,
            "final iterations={} changed_weight_fraction={:.9} objective={:.9} repeated_state={repeated_state:?}",
            stepper.diagnostics.len(),
            final_diagnostic.changed_weight_fraction,
            final_diagnostic.objective,
        )
        .expect("write final resume diagnostic");
        trace.flush().expect("flush final resume diagnostic");
    }

    #[test]
    fn test_only_emd_stepper_matches_continuous_and_resumed_trajectory() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.2, 0.8, 0.0, 1.0];
        let weights = vec![3, 5, 7, 11];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let expected =
            kmeans_emd_with_diagnostics(&points, 2, &weights, &ground, 2, 17, 5, None, false)
                .expect("valid uninterrupted clustering");

        let mut continuous = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        for _ in 0..5 {
            continuous.step().expect("valid continuous step");
        }

        let mut split = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        for _ in 0..2 {
            split.step().expect("valid prefix step");
        }
        let snapshot = split.snapshot();
        let mut resumed =
            TestOnlyEmdStepper::from_snapshot(&points, 2, &weights, &ground, 2, snapshot);
        for _ in 0..3 {
            resumed.step().expect("valid resumed step");
        }

        assert_eq!(continuous.assignment, expected.assignment);
        assert_eq!(continuous.assignment, resumed.assignment);
        assert_eq!(
            continuous
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            continuous
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            resumed
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(continuous.objective.len(), expected.objective.len());
        for ((actual, expected), resumed) in continuous
            .objective
            .iter()
            .zip(&expected.objective)
            .zip(&resumed.objective)
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
            assert_eq!(actual.to_bits(), resumed.to_bits());
        }
        assert_eq!(continuous.diagnostics.len(), expected.diagnostics.len());
        for (actual, expected) in continuous.diagnostics.iter().zip(&expected.diagnostics) {
            assert_eq!(
                actual.objective_before_update.to_bits(),
                expected.objective_before_update.to_bits()
            );
            assert_eq!(
                actual.objective_after_update.to_bits(),
                expected.objective_after_update.to_bits()
            );
            assert_eq!(
                actual.objective_after_reassignment.map(f64::to_bits),
                expected.objective_after_reassignment.map(f64::to_bits)
            );
            assert_eq!(
                actual.moved_weight_fraction.to_bits(),
                expected.moved_weight_fraction.to_bits()
            );
            assert_eq!(actual.occupied_buckets, expected.occupied_buckets);
        }
    }

    #[test]
    fn test_only_emd_checkpoint_restores_exact_state_and_rejects_other_inputs() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.2, 0.8, 0.0, 1.0];
        let weights = vec![3, 5, 7, 11];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let path = std::env::temp_dir().join(format!(
            "hu20-test-only-stepper-checkpoint-{}.bin",
            std::process::id()
        ));

        let mut continuous = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        for _ in 0..5 {
            continuous.step().expect("valid continuous step");
        }

        let mut interrupted = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        for _ in 0..2 {
            interrupted.step().expect("valid checkpoint prefix");
        }
        TestOnlyEmdCheckpoint::write_atomic(&path, &interrupted).expect("write checkpoint");
        let snapshot = TestOnlyEmdCheckpoint::load(&path, &points, 2, &weights, &ground, 2)
            .expect("load compatible checkpoint");
        let mut resumed =
            TestOnlyEmdStepper::from_snapshot(&points, 2, &weights, &ground, 2, snapshot);
        for _ in 0..3 {
            resumed.step().expect("valid resumed step");
        }

        assert_eq!(continuous.assignment, resumed.assignment);
        assert_eq!(
            continuous
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            resumed
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            continuous
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            resumed
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        let incompatible_ground = vec![0.0, 2.0, 2.0, 0.0];
        assert!(
            TestOnlyEmdCheckpoint::load(&path, &points, 2, &weights, &incompatible_ground, 2)
                .is_err()
        );
        std::fs::remove_file(path).expect("remove checkpoint");
    }

    #[test]
    fn test_only_trace_parser_reads_iteration_metrics() {
        let path =
            std::env::temp_dir().join(format!("hu20-test-only-trace-{}.log", std::process::id()));
        std::fs::write(
            &path,
            "iteration=11 objective_before=0.043054945 objective_after=0.043071678 moved_weight_fraction=0.005881799 occupied_buckets=20 state_fingerprint=ABC\n",
        )
        .expect("write trace fixture");
        let parsed = test_only_load_trace_metrics(&path).expect("parse trace fixture");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[&11], (0.043054945, 0.043071678, 0.005881799));
        std::fs::remove_file(path).expect("remove trace fixture");
    }

    #[test]
    fn test_only_resume_from_checkpoint_restores_bitwise_state() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.2, 0.8, 0.0, 1.0];
        let weights = vec![3, 5, 7, 11];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let path =
            std::env::temp_dir().join(format!("hu20-test-only-resume-{}.bin", std::process::id()));
        let mut expected = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        for _ in 0..2 {
            expected.step().expect("valid checkpoint prefix");
        }
        TestOnlyEmdCheckpoint::write_atomic(&path, &expected).expect("write checkpoint");

        let resumed = test_only_resume_from_checkpoint(&path, &points, 2, &weights, &ground, 2)
            .expect("load compatible checkpoint");

        assert_eq!(resumed.assignment, expected.assignment);
        assert_eq!(
            resumed
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            resumed
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(resumed.diagnostics.len(), expected.diagnostics.len());
        for (actual, expected) in resumed.diagnostics.iter().zip(&expected.diagnostics) {
            assert_eq!(
                (
                    actual.objective_before_update.to_bits(),
                    actual.objective_after_update.to_bits(),
                    actual.objective_after_reassignment.map(f64::to_bits),
                    actual.moved_weight_fraction.to_bits(),
                    actual.occupied_buckets,
                ),
                (
                    expected.objective_before_update.to_bits(),
                    expected.objective_after_update.to_bits(),
                    expected.objective_after_reassignment.map(f64::to_bits),
                    expected.moved_weight_fraction.to_bits(),
                    expected.occupied_buckets,
                )
            );
        }
        std::fs::remove_file(path).expect("remove checkpoint");
    }

    #[test]
    fn test_only_checkpoint_history_uses_full_state_words() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.2, 0.8, 0.0, 1.0];
        let weights = vec![3, 5, 7, 11];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let directory =
            std::env::temp_dir().join(format!("hu20-test-only-history-{}", std::process::id()));
        std::fs::create_dir(&directory).expect("create checkpoint fixture directory");
        let first = directory.join("checkpoint-01.bin");
        let second = directory.join("checkpoint-02.bin");
        let mut stepper = TestOnlyEmdStepper::new(&points, 2, &weights, &ground, 2, 17);
        stepper.step().expect("valid first step");
        TestOnlyEmdCheckpoint::write_atomic(&first, &stepper).expect("write first checkpoint");
        let first_state = stepper.state_words();
        stepper.step().expect("valid second step");
        TestOnlyEmdCheckpoint::write_atomic(&second, &stepper).expect("write second checkpoint");

        let history = test_only_load_checkpoint_history(
            &[(1, first.as_path())],
            &points,
            2,
            &weights,
            &ground,
            2,
        )
        .expect("load full checkpoint history");

        assert_eq!(history.get(&first_state), Some(&1));
        let error = test_only_load_checkpoint_history(
            &[(1, first.as_path()), (2, second.as_path())],
            &points,
            2,
            &weights,
            &ground,
            2,
        )
        .expect_err("repeated full state must be a detected cycle");
        assert!(error.contains("повторяет полное состояние checkpoint 1"));
        std::fs::remove_dir_all(directory).expect("remove checkpoint fixture directory");
    }

    #[test]
    #[ignore = "ручная проверка сохранённого checkpoint без EMD-проходов"]
    fn test_only_load_saved_flop_checkpoint_without_emd() {
        let path = std::path::PathBuf::from(
            std::env::var("HU20_FLOP_CHECKPOINT_PATH")
                .expect("HU20_FLOP_CHECKPOINT_PATH is required"),
        );
        let checkpoint =
            TestOnlyEmdCheckpoint::load_verified(&path, 20, 21_150, 20, 0xDF7E_8949_91B2_639B)
                .expect("load compatible saved checkpoint");
        assert_eq!(checkpoint.diagnostics.len(), 18);
        assert_eq!(checkpoint.assignment.len(), 21_150);
        assert_eq!(checkpoint.centroids.len(), 400);
        assert_eq!(checkpoint.objective.len(), 18);
        assert!(!checkpoint.stabilized);
    }

    #[test]
    fn centroid_emd_matrix_uses_lower_street_metric() {
        let lower = vec![0.0, 2.0, 2.0, 0.0];
        let centroids = vec![1.0, 0.0, 0.0, 1.0];
        let matrix =
            centroid_emd_matrix(&centroids, 2, &lower).expect("valid centroid ground matrix");
        assert_eq!(matrix.len(), 4);
        assert!((matrix[1] - 2.0).abs() < 1e-6);
        assert!((matrix[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn profiled_centroid_emd_matrix_counts_unique_completed_comparisons() {
        let lower = vec![0.0, 2.0, 2.0, 0.0];
        let centroids = vec![1.0, 0.0, 0.0, 1.0, 0.5, 0.5];

        let profiled = centroid_emd_matrix_profiled(&centroids, 2, &lower)
            .expect("valid profiled centroid ground matrix");

        assert_eq!(profiled.emd_calls, 3);
        assert_eq!(profiled.matrix.len(), 9);
        assert!((profiled.matrix[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn emd_tolerates_f32_centroid_normalization_error() {
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let distance = emd(&[0.5, 0.5], &[0.499_999_97, 0.5], &ground, 2);
        assert!(distance.is_finite());
    }

    #[test]
    fn emd_rejects_non_finite_ground_distance() {
        let error = try_emd(&[1.0, 0.0], &[0.0, 1.0], &[0.0, f32::NAN, 1.0, 0.0], 2)
            .expect_err("non-finite ground distance must not produce a numeric EMD");
        assert!(matches!(
            error,
            EmdError::NonFiniteGroundDistance { index: 1 }
        ));
    }

    #[test]
    fn emd_rejects_zero_total_mass() {
        let error = try_emd(&[0.0, 0.0], &[1.0, 0.0], &[0.0, 1.0, 1.0, 0.0], 2)
            .expect_err("zero mass must not reach the transport loop");
        assert!(matches!(
            error,
            EmdError::NonPositiveTotalMass { side: "left" }
        ));
    }

    #[test]
    fn predecessor_reconstruction_rejects_cycle() {
        let previous = vec![None, Some((2, 0)), Some((1, 0))];
        let error = reconstruct_path(&previous, 0, 2)
            .expect_err("cyclic predecessor chain must not loop indefinitely");
        assert!(matches!(error, EmdError::PredecessorCycle { node: 2 }));
    }

    #[test]
    fn kmeans_emd_records_finite_iteration_diagnostics() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.0, 1.0, 0.1, 0.9];
        let weight = vec![2, 1, 3, 4];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let result =
            kmeans_emd(&points, 2, &weight, &ground, 2, 7, 5).expect("valid EMD clustering");

        assert!(!result.diagnostics.is_empty());
        for iteration in result.diagnostics {
            assert!(iteration.objective_before_update.is_finite());
            assert!(iteration.objective_after_update.is_finite());
            assert!((0.0..=1.0).contains(&iteration.moved_weight_fraction));
            assert!((1..=2).contains(&iteration.occupied_buckets));
        }
    }

    #[test]
    fn kmeans_emd_links_reassignment_objective_to_previous_update() {
        let result = kmeans_emd(
            &[1.0, 0.0, 0.8, 0.2, 0.0, 1.0, 0.1, 0.9],
            2,
            &[2, 1, 3, 4],
            &[0.0, 1.0, 1.0, 0.0],
            2,
            7,
            2,
        )
        .expect("valid EMD clustering");
        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(
            result.diagnostics[0].objective_after_reassignment,
            Some(result.diagnostics[1].objective_before_update)
        );
        assert_eq!(result.diagnostics[1].objective_after_reassignment, None);
    }

    #[test]
    fn kmeans_emd_reports_final_assignment_consistency_when_requested() {
        let result = kmeans_emd_with_diagnostics(
            &[1.0, 0.0, 0.8, 0.2, 0.0, 1.0, 0.1, 0.9],
            2,
            &[2, 1, 3, 4],
            &[0.0, 1.0, 1.0, 0.0],
            2,
            7,
            1,
            None,
            true,
        )
        .expect("valid EMD clustering");

        let diagnostic = result
            .final_assignment_diagnostic
            .expect("requested diagnostic must be present");
        assert!(diagnostic.objective.is_finite());
        assert!((0.0..=1.0).contains(&diagnostic.changed_weight_fraction));
    }

    #[test]
    fn kmeans_emd_profile_counts_completed_calls_without_changing_clustering() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.0, 1.0, 0.1, 0.9];
        let weight = vec![2, 1, 3, 4];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let baseline =
            kmeans_emd_with_diagnostics(&points, 2, &weight, &ground, 2, 7, 5, None, true)
                .expect("valid baseline clustering");
        let profiled = kmeans_emd_profiled(&points, 2, &weight, &ground, 2, 7, 5, None, true, true)
            .expect("valid profiled clustering");

        assert_eq!(profiled.clustering.assignment, baseline.assignment);
        assert_eq!(profiled.clustering.centroids, baseline.centroids);
        assert_eq!(profiled.clustering.objective, baseline.objective);
        let profile = profiled.profile.expect("requested EMD profile");
        assert_eq!(profile.iterations.len(), baseline.diagnostics.len());
        for iteration in profile.iterations {
            assert_eq!(iteration.assignment_emd_calls, 8);
            assert_eq!(iteration.objective_after_update_emd_calls, 4);
        }
        assert_eq!(profile.final_assignment_diagnostic_emd_calls, Some(8));
    }

    #[test]
    fn streaming_assignment_matches_vec_reference_clustering_bit_for_bit() {
        let points = vec![1.0, 0.0, 0.8, 0.2, 0.0, 1.0, 0.1, 0.9];
        let weight = vec![2, 1, 3, 4];
        let ground = vec![0.0, 1.0, 1.0, 0.0];
        let reference =
            kmeans_emd_with_reference_assignment(&points, 2, &weight, &ground, 2, 7, 5, true)
                .expect("valid Vec-based reference clustering");
        let streaming =
            kmeans_emd_with_diagnostics(&points, 2, &weight, &ground, 2, 7, 5, None, true)
                .expect("valid streaming clustering");

        assert_eq!(streaming.assignment, reference.assignment);
        assert_eq!(
            streaming
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            reference
                .centroids
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            streaming
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            reference
                .objective
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(streaming.diagnostics.len(), reference.diagnostics.len());
        for (actual, expected) in streaming.diagnostics.iter().zip(&reference.diagnostics) {
            assert_eq!(
                actual.objective_before_update.to_bits(),
                expected.objective_before_update.to_bits()
            );
            assert_eq!(
                actual.objective_after_update.to_bits(),
                expected.objective_after_update.to_bits()
            );
            assert_eq!(
                actual.objective_after_reassignment.map(f64::to_bits),
                expected.objective_after_reassignment.map(f64::to_bits)
            );
            assert_eq!(
                actual.moved_weight_fraction.to_bits(),
                expected.moved_weight_fraction.to_bits()
            );
            assert_eq!(actual.occupied_buckets, expected.occupied_buckets);
        }
        let actual = streaming
            .final_assignment_diagnostic
            .expect("requested streaming diagnostic");
        let expected = reference
            .final_assignment_diagnostic
            .expect("requested reference diagnostic");
        assert_eq!(
            actual.changed_weight_fraction.to_bits(),
            expected.changed_weight_fraction.to_bits()
        );
        assert_eq!(actual.objective.to_bits(), expected.objective.to_bits());
    }

    #[test]
    fn scratch_clustering_matches_pre_scratch_reference_with_one_and_two_rayon_threads() {
        let points = vec![
            0.5, 0.3, 0.2, 0.1, 0.6, 0.3, 0.2, 0.1, 0.7, 0.4, 0.4, 0.2, 0.3, 0.3, 0.4,
        ];
        let weight = vec![2, 3, 5, 7, 11];
        let ground = vec![0.0, 1.0, 2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0];
        let reference =
            kmeans_emd_with_pre_scratch_reference(&points, 3, &weight, &ground, 3, 17, 4, true)
                .expect("valid pre-scratch reference clustering");
        for threads in [1, 2] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("test pool");
            let actual = pool
                .install(|| {
                    kmeans_emd_with_diagnostics(&points, 3, &weight, &ground, 3, 17, 4, None, true)
                })
                .expect("valid scratch clustering");
            assert_eq!(actual.assignment, reference.assignment);
            assert_eq!(actual.objective.len(), reference.objective.len());
            assert_eq!(actual.diagnostics.len(), reference.diagnostics.len());
            assert_eq!(
                actual
                    .centroids
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                reference
                    .centroids
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                actual
                    .objective
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                reference
                    .objective
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            for (actual, expected) in actual.diagnostics.iter().zip(&reference.diagnostics) {
                assert_eq!(
                    actual.objective_before_update.to_bits(),
                    expected.objective_before_update.to_bits()
                );
                assert_eq!(
                    actual.objective_after_update.to_bits(),
                    expected.objective_after_update.to_bits()
                );
                assert_eq!(
                    actual.objective_after_reassignment.map(f64::to_bits),
                    expected.objective_after_reassignment.map(f64::to_bits)
                );
                assert_eq!(
                    actual.moved_weight_fraction.to_bits(),
                    expected.moved_weight_fraction.to_bits()
                );
                assert_eq!(actual.occupied_buckets, expected.occupied_buckets);
            }
            let actual = actual
                .final_assignment_diagnostic
                .as_ref()
                .expect("scratch diagnostic");
            let expected = reference
                .final_assignment_diagnostic
                .as_ref()
                .expect("reference diagnostic");
            assert_eq!(
                actual.changed_weight_fraction.to_bits(),
                expected.changed_weight_fraction.to_bits()
            );
            assert_eq!(actual.objective.to_bits(), expected.objective.to_bits());
        }
    }

    #[test]
    fn arithmetic_mean_can_increase_emd_objective() {
        let ground = [0.0, 1.0, 1.0, 0.0];
        let points = [[1.0, 0.0], [0.0, 1.0]];
        let weights = [3.0, 1.0];
        let previous_center = points[0];
        let mean_center = [0.75, 0.25];
        let objective = |center: &[f32]| {
            points
                .iter()
                .zip(weights)
                .map(|(point, weight)| emd(point, center, &ground, 2) as f64 * weight)
                .sum::<f64>()
                / 4.0
        };
        assert!(objective(&mean_center) > objective(&previous_center));
    }

    #[test]
    fn kmeans_emd_returns_context_for_invalid_distribution() {
        let result = kmeans_emd(&[-1.0, 2.0], 2, &[1], &[0.0, 1.0, 1.0, 0.0], 1, 7, 1);
        assert!(matches!(
            result,
            Err(EmdContextError {
                point_index: 0,
                centroid_index: 0,
                ..
            })
        ));
    }

    #[test]
    fn kmeans_emd_reports_iteration_phase_boundaries() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let points = vec![1.0, 0.0, 0.0, 1.0];
        kmeans_emd_with_progress(
            &points,
            2,
            &[1, 1],
            &[0.0, 1.0, 1.0, 0.0],
            2,
            7,
            1,
            Some(Arc::new(move |event| {
                captured.lock().unwrap().push(event);
            })),
        )
        .expect("valid EMD clustering");
        assert!(events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event, EmdProgressEvent::IterationStart { iteration: 1 })));
        assert!(events.lock().unwrap().iter().any(|event| matches!(
            event,
            EmdProgressEvent::AssignmentProgress {
                iteration: 1,
                processed: 2,
                total: 2,
                emd_calls: 4,
            }
        )));
        assert!(events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event, EmdProgressEvent::IterationEnd { iteration: 1, .. })));
    }

    #[test]
    fn kmeans_wasserstein_1d_matches_general_emd_clustering() {
        let coordinates = vec![0.9, 0.1, 0.5];
        let points = vec![0.7, 0.1, 0.2, 0.1, 0.8, 0.1, 0.2, 0.2, 0.6, 0.0, 0.2, 0.8];
        let weights = vec![1, 2, 3, 4];
        let general = kmeans_emd(&points, 3, &weights, &equity_ground(&coordinates), 2, 23, 4)
            .expect("valid EMD clustering");
        let fast = kmeans_wasserstein_1d(&points, &weights, &coordinates, 2, 23, 4);
        assert_eq!(fast.assignment, general.assignment);
        assert_eq!(fast.centroids.len(), general.centroids.len());
        for (actual, expected) in fast.objective.iter().zip(&general.objective) {
            assert!(
                (actual - expected).abs() < 5e-6,
                "actual={actual}, expected={expected}"
            );
        }
    }
}
