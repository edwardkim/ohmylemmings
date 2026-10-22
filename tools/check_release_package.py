#!/usr/bin/env python3
"""Install a verified development archive into a temporary folder and smoke-test it."""
import argparse
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


def check_web(directory, browser=None):
    browser = browser or next((shutil.which(n) for n in ['google-chrome', 'chromium', 'chromium-browser'] if shutil.which(n)), None)
    require(browser is not None, 'Chrome/Chromium is required for package smoke')
    ready = threading.Event()
    result = {}
    helpers = {
        '/__package_check.js': ROOT / 'tools/web/package_check.js',
        '/__dispatch_check.js': ROOT / 'tools/web/dispatch_check.js',
        '/dispatch_events.js': ROOT / 'tools/performance/dispatch_events.js',
        '/measure.js': ROOT / 'tools/performance/measure.js',
    }

    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_GET(self):
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
            if self.path != '/__result' or not 0 < size < 100000:
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
                       f'http://127.0.0.1:{server.server_port}/__check.html']
            with tempfile.TemporaryFile(mode='w+') as log:
                process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                try:
                    for _ in range(60):
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
    with tempfile.TemporaryDirectory(prefix='lemmings-package-install-') as temporary:
        directory = Path(temporary)
        if package['target'] == 'web':
            with zipfile.ZipFile(path) as archive:
                archive.extractall(directory)
            smoke = check_web(directory, browser)
        else:
            with tarfile.open(path) as archive:
                archive.extractall(directory, filter='data')
            smoke = check_native(directory)
    return dict(package=package, smoke=smoke)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--browser')
    args = parser.parse_args()
    print(json.dumps(check(args.archive, args.browser)))
