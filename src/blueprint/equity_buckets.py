"""Read equity-distribution bucket tables built by `native/hu20-buckets`.

A situation is a two-card holding plus a flop, turn or river board. Its class
key is suit-isomorphism invariant and matches the Rust builder bit for bit.
`EquityCards` holds one K=50 table per street for the equity-bucket information
keys; the native trainer's `cards.rs` reads the same files.
"""

from hashlib import sha256
from pathlib import Path

import numpy as np

RANKS = "23456789TJQKA"
SUITS = "cdhs"
STREETS = {3: "flop", 4: "turn", 5: "river"}
# #163's K=50 tables (docs/reports/hu20-equity-buckets.md), validated by #190.
EQUITY_K50_SHA256 = {
    "flop": "084e243ded9fe93ad03cd59601f22f16c3210592f51e1e3646a60983b572c111",
    "turn": "c0bc6a8aa0535553118109d18a32d3b4dc6880e937c263cdc87472b1ae9f168f",
    "river": "70c1cb5ecf4cd292e4c89dc2200f7c8381100be3b0288ceeb0a12a3e83977629",
}
MASK64 = (1 << 64) - 1


def class_key(hole, board):
    signature = [0, 0, 0, 0]
    for card in hole:
        signature[SUITS.index(card[1])] |= 1 << (13 + RANKS.index(card[0]))
    for card in board:
        signature[SUITS.index(card[1])] |= 1 << RANKS.index(card[0])
    key = 0
    for value in sorted(signature, reverse=True):
        key = (key << 26) | value
    return key


def _splitmix(x):
    x = (x + 0x9E3779B97F4A7C15) & MASK64
    x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    x = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & MASK64
    return x ^ (x >> 31)


def hash_key(key):
    return _splitmix(((key >> 64) & MASK64) ^ _splitmix(key & MASK64))


class BucketTable:
    """One street's table: sorted 64-bit class hashes and their bucket ids."""

    def __init__(self, path):
        raw = np.memmap(Path(path), dtype=np.uint8, mode="r")
        if bytes(raw[:8]) != b"HU20BKT1":
            raise ValueError("Not an HU20 bucket table")
        street, self.k = (int(v) for v in np.frombuffer(raw[8:16], dtype="<u4"))
        count = int(np.frombuffer(raw[16:24], dtype="<u8")[0])
        self.street = {1: "flop", 2: "turn", 3: "river"}[street]
        self.hashes = np.frombuffer(raw, dtype="<u8", count=count, offset=24)
        self.buckets = np.frombuffer(raw, dtype="<u2", count=count, offset=24 + 8 * count)

    def bucket(self, hole, board):
        if STREETS[len(board)] != self.street:
            raise ValueError("Board size belongs to another street's table")
        target = np.uint64(hash_key(class_key(hole, board)))
        index = int(np.searchsorted(self.hashes, target))
        if index >= len(self.hashes) or self.hashes[index] != target:
            raise KeyError("Situation is missing from the bucket table")
        return int(self.buckets[index])


