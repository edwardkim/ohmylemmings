"""Offline composed fixture and platform-independent browser lifecycle checks."""
import hashlib
from pathlib import Path
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]

class LocalMusicTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'), 'Node required for local music lifecycle')
    def test_blocked_playback_replacement_cancellation_gains_and_cleanup(self):
        subprocess.run(['node', 'tools/tests/local-music.cjs'], cwd=ROOT, check=True, capture_output=True)

    def test_composed_mp3_fixture_matches_recorded_source(self):
        import json
        row = json.loads((ROOT / 'assets/manifests/local_music_fixture.json').read_text())
        data = (ROOT / row['path']).read_bytes()
        self.assertEqual(len(data), row['bytes'])
        self.assertEqual(hashlib.sha256(data).hexdigest(), row['sha256'])
        self.assertEqual(row['source'], 'Project-authored independent sine tones; no third-party recording')

if __name__ == '__main__': unittest.main()
