"""Exercise the actual JavaScript feedback module offline."""
from pathlib import Path
import subprocess
import shutil
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FrameFeedbackTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'), 'Node required for browser feedback contract')
    def test_real_statistics_and_feedback(self):
        subprocess.run(['node', 'tools/web/frame_feedback.cjs'], cwd=ROOT, check=True,
                       capture_output=True, text=True)
