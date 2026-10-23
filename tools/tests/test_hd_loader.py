import shutil,subprocess,unittest
from pathlib import Path
class HdLoaderTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'),'Node required for browser loader contract')
    def test_selection_cache_atomic_failure_and_legacy_fallback(self):
        subprocess.run(['node','tools/web/hd_loader.cjs'],cwd=Path(__file__).resolve().parents[2],check=True,capture_output=True)
