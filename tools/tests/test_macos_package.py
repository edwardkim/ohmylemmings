import hashlib
import json
from pathlib import Path
import plistlib
import struct
import unittest
import zipfile

from tools.tests import test_windows_package as windows_fixture
from tools.macos_package import APP, create, stage, verify


def universal(kind=2):
    data=bytearray(0x2040)
    struct.pack_into('>II',data,0,0xcafebabe,2)
    for index,cpu in enumerate([0x01000007,0x0100000c]):
        offset=0x1000*(index+1)
        struct.pack_into('>IIIII',data,8+index*20,cpu,3,offset,64,12)
        struct.pack_into('<IIIIIIII',data,offset,0xfeedfacf,cpu,3,kind,0,0,0,0)
    return bytes(data)


class MacPackageTests(unittest.TestCase):
    def setUp(self):
        windows_fixture.WindowsPackageTests.setUp(self)
        self.binary=self.runtime/'OhMyLemmings';self.binary.write_bytes(universal())
        (self.sdl/'libSDL2-2.0.0.dylib').write_bytes(universal(6))
        (self.sdl/'SDL2-LICENSE.txt').write_bytes(b'SDL license')
        manifest=self.root/'assets/manifests/macos_runtime.json'
        manifest.write_text(json.dumps(dict(version='2.32.10',license=dict(size=11,sha256=hashlib.sha256(b'SDL license').hexdigest()))))
        self.app=stage(self.binary,self.assets,self.sdl,self.base/'stage',root=self.root)
        signature=self.app/'Contents/_CodeSignature/CodeResources';signature.parent.mkdir();signature.write_bytes(b'synthetic signature fixture')

    def package(self,folder='dist'):
        return create(self.app,self.base/folder,root=self.root)

    def test_complete_original_app_has_universal_executable_runtime_source_and_permissions(self):
        path=self.package();report=verify(path,root=self.root)
        self.assertEqual(report['game_files'],28)
        self.assertEqual(report['architectures'],['arm64','x86_64'])
        self.assertFalse(report['notarized'])
        with zipfile.ZipFile(path) as z:
            self.assertEqual(z.getinfo(APP+'/Contents/MacOS/OhMyLemmings').external_attr>>16,0o100755)
            info=plistlib.loads(z.read(APP+'/Contents/Info.plist'))
            self.assertEqual(info['CFBundleExecutable'],'OhMyLemmings')
            self.assertEqual(info['LSMinimumSystemVersion'],'13.0')
            self.assertIn(APP+'/Contents/Resources/data/ADLIB.DAT',z.namelist())
            self.assertIn('source.tar.gz',z.namelist())
            self.assertIn('시험',z.read('README.txt').decode())

    def test_missing_corrupt_data_or_linked_binary_is_rejected(self):
        path=self.app/'Contents/Resources/data/MAIN.DAT';path.write_bytes(b'bad')
        with self.assertRaises(ValueError):self.package()
        target=self.base/'actual';target.write_bytes(self.binary.read_bytes());self.binary.unlink();self.binary.symlink_to(target)
        with self.assertRaises(ValueError):stage(self.binary,self.assets,self.sdl,self.base/'other',root=self.root)

    def test_thin_wrong_kind_and_undeclared_app_files_are_rejected(self):
        path=self.app/'Contents/MacOS/OhMyLemmings';path.write_bytes(universal(6))
        with self.assertRaises(ValueError):self.package()
        path.write_bytes(universal());(self.app/'secret.txt').write_text('private')
        with self.assertRaises(ValueError):self.package()

    def test_app_signature_is_required_and_existing_evidence_is_preserved(self):
        sig=self.app/'Contents/_CodeSignature/CodeResources';sig.unlink()
        with self.assertRaises(ValueError):self.package()
        sig.write_bytes(b'synthetic signature fixture')
        first=self.package('first');second=self.package('second')
        self.assertEqual(first.read_bytes(),second.read_bytes())
        with self.assertRaises(FileExistsError):self.package('first')
        with self.assertRaises(FileExistsError):stage(self.binary,self.assets,self.sdl,self.base/'stage',root=self.root)

    def test_modified_archive_or_missing_execute_permission_is_rejected(self):
        path=self.package()
        with zipfile.ZipFile(path) as z: files=[(i,z.read(i.filename)) for i in z.infolist()]
        with zipfile.ZipFile(path,'w') as z:
            for info,data in files:
                if info.filename.endswith('/MacOS/OhMyLemmings'):info.external_attr=0o100644<<16
                z.writestr(info,data)
        Path(str(path)+'.sha256').write_bytes(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n'.encode())
        with self.assertRaises(ValueError):verify(path,root=self.root)
