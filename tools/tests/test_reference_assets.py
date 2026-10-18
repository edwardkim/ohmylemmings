import hashlib
import io
from pathlib import Path
import tempfile
import unittest
from zipfile import ZipFile

from tools.reference_assets import fetch_assets, verify_assets


class ReferenceAssetsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.destination = Path(self.temporary.name) / "assets"
        self.level = b"synthetic level fixture"
        self.sound = b"synthetic audio fixture"
        buffer = io.BytesIO()
        with ZipFile(buffer, "w") as archive:
            archive.writestr("LEVEL000.DAT", self.level)
        self.archive = buffer.getvalue()
        self.payloads = {
            "https://example.test/orig.zip": self.archive,
            "https://example.test/data/adlib.dat": self.sound,
        }
        self.manifest = {
            "sources": [
                {"id": "zip", "kind": "zip", "url": "https://example.test/orig.zip", **self.identity(self.archive)},
                {"id": "raw", "kind": "directory", "base_url": "https://example.test/data/"},
            ],
            "files": [
                {"name": "LEVEL000.DAT", "path": "LEVEL000.DAT", "source": "zip", **self.identity(self.level)},
                {"name": "ADLIB.DAT", "path": "adlib.dat", "source": "raw", **self.identity(self.sound)},
            ],
        }

    @staticmethod
    def identity(data):
        return {"size": len(data), "sha256": hashlib.sha256(data).hexdigest()}

    def fetch(self):
        return fetch_assets(self.manifest, self.destination, lambda url, size: self.payloads[url])

    def test_acquires_zip_and_raw_assets_and_verifies_written_bytes(self):
        self.assertEqual(self.fetch(), 2)
        self.assertEqual((self.destination / "LEVEL000.DAT").read_bytes(), self.level)
        self.assertEqual((self.destination / "ADLIB.DAT").read_bytes(), self.sound)
        self.assertEqual(verify_assets(self.manifest, self.destination), 2)

    def test_corrupt_archive_is_rejected_before_any_asset_is_written(self):
        self.payloads["https://example.test/orig.zip"] = b"x" * len(self.archive)
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            self.fetch()
        self.assertFalse(self.destination.exists())

    def test_corrupt_raw_file_is_rejected_before_any_asset_is_written(self):
        self.payloads["https://example.test/data/adlib.dat"] = b"x" * len(self.sound)
        with self.assertRaisesRegex(ValueError, "ADLIB.DAT: SHA-256 mismatch"):
            self.fetch()
        self.assertFalse(self.destination.exists())

    def test_verification_detects_modified_local_asset(self):
        self.fetch()
        (self.destination / "ADLIB.DAT").write_bytes(b"x" * len(self.sound))
        with self.assertRaisesRegex(ValueError, "ADLIB.DAT: SHA-256 mismatch"):
            verify_assets(self.manifest, self.destination)

    def test_verification_identifies_missing_asset(self):
        self.fetch()
        (self.destination / "LEVEL000.DAT").unlink()
        with self.assertRaisesRegex(ValueError, "LEVEL000.DAT: missing file"):
            verify_assets(self.manifest, self.destination)

    def test_manifest_cannot_write_outside_asset_directory(self):
        self.manifest["files"][0]["name"] = "../escape.dat"
        with self.assertRaisesRegex(ValueError, "Invalid flat asset filename"):
            self.fetch()
        self.assertFalse((self.destination.parent / "escape.dat").exists())
