import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.hd_catalog import split_pack
from tools.tests import test_hd_catalog as fixture
from tools.hd_delivery import delivery_files, verify_delivery


def install_fixture(root, runtime=None):
    """Tiny complete theme catalog; no DOS binaries or production art needed."""
    from tools.tests.test_hd_assets import png
    from tools.hd_assets import runtime_pack
    from tools.hd_catalog import runtime_catalog, write_files
    import hashlib
    data = png(color=2)
    (root/'art.png').write_bytes(data)
    pin = dict(path='art.png', bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
               width=2, height=2, alpha='opaque')
    manifest = dict(schema_version=1, atlases=[pin], frames=[
        dict(key=key, atlas=0, rect=[0,0,2,2], display=[12,12], anchor=[0,0])
        for key in [dict(actor=[0,0])]+[dict(material=n) for n in range(5)]])
    path = root/'assets/manifests/remaster_pack.json'
    path.parent.mkdir(parents=True, exist_ok=True); path.write_text(json.dumps(manifest))
    if runtime is not None:
        base=runtime_pack(root=root);(runtime/'hd-assets.hdp').write_bytes(base)
        write_files(runtime_catalog(root,base),runtime/'hd-assets')
        from tools.web_hd import delivery
        for name,data in delivery(runtime_catalog(root,base)).items():
            path=runtime/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)


class HdDeliveryTests(unittest.TestCase):
    def test_profile_covers_every_member_and_rejects_missing_or_rehashed_changes(self):
        members = split_pack(fixture.HdCatalogTests().pack())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch('tools.hd_delivery.catalog_files', return_value=members):
                files = delivery_files(root)
                self.assertEqual(files['HD-PRESENTATION'], b'hd\n')
                info = json.loads(files['HD-ASSETS.json'])
                self.assertEqual(info['logical_keys'], 3)
                self.assertEqual(info['default'], 'hd')
                self.assertEqual(verify_delivery(files, root), set(files))
                missing = dict(files); del missing['hd-assets/atlas-000.hdp']
                with self.assertRaises(ValueError): verify_delivery(missing, root)
                changed = dict(files); changed['hd-assets/atlas-000.hdp'] += b'corrupt'
                with self.assertRaises(ValueError): verify_delivery(changed, root)
                changed = dict(files); changed['HD-PRESENTATION'] = b'original\n'
                with self.assertRaises(ValueError): verify_delivery(changed, root)

    def test_macos_resources_prefix_uses_the_identical_delivery(self):
        members = split_pack(fixture.HdCatalogTests().pack())
        with patch('tools.hd_delivery.catalog_files', return_value=members):
            files = delivery_files(Path('.'))
            prefixed = {'App/Resources/'+name: data for name, data in files.items()}
            self.assertEqual(verify_delivery(prefixed, Path('.'), 'App/Resources/'), set(prefixed))
