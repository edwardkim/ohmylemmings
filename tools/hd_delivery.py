"""One verified HD presentation profile shared by all downloadable packages."""
import hashlib
import json
import struct

from tools.hd_assets import runtime_pack
from tools.hd_catalog import runtime_catalog, select_parts


def catalog_files(root):
    return runtime_catalog(root, runtime_pack(root=root))


def delivery_files(root, *, web=False):
    members = catalog_files(root)
    keys = set()
    for name, data in members.items():
        if name == 'catalog.hdi':
            continue
        count = struct.unpack_from('<H', data, 10)[0]
        for offset in range(len(data)-count*32, len(data), 32):
            key = data[offset:offset+4]
            if key in keys:
                raise ValueError('Duplicate delivered HD key')
            keys.add(key)
    sizes = [sum(4+len(part) for part in select_parts(members, 1|(1<<(theme+1))|special))+12
             for theme in range(5) for special in [0,64,128,256,512]]
    info = dict(schema_version=1, default='hd', original_available=True,
                catalog='hd-assets/catalog.hdi', logical_keys=len(keys),
                actor_frames=sum(key[0] == 0 for key in keys),
                object_frames=sum(key[0] == 2 for key in keys),
                max_selected_bytes=max(sizes), selected_limit_bytes=128*1024*1024,
                font='Noto Sans SemiBold (embedded, SIL OFL 1.1)', glyphs=95,
                notices='THIRD_PARTY_NOTICES.txt',
                source_manifest='assets/manifests/remaster_pack.json',
                files=[dict(path='hd-assets/'+name, bytes=len(data),
                            sha256=hashlib.sha256(data).hexdigest())
                       for name, data in sorted(members.items())])
    delivered={'hd-assets/'+name: data for name, data in members.items()}
    if web:
        from tools.web_hd import delivery
        delivered=delivery(members)
        info.update(catalog='HD-WEB.json',transport='progressive-gzip-v1',
                    files=[dict(path=name,bytes=len(data),sha256=hashlib.sha256(data).hexdigest()) for name,data in sorted(delivered.items())])
    return {**delivered,
            'HD-PRESENTATION': b'hd\n',
            'HD-ASSETS.json': (json.dumps(info, indent=2, sort_keys=True)+'\n').encode()}


def verify_delivery(files, root, prefix='', *, web=False):
    expected = delivery_files(root, web=web)
    for name, data in expected.items():
        if files.get(prefix+name) != data:
            raise ValueError('HD delivery member differs: '+prefix+name)
    return {prefix+name for name in expected}
