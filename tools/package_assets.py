"""Shared original runtime inventory for platform packagers."""
import json

GAME_FILES = tuple(sorted(['ADLIB.DAT', 'MAIN.DAT', 'ODDTABLE.DAT', 'VGALEMMI.EXE']
    + [f'LEVEL{i:03}.DAT' for i in range(10)] + [f'GROUND{i}O.DAT' for i in range(5)]
    + [f'VGAGR{i}.DAT' for i in range(5)] + [f'VGASPEC{i}.DAT' for i in range(4)]))


def manifest(root, name):
    return json.loads((root/'assets/manifests'/name).read_text(encoding='utf-8'))


def game_pins(root):
    data = manifest(root, 'dos_original.json')
    by_name = {row['name']: row for row in data['files']}
    if not all(name in by_name for name in GAME_FILES):
        raise ValueError('Original-game manifest is incomplete')
    return data, [by_name[name] for name in GAME_FILES]


