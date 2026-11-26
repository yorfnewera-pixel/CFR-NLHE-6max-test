import os
import random
import subprocess
from pathlib import Path

import numpy as np
import pytest

from src.blueprint.equity_buckets import BucketTable, class_key, hash_key

BINARY = Path("native/hu20-buckets/target/release/hu20-buckets")

# Produced by `hu20-buckets key HOLE BOARD`; Python must match bit for bit.
GOLDEN = [
    (("Ah", "Kh"), ("2h", "7c", "9d"), 0x000000c0000040000800000080000000, 0x569f5133b1765a33),
    (("As", "Ks"), ("2s", "7d", "9c"), 0x000000c0000040000800000080000000, 0x569f5133b1765a33),
    (("Ah", "Kd"), ("2h", "7c", "9d", "Qs"), 0x00000080000050000800001000000020, 0xcedd4a5bfce4855c),
    (("Tc", "9c"), ("2c", "3c", "4d", "5h", "Ks"), 0x0000000c0000c0008000000020000004, 0xad83f36d639fd63e),
    (("2d", "2h"), ("Ad", "Kd", "Qd"), 0x000000000f0000020000000000000000, 0xe54792613f12b1c2),
]

LIBRATUS_FIXTURE = bytes.fromhex(
    (Path(__file__).parent / "fixtures" / "libratus-turn-k8.hex").read_text()
)
LIBRATUS_FIXTURE_SHA256 = "dbff96bd85dd47e181714b7cfa207be27396f668dd22f51939877b7a61731d96"
K50_FIXTURE = (
    b"HU20BKT1" + np.array([2, 50], "<u4").tobytes()
    + np.array([2], "<u8").tobytes() + LIBRATUS_FIXTURE[28:]
)


def _libratus_table(path):
    # Импорт внутри helper сохраняет RED-доказательство на родительском commit:
    # pytest собирает тесты, но новый читатель там ещё отсутствует.
    from src.blueprint.equity_buckets import LibratusBucketTable
    table = LibratusBucketTable.__new__(LibratusBucketTable)
    table._load(path, None)
    return table


def _write_libratus_fixture(path, raw=LIBRATUS_FIXTURE):
    path.write_bytes(raw)
    return path


def test_libratus_round_trip_fixture_and_suit_isomorphism(tmp_path):
    import hashlib

    path = _write_libratus_fixture(tmp_path / "turn.bin")
    table = _libratus_table(path)

    assert hashlib.sha256(path.read_bytes()).hexdigest() == LIBRATUS_FIXTURE_SHA256
    assert (table.street, table.k, type(table.k)) == ("turn", 8, int)
    assert table.bucket(("Ah", "Kh"), ("2h", "7c", "9d", "Js")) == 3
    assert table.bucket(("As", "Ks"), ("2s", "7h", "9c", "Jd")) == 3
    assert table.bucket(("Ah", "Kd"), ("2h", "7c", "9d", "Js")) == 7


@pytest.mark.parametrize("name,mutate", [
    ("magic", lambda raw: raw.__setitem__(slice(0, 8), b"HU20BKT1")),
    ("version", lambda raw: raw.__setitem__(slice(8, 12), (2).to_bytes(4, "little"))),
    ("short_header", lambda raw: raw.__delitem__(slice(27, None))),
    ("count_size", lambda raw: raw.__setitem__(slice(20, 28), (3).to_bytes(8, "little"))),
    ("unsorted_hashes", lambda raw: raw.__setitem__(slice(28, 44), raw[36:44] + raw[28:36])),
    ("duplicate_hash", lambda raw: raw.__setitem__(slice(36, 44), raw[28:36])),
    ("bucket_out_of_range", lambda raw: raw.__setitem__(slice(46, 48), (8).to_bytes(2, "little"))),
    ("street", lambda raw: raw.__setitem__(slice(12, 16), (1).to_bytes(4, "little"))),
    ("zero_k", lambda raw: raw.__setitem__(slice(16, 20), (0).to_bytes(4, "little"))),
    ("empty", lambda raw: raw.__setitem__(slice(None), raw[:20] + (0).to_bytes(8, "little"))),
])
def test_libratus_rejects_invalid_format(tmp_path, name, mutate):
    raw = bytearray(LIBRATUS_FIXTURE)
    mutate(raw)
    path = _write_libratus_fixture(tmp_path / f"{name}.bin", bytes(raw))

    with pytest.raises(ValueError):
        _libratus_table(path)


