import json
from pathlib import Path
import tempfile
import unittest
import zipfile
from tools.tests import test_release_package as linux_web
from tools.tests import test_windows_package as windows
from tools.tests import test_macos_package as macos
from tools import detached_source

class DetachedSourceTests(unittest.TestCase):
    def fixture(self, platform):
        cls={'web':linux_web.ReleasePackageTests,'linux':linux_web.ReleasePackageTests,
             'windows':windows.WindowsPackageTests,'macos':macos.MacPackageTests}[platform]
        case=cls();case.setUp();self.addCleanup(case.doCleanups)
        archive=case.package(platform) if platform in ('web','linux') else case.package()
        return case,archive

    def test_all_platforms_keep_runtime_and_modes_but_deliver_one_external_source(self):
        for platform in ['web','linux','windows','macos']:
            with self.subTest(platform=platform):
                case,archive=self.fixture(platform)
                before,modes=detached_source.read_package(archive)
                output=case.base/'detached'/archive.name
                detached_source.detach(archive,output)
                after,new_modes=detached_source.read_package(output)
                source_name='audio-source.tar.gz' if platform=='web' else 'source.tar.gz'
                self.assertNotIn(source_name,after)
                for name,data in before.items():
                    if name not in {source_name,'PACKAGE-MANIFEST.json','SHA256SUMS','REBUILD.md','README.md','README.txt','audio-source.json','audio-license.html'}:
                        self.assertEqual(after[name],data,name);self.assertEqual(new_modes[name],modes[name])
                verifier={'web':linux_web.verify_package,'linux':linux_web.verify_package,
                          'windows':windows.verify,'macos':macos.verify}[platform]
                report=verifier(output,root=case.root)
                self.assertEqual(report['source_delivery'],'external')
                self.assertEqual(report['archive_bytes'],output.stat().st_size)
                info=json.loads(after['PACKAGE-MANIFEST.json'])['source_delivery']
                source=output.parent/info['path'];source.write_bytes(b'corrupt')
                with self.assertRaises(ValueError):verifier(output,root=case.root)
                source.unlink()
                with self.assertRaises(ValueError):verifier(output,root=case.root)

    def test_tampered_runtime_and_external_origin_are_rejected(self):
        case,archive=self.fixture('web');output=case.base/'detached'/archive.name
        detached_source.detach(archive,output)
        files,modes=detached_source.read_package(output)
        files['pkg/lemmings_web_bg.wasm']+=b'changed'
        detached_source.write_package(output,files,modes)
        with self.assertRaises(ValueError):linux_web.verify_package(output,root=case.root)
        files,modes=detached_source.read_package(output)
        manifest=json.loads(files['PACKAGE-MANIFEST.json'])
        manifest['source_delivery']['url']='https://example.invalid/source.tar.gz'
        detached_source.repin(files,manifest)
        detached_source.write_package(output,files,modes)
        with self.assertRaisesRegex(ValueError,'source location'):linux_web.verify_package(output,root=case.root)
