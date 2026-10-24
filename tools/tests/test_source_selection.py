import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from tools.tests import test_web_build as fixtures
from tools.web_build import source_bundle

class SourceSelectionTests(unittest.TestCase):
    def test_only_live_art_and_required_provenance_ship_not_history_or_previews(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'repo';root.mkdir();sub=fixtures.AudioSourceDeliveryTests().fixture(root)
            files={'assets/remaster/live.png':b'live','assets/remaster/old.png':b'old',
                   'assets/remaster/previews/current.png':b'preview','mydocs/reports/old.png':b'report',
                   'assets/remaster/prompts/live.txt':b'prompt','assets/remaster/extension.png':b'extra',
                   'assets/manifests/old_observations.jsonl':b'historical measurements','README.en.md':b'English instructions','third_party/notice.txt':b'license'}
            for n,b in files.items():
                p=root/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
            (root/'assets/manifests/remaster_pack.json').write_text(json.dumps(dict(atlases=[dict(path='assets/remaster/live.png')],provenance=[dict(path='assets/remaster/prompts/live.txt')],catalog_extensions=[dict(path='assets/manifests/extension.json')])))
            (root/'assets/manifests/extension.json').write_text(json.dumps(dict(atlases=[dict(path='assets/remaster/extension.png')],provenance=[])))
            sub.run(['git','-C',str(root),'add','.'],check=True)
            out=Path(d)/'source.tar.gz';source_bundle(out,root)
            with tarfile.open(out) as t:
                names=t.getnames()
                for n in ['assets/remaster/live.png','assets/remaster/extension.png','assets/remaster/prompts/live.txt','README.en.md','third_party/notice.txt']:self.assertIn(n,names)
                for n in ['assets/remaster/old.png','assets/remaster/previews/current.png','mydocs/reports/old.png','assets/manifests/old_observations.jsonl']:self.assertNotIn(n,names)
