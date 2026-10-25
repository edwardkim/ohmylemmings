"""Run the real browser renderer's selection and draw-list contracts offline."""
from pathlib import Path
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class WebGpuTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'), 'Node required for WebGPU contracts')
    def test_selection_recovery_and_draw_list(self):
        result = subprocess.run(['node', 'tools/web/webgpu_check.mjs'],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout+result.stderr)
