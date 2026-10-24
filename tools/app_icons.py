#!/usr/bin/env python3
"""Generate the project-owned vector app icon, or verify its pinned platform forms."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.hd_assets import decode_png

DIRECTORY = 'apps/desktop/resources/'
SIZES = (16, 24, 32, 48, 64, 128, 256)
MANIFEST = 'assets/manifests/app_icon.json'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def ico_images(data):
    require(len(data) >= 6 and struct.unpack_from('<HH', data) == (0, 1), 'Invalid ICO header')
    count = struct.unpack_from('<H', data, 4)[0]
    require(count == len(SIZES) and len(data) >= 6 + 16 * count, 'Invalid ICO inventory')
    result = {}
    end = 6 + 16 * count
    for index in range(count):
        w, h, colors, reserved, planes, bits, size, offset = struct.unpack_from('<BBBBHHII', data, 6 + 16 * index)
        w = w or 256; h = h or 256
        require(w == h and w in SIZES and w not in result and (colors, reserved, planes, bits) == (0, 0, 1, 32)
                and offset == end and offset + size <= len(data), 'Invalid ICO image entry')
        png = data[offset:offset + size]
        width, height, _ = decode_png(png)
        require((width, height) == (w, h), 'ICO image size differs')
        result[w] = png; end += size
    require(end == len(data), 'Trailing ICO data')
    return result


def icon_bytes(root, name):
    """Validate one build/package input against the recorded source inventory."""
    info = json.loads((root / MANIFEST).read_text())
    path = DIRECTORY + name
    pins = [p for p in info['files'] if p['path'] == path]
    require(len(pins) == 1, 'Missing app icon pin')
    pin = pins[0]; source = root / path
    require(source.is_file() and not source.is_symlink() and not any(p.is_symlink() for p in source.parents), 'Missing or linked app icon')
    data = source.read_bytes()
    require(len(data) == pin['bytes'] and hashlib.sha256(data).hexdigest() == pin['sha256'], 'App icon checksum differs')
    return data


def verify_pe_icon(data, ico):
    """Read PE32+ resource directories and compare actual RT_ICON pixels to ICO."""
    def unpack(fmt, offset):
        require(0 <= offset and offset + struct.calcsize(fmt) <= len(data), 'Truncated PE resource')
        return struct.unpack_from(fmt, data, offset)
    require(data[:2] == b'MZ', 'Missing PE header')
    pe = unpack('<I', 60)[0]
    require(unpack('<4s', pe)[0] == b'PE\0\0' and unpack('<H', pe + 24)[0] == 0x20b, 'Expected PE32+')
    sections = unpack('<H', pe + 6)[0]; optional = unpack('<H', pe + 20)[0]
    require(1 <= sections <= 96 and optional >= 136 and unpack('<I', pe + 24 + 108)[0] >= 3, 'Missing PE resource table')
    resource_rva, resource_size = unpack('<II', pe + 24 + 128)
    require(resource_rva and 16 <= resource_size <= len(data), 'Missing PE resources')
    mapping = []
    for index in range(sections):
        _, rva, size, start = unpack('<IIII', pe + 24 + optional + index * 40 + 8)
        require(start + size <= len(data), 'Invalid PE section size')
        mapping.append((rva, size, start))
    def position(rva, size):
        for base, length, start in mapping:
            if base <= rva and rva + size <= base + length:
                return start + rva - base
        raise ValueError('PE resource RVA outside section')
    base = position(resource_rva, resource_size)
    def relative(offset, size):
        require(0 <= offset and offset + size <= resource_size, 'PE resource outside directory')
        return base + offset
    def directory(offset):
        header = relative(offset, 16)
        named, ids = unpack('<HH', header + 12)
        count = named + ids
        require(count <= 4096, 'Too many PE resources')
        relative(offset + 16, count * 8)
        return [unpack('<II', header + 16 + i * 8) for i in range(count)]
    entries = {}
    for kind, kind_offset in directory(0):
        if kind not in (3, 14): continue
        require(kind_offset & 0x80000000, 'Missing PE resource type directory')
        for identifier, id_offset in directory(kind_offset & 0x7fffffff):
            require(not identifier & 0x80000000 and id_offset & 0x80000000, 'Invalid PE icon ID')
            for language, value_offset in directory(id_offset & 0x7fffffff):
                require(not value_offset & 0x80000000, 'Invalid PE icon data entry')
                value_rva, size, _, _ = unpack('<IIII', relative(value_offset, 16))
                start = position(value_rva, size)
                key = (kind, identifier, language)
                require(key not in entries, 'Duplicate PE icon')
                entries[key] = data[start:start + size]
    groups = [(lang, value) for (kind, identifier, lang), value in entries.items() if kind == 14 and identifier == 101]
    require(bool(groups), 'Executable has no application icon group101')
    expected = ico_images(ico)
    for language, group in groups:
        require(len(group) == 6 + 14 * len(expected) and struct.unpack_from('<HHH', group) == (0, 1, len(expected)), 'Incomplete executable icon group')
        sizes = set()
        for index in range(len(expected)):
            w, h, colors, reserved, planes, bits, size, identifier = struct.unpack_from('<BBBBHHIH', group, 6 + 14 * index)
            w = w or 256; h = h or 256
            require(w == h and w in expected and w not in sizes and (colors, reserved, planes, bits) == (0, 0, 1, 32), 'Invalid executable icon size/depth')
            png = entries.get((3, identifier, language))
            require(png == expected[w] and size == len(png), 'Executable icon differs from project artwork')
            sizes.add(w)
    return dict(status='PASS', resource_group=101, sizes=list(expected))


def verify(root=ROOT):
    info = json.loads((root / MANIFEST).read_text())
    require(info['schema_version'] == 1 and info['license'] == 'MIT', 'Invalid icon provenance')
    expected = {DIRECTORY + n for n in ['app-icon.svg', 'app-icon.png', 'app-icon.ico', 'app-icon.icns', 'app-icon.rgba']}
    require({pin['path'] for pin in info['files']} == expected and len(info['files']) == len(expected), 'Invalid icon inventory')
    files = {}
    for pin in info['files']:
        path = root / pin['path']
        require(path.is_file() and not path.is_symlink() and not any(p.is_symlink() for p in path.parents), 'Missing or linked icon')
        data = path.read_bytes()
        require(len(data) == pin['bytes'] and hashlib.sha256(data).hexdigest() == pin['sha256'], 'App icon checksum differs')
        files[path.name] = data
    images = ico_images(files['app-icon.ico'])
    require(images[256] == files['app-icon.png'], 'PNG and Windows icon differ')
    _, _, rgba = decode_png(images[64])
    require(rgba == files['app-icon.rgba'] and any(rgba[3::4]) and 0 in rgba[3::4], 'SDL RGBA icon differs')
    icns = files['app-icon.icns']
    require(icns[:4] == b'icns' and struct.unpack_from('>I', icns, 4)[0] == len(icns), 'Invalid ICNS header')
    pos = 8; found = {}
    expected_icns = {b'icp4': 16, b'icp5': 32, b'icp6': 64, b'ic07': 128, b'ic08': 256, b'ic09': 512, b'ic10': 1024}
    while pos < len(icns):
        require(pos + 8 <= len(icns), 'Truncated ICNS')
        kind, size = struct.unpack_from('>4sI', icns, pos)
        require(kind in expected_icns and kind not in found and size > 8 and pos + size <= len(icns), 'Invalid ICNS entry')
        w, h, _ = decode_png(icns[pos + 8:pos + size])
        require(w == h == expected_icns[kind], 'ICNS image size differs')
        if w in images: require(icns[pos + 8:pos + size] == images[w], 'Mac and Windows icon differ')
        found[kind] = w; pos += size
    require(set(found) == set(expected_icns), 'Missing Mac icon representations')
    return dict(status='PASS', windows_sizes=list(images), mac_sizes=sorted(found.values()), sdl_size=64)


def generate(root=ROOT):
    folder = root / DIRECTORY
    source = folder / 'app-icon.svg'
    pngs = {size: subprocess.check_output(['rsvg-convert', '--width', str(size), '--height', str(size), str(source)])
            for size in (*SIZES, 512, 1024)}
    ico = bytearray(struct.pack('<HHH', 0, 1, len(SIZES)))
    offset = 6 + 16 * len(SIZES)
    for size in SIZES:
        png = pngs[size]
        ico.extend(struct.pack('<BBBBHHII', size % 256, size % 256, 0, 0, 1, 32, len(png), offset)); offset += len(png)
    ico.extend(b''.join(pngs[n] for n in SIZES))
    chunks = b''.join(struct.pack('>4sI', kind, len(pngs[size]) + 8) + pngs[size]
                      for kind, size in [(b'icp4', 16), (b'icp5', 32), (b'icp6', 64), (b'ic07', 128), (b'ic08', 256), (b'ic09', 512), (b'ic10', 1024)])
    outputs = {'app-icon.png': pngs[256], 'app-icon.ico': bytes(ico),
               'app-icon.icns': b'icns' + struct.pack('>I', len(chunks) + 8) + chunks,
               'app-icon.rgba': decode_png(pngs[64])[2]}
    for name, data in outputs.items(): (folder / name).write_bytes(data)
    names = sorted(['app-icon.svg', *outputs])
    info = dict(schema_version=1, issue='https://github.com/edwardkim/ohmylemmings/issues/208',
                license='MIT', author='Oh My Lemmings contributors',
                provenance='Original vector drawing; no original DOS pixels, font or downloaded artwork',
                generator='python3 tools/app_icons.py generate',
                renderer=subprocess.check_output(['rsvg-convert', '--version'], text=True).strip(),
                files=[dict(path=DIRECTORY + n, bytes=(folder/n).stat().st_size,
                            sha256=hashlib.sha256((folder/n).read_bytes()).hexdigest()) for n in names])
    (root / MANIFEST).write_text(json.dumps(info, indent=2) + '\n')
    return verify(root)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['generate', 'verify'])
    args = parser.parse_args()
    print(json.dumps(generate() if args.command == 'generate' else verify()))
