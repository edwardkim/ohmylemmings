import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

from tools.release_package import WEB_FILES, create_package, verify_package
from tools.web_build import SOURCE_REQUIRED


class ReleasePackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / 'source'
        self.runtime = self.base / 'runtime'
        self.root.mkdir()
        for name in SOURCE_REQUIRED:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'public source\n')
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
        self.runtime.mkdir()
        for name in WEB_FILES:
            path = self.runtime / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'\0asm\1\0\0\0' if name.endswith('.wasm') else b'web runtime\n')
            if not name.startswith('pkg/'):
                source = self.root / 'apps/web/www' / name
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_bytes(path.read_bytes())
        # A source distribution has no Git metadata and must stay rebuildable.
        names = sorted(p.relative_to(self.root).as_posix() for p in self.root.rglob('*') if p.is_file())
        (self.root / 'SOURCE-MANIFEST.json').write_text(json.dumps([dict(path=n) for n in names]))
        self.binary = self.base / 'lemmings-desktop'
        elf = bytearray(64)
        elf[:6] = b'\x7fELF\x02\x01'
        elf[18:20] = (62).to_bytes(2, 'little')
        self.binary.write_bytes(elf)

    def package(self, kind, directory='dist'):
        return create_package(kind, self.runtime if kind == 'web' else self.binary,
                              self.base / directory, root=self.root)

    def test_web_allowlist_excludes_originals_and_includes_verified_source_and_hashes(self):
        (self.runtime / 'MAIN.DAT').write_bytes(b'private original')
        (self.runtime / 'performance').mkdir()
        (self.runtime / 'performance/raw.json').write_bytes(b'private run')
        (self.root / 'assets/local').mkdir(parents=True)
        (self.root / 'assets/local/secret.dat').write_bytes(b'private original')
        path = self.package('web')
        result = verify_package(path)
        self.assertEqual(result['target'], 'web')
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            self.assertNotIn('MAIN.DAT', names)
            self.assertFalse(any(n.startswith('performance/') for n in names))
            self.assertIn('audio-source.tar.gz', names)
            self.assertIn('LGPL-2.1-or-later.txt', names)
            info = json.loads(archive.read('audio-source.json'))
            self.assertEqual(info['wasm_sha256'], hashlib.sha256(archive.read('pkg/lemmings_web_bg.wasm')).hexdigest())
        self.assertTrue(result['development'])
        self.assertTrue(result['performance_qualification_deferred'])

    def test_same_inputs_produce_identical_archives_for_both_targets(self):
        for kind in ['linux', 'web']:
            first = self.package(kind, 'first-' + kind)
            second = self.package(kind, 'second-' + kind)
            self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_native_binary_is_executable_and_source_is_delivered(self):
        path = self.package('linux')
        self.assertEqual(verify_package(path)['target'], 'linux-x86_64')
        with tarfile.open(path) as archive:
            self.assertEqual(archive.getmember('lemmings-desktop').mode, 0o755)
            self.assertIn('source.tar.gz', archive.getnames())

    def test_missing_symlinked_stale_or_wrong_architecture_runtime_fails(self):
        target = self.runtime / 'app.js'
        original = target.read_bytes()
        target.unlink()
        with self.assertRaises(ValueError): self.package('web')
        target.symlink_to(self.root / 'apps/web/www/app.js')
        with self.assertRaises(ValueError): self.package('web')
        target.unlink()
        target.write_bytes(original + b'stale')
        with self.assertRaises(ValueError): self.package('web')
        self.binary.write_bytes(b'not an ELF')
        with self.assertRaises(ValueError): self.package('linux')

    def test_existing_evidence_is_never_overwritten(self):
        path = self.package('web')
        data = path.read_bytes()
        with self.assertRaises(FileExistsError): self.package('web')
        self.assertEqual(path.read_bytes(), data)

    def test_changed_archive_bytes_fail_checksum(self):
        path = self.package('web')
        path.write_bytes(path.read_bytes() + b'tampered')
        with self.assertRaises(ValueError): verify_package(path)

    def test_internal_corruption_or_extra_files_fail_even_with_updated_archive_checksum(self):
        for change in ['app.js', 'MAIN.DAT']:
            path = self.package('web', 'changed-' + change)
            with zipfile.ZipFile(path) as archive:
                files = {name: archive.read(name) for name in archive.namelist()}
            files[change] = b'changed bytes'
            with zipfile.ZipFile(path, 'w') as archive:
                for name, data in files.items():
                    entry = zipfile.ZipInfo(name)
                    entry.create_system = 3
                    entry.external_attr = 0o100644 << 16
                    archive.writestr(entry, data)
            Path(str(path) + '.sha256').write_text(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n')
            with self.assertRaises(ValueError): verify_package(path)
