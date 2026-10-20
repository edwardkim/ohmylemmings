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

class AudioSourceDeliveryTests(unittest.TestCase):
    def fixture(self,root):
        import subprocess
        from tools.web_build import SOURCE_REQUIRED
        subprocess.run(['git','init','-q',str(root)],check=True)
        for name in SOURCE_REQUIRED:
            p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('public source '+name+'\n')
        subprocess.run(['git','-C',str(root),'add','--',*sorted(SOURCE_REQUIRED)],check=True)
        return subprocess
    def test_delivery_contains_dirty_and_new_source_but_excludes_originals_private_files_and_links(self):
        from tools.web_build import source_bundle
        import json
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'repo';root.mkdir();subprocess=self.fixture(root)
            p=root/'crates/lemmings-opl/src/core.rs';p.write_text('modified source')
            p=root/'apps/web/src/audio.rs';p.parent.mkdir(parents=True,exist_ok=True);p.write_text('new source')
            for name in ['assets/local/original.dat','target/binary','tools/secret.env','unrelated/private.rs']:
                p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('private')
            (root/'apps/web/src/link.rs').symlink_to(root/'assets/local/original.dat')
            # Even mistakenly tracked local assets must never be redistributed.
            subprocess.run(['git','-C',str(root),'add','assets/local/original.dat'],check=True)
            out=Path(d)/'source.tar.gz';source_bundle(out,root)
            with tarfile.open(out) as archive:
                names=archive.getnames();self.assertIn('apps/web/src/audio.rs',names)
                self.assertEqual(archive.extractfile('crates/lemmings-opl/src/core.rs').read(),b'modified source')
                self.assertFalse(any('private' in n or 'assets/local' in n or n.startswith('target/') or n.endswith('secret.env') or n.endswith('link.rs') for n in names))
                manifest=json.load(archive.extractfile('SOURCE-MANIFEST.json'))
                self.assertEqual([r['path'] for r in manifest],sorted(n for n in names if n!='SOURCE-MANIFEST.json'))
                for row in manifest:self.assertEqual(hashlib.sha256(archive.extractfile(row['path']).read()).hexdigest(),row['sha256'])
    def test_source_archive_is_reproducible_and_missing_license_prevents_delivery(self):
        from tools.web_build import source_bundle
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'repo';root.mkdir();self.fixture(root)
            a=Path(d)/'a.tar.gz';b=Path(d)/'b.tar.gz';source_bundle(a,root);source_bundle(b,root);self.assertEqual(a.read_bytes(),b.read_bytes())
            (root/'crates/lemmings-opl/LICENSE').unlink()
            with self.assertRaises(ValueError):source_bundle(Path(d)/'missing.tar.gz',root)
    def test_local_license_and_source_hashes_match_the_served_wasm_delivery(self):
        from tools.web_build import audio_delivery
        import json
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'repo';root.mkdir();self.fixture(root)
            out=Path(d)/'web';(out/'pkg').mkdir(parents=True);(out/'pkg/lemmings_web_bg.wasm').write_bytes(b'synthetic wasm')
            audio_delivery(out,root)
            row=json.loads((out/'audio-source.json').read_text())
            self.assertEqual(row['wasm_sha256'],hashlib.sha256(b'synthetic wasm').hexdigest())
            self.assertEqual(row['archive_sha256'],hashlib.sha256((out/'audio-source.tar.gz').read_bytes()).hexdigest())
            self.assertEqual((out/'LGPL-2.1-or-later.txt').read_bytes(),(root/'crates/lemmings-opl/LICENSE').read_bytes())
    def test_extracted_source_supports_rebuilding_modified_library_without_git(self):
        from tools.web_build import source_bundle
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'repo';root.mkdir();self.fixture(root)
            a=Path(d)/'source.tar.gz';source_bundle(a,root)
            unpacked=Path(d)/'unpacked';unpacked.mkdir()
            with tarfile.open(a) as archive:archive.extractall(unpacked,filter='data')
            self.assertFalse((unpacked/'.git').exists())
            (unpacked/'crates/lemmings-opl/src/core.rs').write_text('user modified library')
            (unpacked/'crates/lemmings-opl/src/new.rs').write_text('new user module')
            b=Path(d)/'modified.tar.gz';source_bundle(b,unpacked)
            with tarfile.open(b) as archive:
                self.assertEqual(archive.extractfile('crates/lemmings-opl/src/core.rs').read(),b'user modified library')
                self.assertIn('crates/lemmings-opl/src/new.rs',archive.getnames())
