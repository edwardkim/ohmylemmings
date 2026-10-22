import copy
import json
from pathlib import Path
import unittest
import zipfile

from tools.tests import test_full_release_package as fixtures
from tools.release_package import encoded, sha
from tools.pages_package import stage, REPOSITORY


class PagesPackageTests(unittest.TestCase):
    def setUp(self):
        fixtures.FullReleasePackageTests.setUp(self)
        self.archive = fixtures.FullReleasePackageTests.package(self, 'web')
        self.inputs = {
            self.archive.name: self.archive.read_bytes(),
            self.archive.name + '.sha256': Path(str(self.archive) + '.sha256').read_bytes(),
        }
        pin = lambda name, data: dict(path=name, bytes=len(data), sha256=sha(data))
        self.manifest = dict(schema_version=1, repository=REPOSITORY, version='0.1.0', tag='v0.1.0',
                             release_commit='a' * 40, source_tree='b' * 40,
                             files=[pin(n, b) for n, b in self.inputs.items()],
                             packages=[dict(target='web', path=self.archive.name, version='0.1.0',
                                            sha256=sha(self.archive.read_bytes()))])
        self.inputs['RELEASE-MANIFEST.json'] = encoded(self.manifest)
        self.release = dict(tag_name='v0.1.0', draft=False, prerelease=False,
                            html_url=f'https://github.com/{REPOSITORY}/releases/tag/v0.1.0',
                            assets=[dict(name=n, size=len(b), digest='sha256:' + sha(b),
                                         browser_download_url=f'https://github.com/{REPOSITORY}/releases/download/v0.1.0/{n}')
                                    for n, b in self.inputs.items()])
        self.output = self.base / 'pages'

    def run_stage(self, latest='v0.1.0'):
        return stage('v0.1.0', self.release, latest, self.inputs, self.output, root=self.root)

    def repin_manifest(self):
        self.inputs['RELEASE-MANIFEST.json'] = encoded(self.manifest)
        for asset in self.release['assets']:
            value = self.inputs[asset['name']]
            asset.update(size=len(value), digest='sha256:' + sha(value))

    def test_promotion_preserves_every_released_byte_and_public_source(self):
        result = self.run_stage()
        with zipfile.ZipFile(self.archive) as archive:
            for name in archive.namelist():
                self.assertEqual((self.output / name).read_bytes(), archive.read(name))
            self.assertEqual(set(p.relative_to(self.output).as_posix() for p in self.output.rglob('*') if p.is_file()),
                             set(archive.namelist()) | {'.nojekyll', 'deployment.json'})
        self.assertEqual(result['archive_sha256'], sha(self.archive.read_bytes()))
        self.assertEqual(result['release_commit'], 'a' * 40)
        self.assertTrue((self.output / 'audio-source.tar.gz').is_file())

    def test_drafts_prereleases_and_older_releases_cannot_replace_live_game(self):
        for field in ['draft', 'prerelease']:
            self.release[field] = True
            with self.assertRaises(ValueError): self.run_stage()
            self.release[field] = False
        with self.assertRaises(ValueError): self.run_stage(latest='v0.2.0')
        self.release['tag_name'] = 'v0.2.0'
        with self.assertRaises(ValueError): self.run_stage()
        self.assertFalse(self.output.exists())

    def test_corrupt_public_download_fails_before_creating_output(self):
        self.inputs[self.archive.name] += b'corrupt'
        with self.assertRaises(ValueError): self.run_stage()
        self.assertFalse(self.output.exists())

    def test_manifest_version_hash_and_original_data_are_required(self):
        original = copy.deepcopy(self.manifest)
        for change in ['version', 'repository', 'sha256']:
            self.manifest = copy.deepcopy(original)
            if change == 'sha256': self.manifest['files'][0]['sha256'] = '0' * 64
            else: self.manifest[change] = 'wrong'
            self.repin_manifest()
            with self.assertRaises(ValueError): self.run_stage()
        self.assertFalse(self.output.exists())

    def test_missing_duplicate_or_wrong_origin_release_asset_is_rejected(self):
        original = copy.deepcopy(self.release['assets'])
        self.release['assets'].pop()
        with self.assertRaises(ValueError): self.run_stage()
        self.release['assets'] = original + [original[0]]
        with self.assertRaises(ValueError): self.run_stage()
        self.release['assets'] = copy.deepcopy(original)
        self.release['assets'][0]['browser_download_url'] = 'https://elsewhere.invalid/file'
        with self.assertRaises(ValueError): self.run_stage()

    def test_self_consistent_outer_hash_cannot_hide_missing_game_or_source(self):
        for missing in ['data/MAIN.DAT', 'audio-source.tar.gz']:
            with zipfile.ZipFile(self.archive) as archive:
                files = {n: archive.read(n) for n in archive.namelist() if n != missing}
            from tools.release_package import archive_bytes
            self.inputs[self.archive.name] = archive_bytes(files, 'web')
            data = self.inputs[self.archive.name]
            self.inputs[self.archive.name + '.sha256'] = f'{sha(data)}  {self.archive.name}\n'.encode()
            for pin in self.manifest['files']:
                value = self.inputs[pin['path']]
                pin.update(bytes=len(value), sha256=sha(value))
            self.manifest['packages'][0]['sha256'] = sha(data)
            self.repin_manifest()
            with self.assertRaises(ValueError): self.run_stage()
        self.assertFalse(self.output.exists())

    def test_existing_output_is_not_overwritten(self):
        self.run_stage()
        before = (self.output / 'deployment.json').read_bytes()
        with self.assertRaises(FileExistsError): self.run_stage()
        self.assertEqual((self.output / 'deployment.json').read_bytes(), before)
