import hashlib
import io
import tarfile
import tempfile
from pathlib import Path
import unittest

from tools.web_build import install_bindgen, verify_bindgen


class WebBuildTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'tool'
        self.binary = b'synthetic executable bytes'
        self.member = 'tool/wasm-bindgen'
        self.archive = self.pack(self.member)
        identity = lambda data: dict(size=len(data), sha256=hashlib.sha256(data).hexdigest())
        self.manifest = dict(version='test', archive=dict(url='https://example.test/tool.tar.gz', **identity(self.archive)), binary=dict(member=self.member, **identity(self.binary)))

    def pack(self, name, kind=tarfile.REGTYPE):
        out = io.BytesIO()
        with tarfile.open(fileobj=out, mode='w:gz') as archive:
            entry = tarfile.TarInfo(name); entry.size = len(self.binary) if kind == tarfile.REGTYPE else 0; entry.type = kind
            archive.addfile(entry, io.BytesIO(self.binary) if entry.size else None)
        return out.getvalue()

    def install(self):
        return install_bindgen(self.manifest, self.root, lambda url, size: self.archive)

    def test_only_the_pinned_binary_is_installed_and_verified(self):
        path = self.install()
        self.assertEqual(path.read_bytes(), self.binary)
        self.assertTrue(path.stat().st_mode & 0o100)
        self.assertEqual(verify_bindgen(self.manifest, self.root), path)
        self.assertEqual(list(self.root.iterdir()), [path])

    def test_corrupt_archive_is_rejected_before_writing(self):
        self.archive = b'x' * len(self.archive)
        with self.assertRaisesRegex(ValueError, 'SHA-256 mismatch'): self.install()
        self.assertFalse(self.root.exists())

    def test_wrong_member_hash_and_modified_installed_binary_are_rejected(self):
        self.manifest['binary']['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256 mismatch'): self.install()
        self.assertFalse(self.root.exists())
        self.manifest['binary']['sha256'] = hashlib.sha256(self.binary).hexdigest(); path = self.install(); path.write_bytes(b'wrong')
        with self.assertRaisesRegex(ValueError, 'size mismatch'): verify_bindgen(self.manifest, self.root)

    def test_missing_binary_is_an_actionable_error(self):
        with self.assertRaisesRegex(ValueError, 'web_build.py fetch'): verify_bindgen(self.manifest, self.root)

    def test_tar_links_and_missing_members_are_rejected(self):
        for name, kind in [(self.member, tarfile.SYMTYPE), ('other', tarfile.REGTYPE)]:
            self.archive = self.pack(name, kind)
            self.manifest['archive'].update(size=len(self.archive), sha256=hashlib.sha256(self.archive).hexdigest())
            with self.assertRaises(ValueError): self.install()
            self.assertFalse(self.root.exists())
