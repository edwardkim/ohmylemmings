import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

from tools.app_icons import ROOT, DIRECTORY, MANIFEST, SIZES, ico_images, verify, verify_pe_icon


def install_fixture(root):
    for name in [MANIFEST, *(DIRECTORY + n for n in ['app-icon.svg', 'app-icon.png', 'app-icon.ico', 'app-icon.icns', 'app-icon.rgba'])]:
        target = root / name; target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / name, target)


def icon_pe():
    """Synthetic x64 PE resource tree with real project icon bytes; never executed."""
    images = ico_images((ROOT / DIRECTORY / 'app-icon.ico').read_bytes())
    # Resource directories: type -> integer ID -> language -> data entry.
    blob = bytearray()
    def allocate(size):
        start = len(blob); blob.extend(bytes(size)); return start
    def directory(rows):
        offset = allocate(16 + len(rows) * 8)
        struct.pack_into('<HH', blob, offset + 12, 0, len(rows))
        return offset
    top = directory([3, 14]); icons = directory(images); groups = directory([101])
    struct.pack_into('<IIII', blob, top + 16, 3, icons | 0x80000000, 14, groups | 0x80000000)
    payloads = []
    for parent, entries in [(icons, [(i + 1, png) for i, png in enumerate(images.values())]),
                            (groups, [(101, struct.pack('<HHH', 0, 1, len(images)) + b''.join(
                                struct.pack('<BBBBHHIH', size % 256, size % 256, 0, 0, 1, 32, len(png), i + 1)
                                for i, (size, png) in enumerate(images.items())))])]:
        for index, (identifier, value) in enumerate(entries):
            language = directory([1033]); entry = allocate(16)
            struct.pack_into('<II', blob, parent + 16 + index * 8, identifier, language | 0x80000000)
            struct.pack_into('<II', blob, language + 16, 1033, entry)
            payloads.append((entry, value))
    for entry, value in payloads:
        offset = allocate(len(value)); blob[offset:] = value
        struct.pack_into('<IIII', blob, entry, 0x1000 + offset, len(value), 0, 0)
    data = bytearray(0x200)
    data[:2] = b'MZ'; struct.pack_into('<I', data, 60, 0x80)
    data[0x80:0x84] = b'PE\0\0'
    struct.pack_into('<HHIIIHH', data, 0x84, 0x8664, 1, 0, 0, 0, 240, 2)
    struct.pack_into('<H', data, 0x98, 0x20b)
    struct.pack_into('<I', data, 0x98 + 108, 16)
    struct.pack_into('<II', data, 0x98 + 112 + 16, 0x1000, len(blob))
    struct.pack_into('<8sIIII', data, 0x98 + 240, b'.rsrc\0\0\0', len(blob), 0x1000, len(blob), 0x200)
    return bytes(data + blob)


class AppIconTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); install_fixture(self.root)

    def test_platform_representations_match_the_source_inventory(self):
        report = verify(self.root)
        self.assertEqual(report['windows_sizes'], list(SIZES))
        self.assertEqual(report['mac_sizes'][-1], 1024)
        self.assertEqual(report['sdl_size'], 64)

    def test_missing_modified_and_linked_icon_are_rejected(self):
        path = self.root / DIRECTORY / 'app-icon.png'; data = path.read_bytes()
        path.unlink()
        with self.assertRaises(ValueError): verify(self.root)
        path.write_bytes(data + b'changed')
        with self.assertRaises(ValueError): verify(self.root)
        path.unlink(); other = self.root / 'linked.png'; other.write_bytes(data); path.symlink_to(other)
        with self.assertRaises(ValueError): verify(self.root)

    def test_executable_contains_the_complete_matching_icon_group(self):
        ico = (self.root / DIRECTORY / 'app-icon.ico').read_bytes()
        self.assertEqual(verify_pe_icon(icon_pe(), ico)['sizes'], list(SIZES))

    def test_missing_truncated_and_corrupt_pe_icon_are_rejected(self):
        ico = (self.root / DIRECTORY / 'app-icon.ico').read_bytes(); pe = icon_pe()
        for data in [b'MZ' + bytes(254), pe[:512], pe[:200], pe[:-1], pe[:-1] + bytes([pe[-1] ^ 255])]:
            with self.subTest(length=len(data)), self.assertRaises(ValueError): verify_pe_icon(data, ico)

    def test_ico_rejects_out_of_bounds_overlapping_and_truncated_images(self):
        good = (self.root / DIRECTORY / 'app-icon.ico').read_bytes()
        overlap = bytearray(good); struct.pack_into('<I', overlap, 18, 0)
        for bad in [good[:10], good[:-1], bytes(overlap), good + b'junk']:
            with self.assertRaises(ValueError): ico_images(bad)
