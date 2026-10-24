import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from tools.windows_package import GAME_FILES, create, verify
from tools.web_build import SOURCE_REQUIRED


def pe(dll=False):
    if not dll:
        from tools.tests.test_app_icons import icon_pe
        return icon_pe()
    data = bytearray(256)
    data[:2] = b'MZ'; data[60:64] = (128).to_bytes(4, 'little')
    data[128:132] = b'PE\0\0'; data[132:134] = (0x8664).to_bytes(2, 'little')
    data[150:152] = (0x2000 if dll else 2).to_bytes(2, 'little')
    data[152:154] = (0x20b).to_bytes(2, 'little')
    return bytes(data)


class WindowsPackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name); self.root = self.base/'source'; self.root.mkdir()
        for name in SOURCE_REQUIRED:
            path = self.root/name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'public\n')
        (self.root/'Cargo.toml').write_text('[workspace.package]\nversion="0.1.0"\n')
        from tools.tests.test_app_icons import install_fixture as install_icons
        install_icons(self.root)
        launcher = self.root/'apps/desktop/windows/start-game.cmd'; launcher.parent.mkdir(parents=True)
        launcher.write_bytes(b'@echo off\r\n"%~dp0OhMyLemmings.exe" %*\r\n')
        self.runtime = self.base/'runtime'; self.runtime.mkdir()
        (self.runtime/'lemmings-desktop.exe').write_bytes(pe())
        self.sdl = self.base/'sdl'; self.sdl.mkdir()
        (self.sdl/'SDL2.dll').write_bytes(pe(True)); (self.sdl/'SDL2-LICENSE.txt').write_bytes(b'SDL license')
        self.assets = self.base/'assets'; self.assets.mkdir()
        pins = []
        for name in GAME_FILES:
            data = ('synthetic '+name).encode(); (self.assets/name).write_bytes(data)
            pins.append(dict(name=name,size=len(data),sha256=hashlib.sha256(data).hexdigest()))
        manifests = self.root/'assets/manifests'; manifests.mkdir(parents=True, exist_ok=True)
        (manifests/'dos_original.json').write_text(json.dumps(dict(profile='synthetic tests',sources=[],files=pins)))
        sdlpins = [dict(name=n,size=len((self.sdl/n).read_bytes()),sha256=hashlib.sha256((self.sdl/n).read_bytes()).hexdigest())
                   for n in ['SDL2.dll','SDL2-LICENSE.txt']]
        (manifests/'windows_runtime.json').write_text(json.dumps(dict(files=sdlpins)))
        from tools.tests.test_hd_delivery import install_fixture
        install_fixture(self.root)
        names = sorted(p.relative_to(self.root).as_posix() for p in self.root.rglob('*') if p.is_file())
        (self.root/'SOURCE-MANIFEST.json').write_text(json.dumps([dict(path=n) for n in names]))

    def package(self, name='dist'):
        return create(self.runtime, self.assets, self.sdl, self.base/name, root=self.root)

    def test_complete_game_data_dll_launcher_source_and_hashes_are_included(self):
        (self.assets/'INSTALL.EXE').write_bytes(b'unused DOS installer')
        (self.runtime/'secret.txt').write_bytes(b'private')
        path = self.package(); result = verify(path, root=self.root)
        self.assertEqual(Path(str(path)+'.sha256').read_bytes(),
                         f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n'.encode('ascii'))
        self.assertEqual(result['game_files'], 28)
        self.assertTrue(result['original_game_data'])
        with zipfile.ZipFile(path) as archive:
            self.assertIn('OhMyLemmings.exe', archive.namelist())
            self.assertIn('start-game.cmd', archive.namelist())
            self.assertIn('source.tar.gz', archive.namelist())
            self.assertEqual(archive.read('HD-PRESENTATION'), b'hd\n')
            self.assertTrue(json.loads(archive.read('PACKAGE-MANIFEST.json'))['hd_remaster'])
            self.assertNotIn('data/INSTALL.EXE', archive.namelist())
            self.assertNotIn('secret.txt', archive.namelist())
            for name in GAME_FILES: self.assertEqual(archive.read('data/'+name), (self.assets/name).read_bytes())

    def test_windows_delivers_both_project_and_third_party_notices(self):
        path = self.package()
        with zipfile.ZipFile(path) as archive:
            self.assertEqual(json.loads(archive.read('PACKAGE-MANIFEST.json'))['project_license'], 'MIT')
            for name in ['LICENSE', 'THIRD_PARTY_NOTICES.txt']:
                self.assertEqual(archive.read(name), (self.root / name).read_bytes())

    def test_missing_corrupt_or_linked_game_data_fails_before_creating_package(self):
        path = self.assets/'MAIN.DAT'; data = path.read_bytes(); path.unlink()
        with self.assertRaises(ValueError): self.package()
        path.write_bytes(data+b'changed')
        with self.assertRaises(ValueError): self.package()
        path.unlink(); target = self.base/'other'; target.write_bytes(data); path.symlink_to(target)
        with self.assertRaises(ValueError): self.package()
        self.assertFalse((self.base/'dist').exists())

    def test_wrong_pe_architecture_or_missing_sdl_fails(self):
        path = self.runtime/'lemmings-desktop.exe'; path.write_bytes(b'not PE')
        with self.assertRaises(ValueError): self.package()
        data = bytearray(pe()); data[132:134] = (0x14c).to_bytes(2,'little'); path.write_bytes(data)
        with self.assertRaises(ValueError): self.package()
        path.write_bytes(pe()); (self.sdl/'SDL2.dll').unlink()
        with self.assertRaises(ValueError): self.package()

    def test_missing_or_modified_executable_icon_fails_packaging(self):
        path = self.runtime/'lemmings-desktop.exe'; data = pe()
        path.write_bytes(data[:512])
        with self.assertRaises(ValueError): self.package()
        path.write_bytes(data[:-1] + bytes([data[-1] ^ 255]))
        with self.assertRaises(ValueError): self.package()

    def test_identical_inputs_are_deterministic_and_evidence_is_not_overwritten(self):
        first = self.package('first'); second = self.package('second')
        self.assertEqual(first.read_bytes(), second.read_bytes())
        with self.assertRaises(FileExistsError): self.package('first')

    def test_tampered_zip_or_original_asset_is_rejected(self):
        path = self.package()
        with zipfile.ZipFile(path) as archive: files = {n:archive.read(n) for n in archive.namelist()}
        files['data/MAIN.DAT'] += b'corruption'
        from tools.release_package import archive_bytes
        path.write_bytes(archive_bytes(files,'web'))
        Path(str(path)+'.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest()+'  '+path.name+'\n')
        with self.assertRaises(ValueError): verify(path, root=self.root)
