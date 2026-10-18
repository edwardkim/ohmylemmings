import importlib.util
from pathlib import Path
import struct
import unittest

SPEC = importlib.util.spec_from_file_location('dos_memory', Path(__file__).parents[1] / 'dos_memory.py')
memory = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(memory)


class DosMemoryTests(unittest.TestCase):
    def image(self):
        exe = bytearray(512)
        exe[:2] = b'MZ'
        struct.pack_into('<H', exe, 8, 2)
        exe[32:35] = b'\xba\x20\x00'
        exe[256:272] = b'SYNTHETIC-ANCHOR!'
        ram = bytearray(2*1024*1024)
        ram[0x1000:0x1000+480] = exe[32:]
        struct.pack_into('<H', ram, 0x1001, 0x120)
        return exe, ram

    def test_relocated_mz_identifies_ram_and_data_without_writing(self):
        exe, ram = self.image()
        before = bytes(ram)
        found = memory.locate_ram(bytes(ram), 0x10000000, bytes(exe), 256)
        self.assertEqual(found, (0x10000000, 0x1200))
        self.assertEqual(bytes(ram), before)

    def test_missing_duplicate_or_invalid_relocation_is_rejected(self):
        exe, ram = self.image()
        self.assertIsNone(memory.locate_ram(bytes(len(ram)), 0, bytes(exe), 256))
        struct.pack_into('<H', ram, 0x1001, 0x10)
        self.assertIsNone(memory.locate_ram(bytes(ram), 0, bytes(exe), 256))
        exe, ram = self.image()
        # Two complete candidate RAM images must not be silently disambiguated.
        with self.assertRaises(ValueError):
            memory.locate_ram(bytes(ram)+bytes(ram), 0, bytes(exe), 256)

    def test_bad_executable_or_truncated_region_cannot_be_located(self):
        exe, ram = self.image()
        for bad in [b'', b'PE'+bytes(exe[2:]), bytes(exe[:100])]:
            with self.assertRaises(ValueError):
                memory.locate_ram(bytes(ram), 0, bad, 256)
        self.assertIsNone(memory.locate_ram(bytes(ram[:0x2000]), 0, bytes(exe), 256))

    def test_sample_decodes_only_documented_fields_and_all_object_slots(self):
        data = bytearray(0x2200)
        data[0x52], data[0x43], data[0x55] = 55, 1, 255
        struct.pack_into('<H', data, 0x1fc9, 17045)
        data[0x17cb+2], data[0x17cb+28+2], data[0x17cb+56+2] = 0, 0, 2
        sample = memory.decode_sample(bytes(data))
        self.assertEqual(sample['startup_counter_u8'], 55)
        self.assertEqual(sample['released_u8'], 1)
        self.assertEqual(sample['pit_divisor_u16'], 17045)
        self.assertEqual(sample['object_frames_u8'][:3], [0, 0, 2])
        with self.assertRaises(ValueError):
            memory.decode_sample(b'')
