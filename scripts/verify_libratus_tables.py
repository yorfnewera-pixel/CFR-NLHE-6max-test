"""Сверка читателей HU20LIB1 на выпущенных таблицах и повреждениях."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import struct
import tempfile

from src.blueprint.equity_buckets import LibratusBucketTable


def verify_rust(executable, table, manifest):
    return subprocess.run([str(executable), "verify-libratus", str(table), str(manifest)],
                          capture_output=True, text=True, check=False)


def check(executable, table, manifest):
    raw = table.read_bytes()
    parsed = LibratusBucketTable(table, manifest)
    reconstructed = raw[:28] + parsed.hashes.tobytes() + parsed.buckets.tobytes()
    digest = hashlib.sha256(reconstructed).hexdigest()
    result = verify_rust(executable, table, manifest)
    if result.returncode or result.stdout.strip() != digest or reconstructed != raw:
        raise AssertionError(f"читатели не согласны: {table}: {result.stderr}")
    metadata = json.loads(manifest.read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory() as temporary:
        broken = Path(temporary) / "broken.bin"
        bad_manifest = Path(temporary) / "manifest.json"
        for case in ("magic", "bucket", "manifest", "missing-class"):
            data = bytearray(raw)
            altered = dict(metadata)
            if case == "magic":
                data[0] ^= 1
            elif case == "bucket":
                data[-2] ^= 1
            elif case == "missing-class":
                count = metadata["class_count"]
                data = data[:28 + (count - 1) * 8] + data[28 + count * 8:-2]
                struct.pack_into("<Q", data, 20, count - 1)
                altered["class_count"] = count - 1
                altered["table_sha256"] = hashlib.sha256(data).hexdigest()
            else:
                altered["street"] = 5 - metadata["street"]
            broken.write_bytes(data)
            bad_manifest.write_text(json.dumps(altered), encoding="utf-8")
            try:
                LibratusBucketTable(broken, bad_manifest)
            except (ValueError, KeyError):
                pass
            else:
                raise AssertionError(f"Python принял повреждение {case}")
            if verify_rust(executable, broken, bad_manifest).returncode == 0:
                raise AssertionError(f"Rust принял повреждение {case}")
    print(f"{table.name}: {len(parsed.buckets)} ID, SHA-256 {digest}; повреждения отклонены")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("executable", type=Path)
    parser.add_argument("directory", type=Path)
    arguments = parser.parse_args()
    for path in sorted(arguments.directory.glob("libratus-*.bin")):
        check(arguments.executable.resolve(), path.resolve(), path.with_suffix(".json").resolve())