class LibratusBucketTable:
    """Строго валидируемая turn/river-таблица формата HU20LIB1."""

    MAGIC = b"HU20LIB1"
    VERSION = 1
    _STREETS = {2: "turn", 3: "river"}

    def __init__(self, path, manifest_path=None):
        raw = Path(path).read_bytes()
        if len(raw) < 28:
            raise ValueError("file size does not match header")
        if raw[:8] != self.MAGIC:
            raise ValueError("not an HU20LIB1 table")
        version, street, k = np.frombuffer(raw[8:20], dtype="<u4")
        if int(version) != self.VERSION:
            raise ValueError("unsupported format version")
        if int(street) not in self._STREETS:
            raise ValueError("unknown street")
        self.k = int(k)
        if not 1 <= self.k <= 65535:
            raise ValueError("K is out of range")
        count = int(np.frombuffer(raw[20:28], dtype="<u8")[0])
        if count < 1 or len(raw) != 28 + 10 * count:
            raise ValueError("file size does not match header")
        self.street = self._STREETS[int(street)]
        self.hashes = np.frombuffer(raw, dtype="<u8", count=count, offset=28)
        self.buckets = np.frombuffer(raw, dtype="<u2", count=count, offset=28 + 8 * count)
        if np.any(self.hashes[1:] <= self.hashes[:-1]):
            raise ValueError("hashes are not strictly increasing")
        if np.any(self.buckets >= self.k):
            raise ValueError("bucket ID is out of range")
        if manifest_path is not None:
            import hashlib
            import json

            manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
            expected_fields = {"format", "street", "k", "class_count", "total_weight", "ranks", "code_revision", "working_tree_dirty", "table_sha256", "canonicalization", "context", "normalization", "feature", "bins", "distance", "centroid", "ochs_map_sha256", "empty_group_rule", "seed", "initialization", "restarts", "restart_selection", "max_iterations", "stop_rule", "tie_break"}
            if (manifest.get("format") != "HU20LIB1"
                    or set(manifest) != expected_fields
                    or manifest.get("street") != int(street)
                    or manifest.get("k") != self.k
                    or manifest.get("class_count") != count
                    or manifest.get("table_sha256") != hashlib.sha256(raw).hexdigest()
                    or manifest.get("feature") != ("turn-equity-cdf" if self.street == "turn" else "river-ochs-8")
                    or manifest.get("distance") != ("cdf-l1-sum" if self.street == "turn" else "squared-l2")
                    or manifest.get("bins") != (50 if self.street == "turn" else 0)
                    or manifest.get("canonicalization") != "hole-current-board-suit-isomorphism-v1"
                    or manifest.get("context") != "HU20-menu-V1"
                    or manifest.get("normalization") != ("runout-count-cdf" if self.street == "turn" else "eligible-opponent-equity-half-ties")
                    or manifest.get("restart_selection") != "minimum-full-weighted-objective-first-tie"
                    or manifest.get("ochs_map_sha256") != "6483f50df6ebd7e51d37353be6040f1c1bfd800907f870991576ae31c8f0a335"
                    or manifest.get("empty_group_rule") != "uniform-equity-fill-v1"
                    or manifest.get("tie_break") != "first-minimum"
                    or manifest.get("stop_rule") != "moved-weight-fraction-lt-1e-4"
                    or not manifest.get("total_weight") or not manifest.get("code_revision")
                    or not manifest.get("restarts") or not manifest.get("max_iterations")):
                raise ValueError("manifest does not match table")
            if manifest.get("ranks") != 7:
                raise ValueError("unsupported deck in manifest")
            if (not 1 <= manifest["max_iterations"] <= 50
                    or (self.street == "turn" and (manifest["total_weight"] != 5651100
                        or manifest["initialization"] != "k50-weighted-sample-200000"
                        or manifest["restarts"] != 1 or manifest["centroid"] not in {"mean", "lower-weighted-median"}))
                    or (self.street == "river" and (manifest["total_weight"] != 24864840
                        or manifest["initialization"] != "all-weighted-if-n-le-20000-else-weighted-20000-with-replacement"
                        or manifest["restarts"] != 3 or manifest["centroid"] != "weighted-mean"))):
                raise ValueError("invalid manifest parameters")
            if manifest.get("ranks") == 7:
                expected_count, expected_hash = {
                    "turn": (260862, "bafb46f8241f6e1048e51b995253a3763f2599e2d6c844f9626aa07ffbd993b0"),
                    "river": (1103011, "3d06ba9dfe40250cb3d554888b1ec1bb55964562076e90da0e59eecd21eb78a6"),
                }[self.street]
                if count != expected_count or hashlib.sha256(raw[28:28 + 8 * count]).hexdigest() != expected_hash:
                    raise ValueError("incomplete class set")

    def bucket(self, hole, board):
        if STREETS[len(board)] != self.street:
            raise ValueError("Board size belongs to another street's table")
        target = np.uint64(hash_key(class_key(hole, board)))
        index = int(np.searchsorted(self.hashes, target))
        if index >= len(self.hashes) or self.hashes[index] != target:
            raise KeyError("Situation is missing from the bucket table")
        return int(self.buckets[index])


def write_table(path, street, k, buckets):
    """A table in the builder's format, from {class key: bucket}; for test fixtures."""
    code = {"flop": 1, "turn": 2, "river": 3}[street]
    rows = sorted((hash_key(key), bucket) for key, bucket in buckets.items())
    if any(a[0] == b[0] for a, b in zip(rows, rows[1:])):
        raise ValueError("64-bit class hash collision")
    header = b"HU20BKT1" + np.array([code, k], dtype="<u4").tobytes() + np.array([len(rows)], dtype="<u8").tobytes()
    hashes = np.array([r[0] for r in rows], dtype="<u8").tobytes()
    ids = np.array([r[1] for r in rows], dtype="<u2").tobytes()
    Path(path).write_bytes(header + hashes + ids)


def file_sha256(path):
    digest = sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


class EquityCards:
    """`{flop,turn,river}-k50.bin` from one directory. Unless `pinned=False` (test tables
    only), each file must be #163's, so the schema name identifies its labels."""

    def __init__(self, directory, *, pinned=True):
        self.tables, self.sha256 = {}, {}
        for street in ("flop", "turn", "river"):
            path = Path(directory) / f"{street}-k50.bin"
            digest = file_sha256(path)
            if pinned and digest != EQUITY_K50_SHA256[street]:
                raise ValueError(f"{path} is not #163's {street} K=50 table")
            table = BucketTable(path)
            if table.street != street or table.k != 50:
                raise ValueError(f"{path} is not a {street} K=50 table")
            self.tables[street], self.sha256[street] = table, digest

    def bucket(self, hole, board):
        return self.tables[STREETS[len(board)]].bucket(hole, board)


def load_cards(schema, identity, directory):
    """Bind a reader to explicitly supplied, pinned, memory-mapped tables.

    Never consult the legacy registration used by key/parity fixture builders.
    """
    from src.blueprint.abstraction import EQUITY_SCHEMAS
    if schema not in EQUITY_SCHEMAS:
        if directory is not None:
            raise ValueError("Card bucket tables require a K50 schema")
        return None
    if directory is None:
        raise ValueError("K50 models require explicit card_buckets with #163's pinned tables")
    cards = EquityCards(directory)
    if identity.get("card_tables") != cards.sha256:
        raise ValueError("K50 model table SHA256 identity differs from supplied tables")
    return cards


_registered = None


def register(cards):
    """Tables that equity-bucket information keys use from now on in this process."""
    global _registered
    _registered = cards


def registered():
    if _registered is None:
        raise ValueError("Equity-bucket keys need tables; call equity_buckets.register first")
    return _registered
