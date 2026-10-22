import hashlib
import json
import tarfile
import zipfile

from tools.tests import test_release_package as fixtures
from tools.release_package import create_package, verify_package
from tools.windows_package import GAME_FILES


class FullReleasePackageTests(fixtures.unittest.TestCase):
    def setUp(self):
        fixtures.ReleasePackageTests.setUp(self)
        for path in [self.runtime / 'index.html', self.root / 'apps/web/www/index.html']:
            path.write_bytes(b'<!doctype html><html><head></head><body></body></html>')
        self.assets = self.base / 'original'
        self.assets.mkdir()
        pins = []
        for name in GAME_FILES:
            data = ('fixture ' + name).encode()
            (self.assets / name).write_bytes(data)
            pins.append(dict(name=name, size=len(data), sha256=hashlib.sha256(data).hexdigest()))
        directory = self.root / 'assets/manifests'
        directory.mkdir(parents=True, exist_ok=True)
        (directory / 'dos_original.json').write_text(json.dumps(dict(profile='fixture', files=pins, sources=[])))

    def package(self, kind, directory='dist'):
        return create_package(kind, self.runtime if kind == 'web' else self.binary,
                              self.base / directory, root=self.root, assets=self.assets)

    def test_both_targets_deliver_exact_original_data_and_default_launch_instructions(self):
        (self.assets / 'PRIVATE.DAT').write_bytes(b'not runtime')
        for kind in ['linux', 'web']:
            path = self.package(kind, kind)
            result = verify_package(path, root=self.root)
            self.assertTrue(result['original_game_data'])
            self.assertEqual(result['game_files'], 28)
            self.assertNotIn('-dev-', path.name)
            if kind == 'web':
                with zipfile.ZipFile(path) as archive:
                    files = {n: archive.read(n) for n in archive.namelist()}
            else:
                with tarfile.open(path) as archive:
                    files = {n: archive.extractfile(n).read() for n in archive.getnames()}
            self.assertEqual({n[5:] for n in files if n.startswith('data/')}, set(GAME_FILES))
            self.assertEqual(json.loads(files['GAME-DATA.json'])['levels'], 120)
            self.assertIn(b'Fun 01', files['README.md'])
            if kind == 'web': self.assertIn(b'name="game-data"', files['index.html'])
            again = self.package(kind, kind + '-again')
            self.assertEqual(path.read_bytes(), again.read_bytes())

    def test_missing_corrupt_or_linked_original_fails_before_writing_archive(self):
        path = self.assets / 'MAIN.DAT'
        original = path.read_bytes()
        path.unlink()
        for kind in ['linux', 'web']:
            with self.assertRaises(ValueError): self.package(kind)
        path.write_bytes(original + b'corrupt')
        with self.assertRaises(ValueError): self.package('web')
        path.unlink()
        other = self.base / 'linked.dat'
        other.write_bytes(original)
        path.symlink_to(other)
        with self.assertRaises(ValueError): self.package('linux')
        self.assertFalse((self.base / 'dist').exists())

    def test_original_pins_cannot_be_replaced_by_self_consistent_tampered_metadata(self):
        from tools.release_package import archive_bytes, encoded, sha
        path = self.package('web')
        with zipfile.ZipFile(path) as archive:
            files = {n: archive.read(n) for n in archive.namelist()}
        files['data/MAIN.DAT'] = b'tampered'
        info = json.loads(files['GAME-DATA.json'])
        pin = next(p for p in info['files'] if p['name'] == 'MAIN.DAT')
        pin.update(size=8, sha256=sha(b'tampered'))
        files['GAME-DATA.json'] = encoded(info)
        manifest = json.loads(files['PACKAGE-MANIFEST.json'])
        for pin in manifest['files']:
            pin.update(bytes=len(files[pin['path']]), sha256=sha(files[pin['path']]))
        files['PACKAGE-MANIFEST.json'] = encoded(manifest)
        files['SHA256SUMS'] = ''.join(f'{sha(v)}  {n}\n' for n,v in sorted(files.items()) if n != 'SHA256SUMS').encode()
        path.write_bytes(archive_bytes(files, 'web'))
        fixtures.Path(str(path) + '.sha256').write_bytes(f'{sha(path.read_bytes())}  {path.name}\n'.encode())
        with self.assertRaises(ValueError): verify_package(path, root=self.root)
