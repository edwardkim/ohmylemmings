"""Acquire and verify the pinned DOS reference files using Python's standard library."""

import argparse
import hashlib
import io
import json
from pathlib import Path
import sys
from urllib.request import urlopen
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = ROOT / "assets/manifests/dos_original.json"


def checked_bytes(data, expected, label):
    if len(data) != expected["size"]:
        raise ValueError(f"{label}: size mismatch")
    if hashlib.sha256(data).hexdigest() != expected["sha256"]:
        raise ValueError(f"{label}: SHA-256 mismatch")
    if expected.get("symbols"):
        lines = data.decode(expected.get("text_encoding", "utf-8")).splitlines()
        for symbol in expected["symbols"]:
            line = symbol["line"]
            if not 1 <= line <= len(lines) or not lines[line - 1].strip().startswith(symbol["needle"]):
                raise ValueError(f"{label}: source symbol location mismatch ({symbol['name']})")
    return data


def file_name(entry):
    name = entry["name"]
    if not name or name in (".", "..") or "/" in name or "\\" in name:
        raise ValueError(f"Invalid flat asset filename: {name!r}")
    return name


def download_bytes(url, expected_size):
    if not url.startswith("https://"):
        raise ValueError("Reference downloads require HTTPS")
    with urlopen(url, timeout=30) as response:
        return response.read(expected_size + 1)


def fetch_assets(manifest, destination, download=download_bytes):
    """Validate all downloads before writing any reference files."""
    sources = {source["id"]: source for source in manifest["sources"]}
    archives = {}
    pending = {}
    for entry in manifest["files"]:
        name = file_name(entry)
        if name in pending:
            raise ValueError(f"Duplicate asset filename: {name}")
        source = sources[entry["source"]]
        if source["kind"] == "zip":
            if source["id"] not in archives:
                data = download(source["url"], source["size"])
                archives[source["id"]] = checked_bytes(data, source, source["id"])
            with ZipFile(io.BytesIO(archives[source["id"]])) as archive:
                member = archive.getinfo(entry["path"])
                if member.file_size != entry["size"]:
                    raise ValueError(f"{name}: archive member size mismatch")
                data = archive.read(member)
        elif source["kind"] == "directory":
            file_name({"name": entry["path"]})
            data = download(source["base_url"] + entry["path"], entry["size"])
        else:
            raise ValueError(f"Unsupported source kind: {source['kind']}")
        pending[name] = checked_bytes(data, entry, name)

    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for name, data in pending.items():
        (destination / name).write_bytes(data)
    verify_assets(manifest, destination)
    return len(pending)


def verify_assets(manifest, destination):
    destination = Path(destination)
    for entry in manifest["files"]:
        name = file_name(entry)
        path = destination / name
        if not path.is_file():
            raise ValueError(f"{name}: missing file")
        checked_bytes(path.read_bytes(), entry, name)
    return len(manifest["files"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("fetch", "verify"))
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--destination", type=Path)
    args = parser.parse_args()
    try:
        manifest = json.loads(args.manifest.read_text())
        if manifest["schema_version"] != 1:
            raise ValueError("Unsupported manifest schema")
        destination = args.destination or ROOT / manifest["local_directory"]
        operation = fetch_assets if args.command == "fetch" else verify_assets
        count = operation(manifest, destination)
    except (OSError, ValueError, KeyError) as error:
        print(f"Reference assets: {error}", file=sys.stderr)
        return 1
    print(f"{args.command}: {count} verified files in {destination}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
