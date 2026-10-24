"""Verify the extracted Native executable's local MP3 mixer and SDL lifecycle."""
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def check(binary, evidence=None):
    env = dict(os.environ, SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy', SDL_RENDER_DRIVER='software')
    fixture = ROOT / 'apps/desktop/tests/fixtures/local-tone.mp3'
    result = subprocess.run([str(binary), '--local-music-probe', str(fixture)], env=env,
                            capture_output=True, text=True, check=True, timeout=30)
    row = json.loads(result.stdout)
    if (row.get('status') != 'PASS' or any(row.get(key) is not True for key in
            ['stereo', 'device_consumed', 'hide_clears_queue', 'modes', 'dialog_pause_preserves_selection'])
            or type(row.get('peak')) is not int or not 0 < row['peak'] <= 32767):
        raise ValueError('Packaged local MP3 audio path incomplete')
    if evidence:
        (evidence / 'local-music.json').write_text(json.dumps(row, indent=2) + '\n')
        (evidence / 'local-music.log').write_text(result.stderr)
    return row
