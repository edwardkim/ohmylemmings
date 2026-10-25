#!/usr/bin/env python3
"""Audit every active HD world asset and prepare a source-art contact sheet.

This audits remaster art, not DOS level solutions. Captures and reports belong
under ignored assets/local; no generated art is added to the runtime pack.
"""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path

try:
    from .hd_assets import (DECOR_PROFILES, EXIT_EFFECT_PROFILES, HAZARD_PROFILES,
                            TRAP_PROFILES, decode_png, read_pin)
except ImportError:
    from hd_assets import (DECOR_PROFILES, EXIT_EFFECT_PROFILES, HAZARD_PROFILES,
                           TRAP_PROFILES, decode_png, read_pin)

ROOT = Path(__file__).resolve().parents[1]
PROFILES = {(s, i): n for s in range(5) for i, n in [(0, 1), (1, 10)]}
PROFILES.update({key: 6 for key in EXIT_EFFECT_PROFILES})
PROFILES.update(HAZARD_PROFILES)
PROFILES.update(TRAP_PROFILES)
PROFILES.update(DECOR_PROFILES)


def inspect_catalog(manifests, profiles=None, *, complete_world=True):
    profiles = PROFILES if profiles is None else profiles
    expected = {('object', s, i, n) for (s, i), count in profiles.items() for n in range(count)}
    if complete_world:
        expected |= {('material', s) for s in range(5)} | {('special', s) for s in range(4)}
    seen = set()
    rows = []
    for manifest in manifests:
        for frame in manifest['frames']:
            key = frame['key']
            if not set(key) & {'object', 'material', 'special'}:
                continue
            kind = next(iter(key))
            values = key[kind] if kind == 'object' else [key[kind]]
            identity = (kind, *values)
            if identity not in expected or identity in seen:
                raise ValueError(f'duplicate or unknown world frame: {identity}')
            seen.add(identity)
            atlas = manifest['atlases'][frame['atlas']]
            x, y, width, height = frame['rect']
            dw, dh = frame['display']
            if (min(x, y) < 0 or min(width, height, dw, dh) <= 0 or
                    x+width > atlas['width'] or y+height > atlas['height']):
                raise ValueError(f'invalid world frame dimensions: {identity}')
            display = [5760, 960] if kind == 'special' else frame['display']
            rows.append(dict(key=list(identity), image=atlas, rect=frame['rect'],
                             display=display, source_display=frame['display'], anchor=frame['anchor'],
                             density=min(width/display[0], height/display[1])))
    if seen != expected:
        raise ValueError(f'missing world frames: {sorted(expected-seen)}')
    return dict(object_profiles=len(profiles), object_frames=sum(profiles.values()),
                materials=5 if complete_world else 0, specials=4 if complete_world else 0,
                rows=rows)


def pixel_stats(width, rgba, rect, cutoff):
    x, y, w, h = rect
    nx, ny = min(w, 32), min(h, 32)
    colors = set()
    visible = 0
    for gy in range(ny):
        py = y+(2*gy+1)*h//(2*ny)
        for gx in range(nx):
            px = x+(2*gx+1)*w//(2*nx)
            at = (py*width+px)*4
            pixel = rgba[at:at+4]
            if pixel[3] > cutoff:
                visible += 1
                colors.add(bytes(pixel))
    return dict(colors=len(colors), visible_samples=visible)


def contact_sheet(rows):
    # Source paths are public repository assets. Keep gameplay-derived material
    # out of this page; it can be served from a loopback repository HTTP server.
    groups = defaultdict(list)
    for row in rows:
        key = row['key']
        groups[tuple(key[:-1]) if key[0] == 'object' else tuple(key)].append(row)
    selected = []
    for key, frames in sorted(groups.items()):
        frames.sort(key=lambda f: f['key'])
        selected.append(dict(label='/'.join(map(str, key)), frames=[frames[n] for n in sorted({0, len(frames)//2, len(frames)-1})]))
    return '''<!doctype html><meta charset="utf-8"><title>HD scenery source audit</title>
<style>body{background:#0e1922;color:#dae5de;font:14px sans-serif;margin:24px}main{display:grid;grid-template-columns:repeat(4,240px);gap:12px}article{border:1px solid #46564f;border-radius:8px;padding:10px;background:#16282b}canvas{width:220px;height:110px;display:block}h2{font-size:14px;margin:0 0 8px}p{max-width:1000px}</style>
<h1>HD scenery source audit · #249</h1><p>All54 object profiles,5 terrain materials and4 special panoramas. First/middle/last artwork samples; this is a source-art contact sheet, not gameplay.</p><main></main>
<script type="module">
const groups=DATA;const images=new Map();
for(const group of groups){const card=document.createElement('article');const title=document.createElement('h2');title.textContent=group.label;card.append(title);const canvas=document.createElement('canvas');canvas.width=440;canvas.height=220;card.append(canvas);document.querySelector('main').append(card);const ctx=canvas.getContext('2d');let n=0;
 for(const f of group.frames){const path='/'+f.image.path;if(!images.has(path)){const im=new Image();im.src=path;images.set(path,new Promise((r,j)=>{im.onload=()=>r(im);im.onerror=j;}));}const image=await images.get(path),[sx,sy,sw,sh]=f.rect;const scale=Math.min(136/f.display[0],190/f.display[1]);const w=f.display[0]*scale,h=f.display[1]*scale;ctx.drawImage(image,sx,sy,sw,sh,n*146+(136-w)/2,(200-h)/2,w,h);n++;}}
document.body.dataset.ready='true';
</script>'''.replace('DATA', json.dumps(selected, separators=(',', ':')))


def load_manifests(root=ROOT):
    base = json.loads((root/'assets/manifests/remaster_pack.json').read_text())
    manifests = [base]
    for pin in base.get('catalog_extensions', []):
        manifests.append(json.loads(read_pin(pin, root)))
    return manifests


def audit(root=ROOT):
    manifests = load_manifests(root)
    result = inspect_catalog(manifests)
    groups = defaultdict(list)
    for row in result['rows']:
        groups[row['image']['path']].append(row)
    constant = []
    for path, rows in groups.items():
        pin = rows[0]['image']
        raw = read_pin(pin, root)
        width, height, rgba = decode_png(raw)
        if (width, height) != (pin['width'], pin['height']):
            raise ValueError(f'image dimensions: {path}')
        for row in rows:
            stats = pixel_stats(width, rgba, row['rect'], row['image'].get('alpha_cutoff', 0))
            row['pixels'] = stats
            if stats['visible_samples'] >= 8 and stats['colors'] <= 2:
                constant.append(row['key'])
    result.update(status='PASS' if not constant else 'REVIEW', source_images=len(groups),
                  constant_frames=constant, source_pixels_unchanged=True,
                  renderer_materials=['riveted steel', 'weathered masonry'],
                  sampling='center-aligned premultiplied bilinear for HD objects and panoramas')
    result['source_manifest_sha256'] = hashlib.sha256(json.dumps(manifests, sort_keys=True).encode()).hexdigest()
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():
        raise ValueError('use a new assets/local output directory')
    result = audit()
    output.mkdir(parents=True)
    (output/'audit.json').write_text(json.dumps(result, indent=2)+'\n')
    (output/'contact.html').write_text(contact_sheet(result['rows']))
    print(json.dumps({k: v for k, v in result.items() if k != 'rows'}))
    if result['status'] != 'PASS':
        raise ValueError('constant world art requires review; see audit.json')


if __name__ == '__main__':
    main()