def test_libratus_and_k50_readers_reject_each_others_magic(tmp_path):
    libratus_path = _write_libratus_fixture(tmp_path / "libratus.bin")
    k50_path = tmp_path / "k50.bin"
    k50_path.write_bytes(K50_FIXTURE)

    with pytest.raises(ValueError):
        BucketTable(libratus_path)
    with pytest.raises(ValueError):
        _libratus_table(k50_path)


def test_libratus_k50_reader_preserves_fixture_bytes_and_ids(tmp_path):
    path = tmp_path / "turn-k50.bin"
    path.write_bytes(K50_FIXTURE)

    table = BucketTable(path)

    assert path.read_bytes() == K50_FIXTURE
    assert (table.street, table.k) == ("turn", 50)
    assert table.bucket(("Ah", "Kh"), ("2h", "7c", "9d", "Js")) == 3
    assert table.bucket(("Ah", "Kd"), ("2h", "7c", "9d", "Js")) == 7


@pytest.mark.parametrize("hole,board,key,digest", GOLDEN)
def test_class_key_matches_rust_builder(hole, board, key, digest):
    assert class_key(hole, board) == key
    assert hash_key(key) == digest


def test_reader_finds_buckets_and_rejects_other_streets(tmp_path):
    rows = sorted((hash_key(class_key(h, b)), bucket) for h, b, bucket in [
        (("Ah", "Kh"), ("2h", "7c", "9d"), 3), (("Ah", "Kd"), ("2h", "7c", "9d"), 7)])
    path = tmp_path / "flop-k8.bin"
    with path.open("wb") as out:
        out.write(b"HU20BKT1" + np.array([1, 8], "<u4").tobytes() + np.array([len(rows)], "<u8").tobytes())
        out.write(np.array([r[0] for r in rows], "<u8").tobytes() + np.array([r[1] for r in rows], "<u2").tobytes())
    table = BucketTable(path)
    assert (table.street, table.k) == ("flop", 8)
    assert table.bucket(("Ks", "As"), ("9c", "2s", "7d")) == 3  # suit-isomorphic, any order
    assert table.bucket(("Ah", "Kd"), ("2h", "7c", "9d")) == 7
    with pytest.raises(ValueError):
        table.bucket(("Ah", "Kd"), ("2h", "7c", "9d", "Qs"))
    with pytest.raises(KeyError):
        table.bucket(("2c", "3d"), ("4h", "5s", "7c"))


@pytest.mark.skipif(not BINARY.exists(), reason="build native/hu20-buckets first")
def test_native_evaluator_orders_hands_like_the_engine():
    from src.game.showdown import hand_value
    deck = [r + s for r in "23456789TJQKA" for s in "cdhs"]
    rng = random.Random(7)
    hands = [rng.sample(deck, 7) for _ in range(5000)]
    out = subprocess.run([str(BINARY), "eval"], input="\n".join(" ".join(h) for h in hands),
                         capture_output=True, text=True, check=True).stdout.split()
    native = [int(v) for v in out]
    engine = [hand_value(tuple(h)) for h in hands]
    for _ in range(50000):
        i, j = rng.randrange(len(hands)), rng.randrange(len(hands))
        assert (engine[i] > engine[j]) - (engine[i] < engine[j]) == (native[i] > native[j]) - (native[i] < native[j])
