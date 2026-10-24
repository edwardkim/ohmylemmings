#!/usr/bin/env python3
"""Install a verified development archive into a temporary folder and smoke-test it."""
import argparse
import base64
import functools
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import zipfile

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.release_package import require, verify_package


def check_native(directory):
    binary = directory / 'lemmings-desktop'
    env = dict(os.environ, SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy', SDL_RENDER_DRIVER='software')
    capture = directory / 'smoke'
    command = [str(binary), '--synthetic-outcome', 'win', '--steps', '1200', '--fast',
               '--render-every', '7', '--result-actions', 'restart,continue', '--capture', str(capture)]
    subprocess.run(command, cwd=directory, env=env, capture_output=True, text=True, check=True, timeout=30)
    transitions = [json.loads(line) for line in (capture / 'transitions.jsonl').read_text().splitlines()]
    require([row['action'].lower() for row in transitions] == ['restart', 'continue'], 'Packaged result transitions differ')
    phases = []
    for folder in [capture, *sorted(capture.glob('attempt-*'))]:
        rows = [json.loads(line) for line in (folder / 'completion.jsonl').read_text().splitlines()]
        phases.append(rows[-1]['phase'])
    require(phases == ['Won', 'Won', 'Lost'], 'Packaged completion/restart differs')
    probe = subprocess.run([str(binary), '--audio-probe'], cwd=directory, env=env,
                           capture_output=True, text=True, check=True, timeout=20)
    from tools.check_audio_output import validate_sdl
    audio = json.loads(probe.stdout)
    validate_sdl(audio)
    return dict(status='PASS', outcomes=phases, actions=['restart', 'continue'], audio=audio,
                scope='extracted package; SDL dummy video/software and audio device; no human audibility or performance qualification')


def check_web(directory, browser=None, full=False):
    browser = browser or next((shutil.which(n) for n in ['google-chrome', 'chromium', 'chromium-browser'] if shutil.which(n)), None)
    require(browser is not None, 'Chrome/Chromium is required for package smoke')
    ready = threading.Event()
    result = {}
    helpers = {
        '/__package_check.js': ROOT / ('tools/web/full_package_check.js' if full else 'tools/web/package_check.js'),
        '/__bundled_check.js': ROOT / 'tools/web/bundled_check.js',
        '/__dispatch_check.js': ROOT / 'tools/web/dispatch_check.js',
        '/__download_check.js': ROOT / 'tools/web/download_check.js',
        '/dispatch_events.js': ROOT / 'tools/performance/dispatch_events.js',
        '/measure.js': ROOT / 'tools/performance/measure.js',
    }

    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def translate_path(self, path):
            if full and path.startswith('/game/'):
                path = '/' + path[len('/game/'):]
            return super().translate_path(path)

        def do_GET(self):
            if full and self.path.startswith('/game/'):
                self.path = '/' + self.path[len('/game/'):]
            if self.path == '/__check.html':
                body = b'<!doctype html><p id="result">Checking</p><script type="module" src="./__package_check.js"></script>'
            elif self.path in helpers:
                body = helpers[self.path].read_bytes()
            else:
                return super().do_GET()
            self.send_response(200)
            self.send_header('Content-Type', 'text/html' if self.path.endswith('.html') else 'text/javascript')
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):
            size = int(self.headers.get('Content-Length', '0'))
            if self.path != '/__result' or not 0 < size < 2000000:
                self.send_error(400)
                return
            result.update(json.loads(self.rfile.read(size)))
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'OK')
            ready.set()

    with ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=directory)) as server:
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-package-browser-') as profile:
            command = [browser, '--headless', '--no-first-run', '--no-default-browser-check', '--disable-gpu',
                       '--autoplay-policy=no-user-gesture-required', f'--user-data-dir={profile}',
                       f'http://127.0.0.1:{server.server_port}/{"game/" if full else ""}__check.html']
            with tempfile.TemporaryFile(mode='w+') as log:
                process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                try:
                    for _ in range(180):
                        if ready.wait(1): break
                        require(process.poll() is None, 'Package browser exited')
                    require(ready.is_set(), 'Package browser timed out')
                finally:
                    if process.poll() is None:
                        process.terminate()
                        try: process.wait(timeout=5)
                        except subprocess.TimeoutExpired: process.kill(); process.wait()
                    server.shutdown()
                    worker.join(timeout=5)
        require(result.get('status') == 'PASS', f'Packaged Web smoke failed: {result}')
    return result


def check(path, browser=None):
    package = verify_package(path)
    full = package.get('original_game_data', False)
    evidence = ROOT / 'assets/local' / ('web-package-check' if package['target'] == 'web' else 'linux-package-check')
    if full:
        evidence.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix='lemmings-package-install-', dir=evidence if full else None) as temporary:
        directory = Path(temporary)
        if package['target'] == 'web':
            with zipfile.ZipFile(path) as archive:
                archive.extractall(directory)
            smoke = check_web(directory, browser, full)
            if full:
                hd_image=smoke.pop('hd_png',None)
                if hd_image:(evidence/'hd-default.png').write_bytes(base64.b64decode(hd_image,validate=True))
                pixels = base64.b64decode(smoke.pop('frame_rgb'), validate=True)
                require(len(pixels) == 320*200*3, 'Truncated browser capture')
                (evidence / 'original-fun01.ppm').write_bytes(b'P6\n320 200\n255\n' + pixels)
        else:
            with tarfile.open(path) as archive:
                archive.extractall(directory, filter='data')
            if full:
                from tools.check_linux_game_package import check as check_game
                smoke = check_game(directory, evidence)
            else:
                smoke = check_native(directory)
    report = dict(package=package, smoke=smoke)
    if full:
        from tools.verify_native_view import png_from_ppm
        png_from_ppm(evidence / 'original-fun01.ppm')
        (evidence / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--browser')
    args = parser.parse_args()
    print(json.dumps(check(args.archive, args.browser)))
