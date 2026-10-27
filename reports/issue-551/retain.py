"""Retain original captures and losslessly compressed native traces with hashes."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def retain(source, destination):
    if not source.is_dir():
        raise ValueError(f"missing input directory: {source}")
    destination.mkdir(parents=True, exist_ok=False)
    manifest = []
    for original in sorted(source.rglob("*")):
        if not original.is_file():
            continue
        relative = original.relative_to(source)
        output = destination / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        compressed = original.suffix in {".json", ".jsonl", ".log", ".stderr"}
        if compressed:
            output = output.with_name(output.name + ".gz")
            with original.open("rb") as incoming, output.open("wb") as raw:
                with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as outgoing:
                    shutil.copyfileobj(incoming, outgoing)
            with gzip.open(output, "rb") as stream:
                if hashlib.file_digest(stream, "sha256").hexdigest() != digest(original):
                    raise ValueError(f"lossless verification failed: {relative}")
        else:
            shutil.copyfile(original, output)
            if digest(output) != digest(original):
                raise ValueError(f"copy verification failed: {relative}")
        manifest.append({"original_path": str(relative), "original_bytes": original.stat().st_size,
                         "original_sha256": digest(original),
                         "retained_path": str(output.relative_to(destination)),
                         "retained_sha256": digest(output)})
    if not manifest:
        raise ValueError(f"empty evidence input: {source}")
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"files": len(manifest), "destination": str(destination), "lossless": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    retain(args.source, args.destination)