@pytest.mark.parametrize("ranks", [5, 13])
def test_libratus_manifest_rejects_table_digest_mismatch(tmp_path, ranks):
    import hashlib
    import json
    from pathlib import Path
    from src.blueprint.equity_buckets import LibratusBucketTable

    raw = bytes.fromhex(Path("tests/fixtures/libratus-turn-k8.hex").read_text().strip())
    table = tmp_path / "table.bin"
    manifest = tmp_path / "manifest.json"
    table.write_bytes(raw)
    metadata = {"format": "HU20LIB1", "street": 2, "k": 8, "class_count": 2,
                "table_sha256": hashlib.sha256(raw).hexdigest(), "total_weight": 2,
                "ranks": ranks, "code_revision": "test", "working_tree_dirty": False,
                "canonicalization": "hole-current-board-suit-isomorphism-v1", "context": "HU20-menu-V1", "normalization": "runout-count-cdf", "restart_selection": "minimum-full-weighted-objective-first-tie",
                "feature": "turn-equity-cdf", "bins": 50, "distance": "cdf-l1-sum",
                "centroid": "weighted-mean", "ochs_map_sha256": "6483f50df6ebd7e51d37353be6040f1c1bfd800907f870991576ae31c8f0a335",
                "empty_group_rule": "uniform-equity-fill-v1", "seed": 2026112101,
                "initialization": "weighted-sample-20000", "restarts": 3,
                "max_iterations": 50, "stop_rule": "moved-weight-fraction-lt-1e-4",
                "tie_break": "first-minimum"}
    manifest.write_text(json.dumps(metadata))
    with pytest.raises(ValueError, match="unsupported deck"):
        LibratusBucketTable(table, manifest)
    data = json.loads(manifest.read_text())
    data["table_sha256"] = "0" * 64
    manifest.write_text(json.dumps(data))
    with pytest.raises(ValueError, match="manifest"):
        LibratusBucketTable(table, manifest)

@pytest.mark.parametrize("field,value", [
    ("working_tree_dirty", "false"),
    ("seed", -1),
    ("code_revision", 123),
])
def test_libratus_manifest_rejects_invalid_json_types(tmp_path, field, value):
    import json
    from src.blueprint.equity_buckets import LibratusBucketTable

    root = Path("results/libratus-pr-b/release1")
    table = root / "libratus-turn-k50.bin"
    manifest = root / "libratus-turn-k50.json"
    if not table.exists() or not manifest.exists():
        pytest.skip("нужны малые таблицы PR B")
    metadata = json.loads(manifest.read_text(encoding="utf-8"))
    metadata[field] = value
    altered = tmp_path / "manifest.json"
    altered.write_text(json.dumps(metadata), encoding="utf-8")
    with pytest.raises(ValueError):
        LibratusBucketTable(table, altered)


def test_libratus_requires_manifest():
    from src.blueprint.equity_buckets import LibratusBucketTable

    table = Path("results/libratus-pr-b/release1/libratus-turn-k50.bin")
    if not table.exists():
        pytest.skip("нужна малая таблица PR B")
    with pytest.raises((TypeError, ValueError)):
        LibratusBucketTable(table)  # type: ignore[call-arg]
    with pytest.raises((TypeError, ValueError)):
        LibratusBucketTable(table, None)


@pytest.mark.parametrize("field,value", [
    ("working_tree_dirty", "false"),
    ("seed", -1),
    ("code_revision", 123),
])
def test_libratus_manifest_type_validation_without_artifacts(field, value):
    from src.blueprint.equity_buckets import _validate_libratus_manifest_types

    valid = {"working_tree_dirty": False, "seed": 0, "code_revision": "123",
             "street": 2, "k": 50, "class_count": 2, "total_weight": 2,
             "ranks": 7, "bins": 50, "restarts": 1, "max_iterations": 50}
    _validate_libratus_manifest_types(valid)
    valid[field] = value
    with pytest.raises(ValueError):
        _validate_libratus_manifest_types(valid)


@pytest.mark.parametrize("field,bits", [
    ("street", 32), ("k", 32), ("class_count", 64), ("total_weight", 64),
    ("ranks", 8), ("bins", 32), ("seed", 64), ("restarts", 32), ("max_iterations", 32),
])
@pytest.mark.parametrize("bad", ["float", "boolean", "negative", "overflow"])
def test_libratus_integer_types_match_rust_without_artifacts(field, bits, bad):
    from src.blueprint.equity_buckets import _validate_libratus_manifest_types

    valid = {"working_tree_dirty": False, "seed": 0, "code_revision": "123",
             "street": 2, "k": 50, "class_count": 2, "total_weight": 2,
             "ranks": 7, "bins": 50, "restarts": 1, "max_iterations": 50}
    _validate_libratus_manifest_types(valid)
    valid[field] = {"float": 7.0, "boolean": True, "negative": -1, "overflow": 2**bits}[bad]
    with pytest.raises(ValueError):
        _validate_libratus_manifest_types(valid)
