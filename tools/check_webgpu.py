#!/usr/bin/env python3
"""Verify real WebGPU pixels and recovery in an isolated browser profile.

Use --runtime with a built development Web directory. No browser flags are
changed globally; optional --browser-arg applies only to the disposable process.
"""
import argparse
import functools
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[1]


def check(runtime, evidence, browser=None, arguments=()):
    if not evidence.is_relative_to(ROOT/'assets/local') or evidence.is_relative_to(runtime):
        raise ValueError('Use a new assets/local evidence folder outside the runtime')
    evidence.mkdir(parents=True, exist_ok=False)
    for name in ['webgpu_browser.html', 'webgpu_browser.js']:
        shutil.copyfile(ROOT/'tools/web'/name, evidence/name)
    for source in runtime.iterdir():
        if source.name not in {'webgpu_browser.html', 'webgpu_browser.js'}:
            (evidence/source.name).symlink_to(source.resolve())
    result = {}
    ready = threading.Event()

    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            if self.path not in {'/__result', '/__screenshot'}:
                self.send_error(404)
                return
            size = int(self.headers.get('Content-Length', '0'))
            if size < 1 or size > (16000000 if self.path == '/__screenshot' else 100000):
                self.send_error(413)
                return
            data = self.rfile.read(size)
            if self.path == '/__screenshot':
                if not data.startswith(b'\x89PNG\r\n\x1a\n'):
                    self.send_error(400)
                    return
                (evidence/'hd-webgpu.png').write_bytes(data)
            else:
                result.update(json.loads(data))
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'OK')
            if self.path == '/__result':
                ready.set()

    browser = browser or shutil.which('chromium') or shutil.which('google-chrome')
    if not browser:
        raise ValueError('Chromium/Chrome required')
    with ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=evidence)) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-webgpu-') as profile, (evidence/'browser.log').open('w') as log:
            command = [browser, '--headless', '--no-first-run', '--no-default-browser-check',
                       f'--user-data-dir={profile}', *arguments,
                       f'http://127.0.0.1:{server.server_port}/webgpu_browser.html']
            process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
            try:
                for _ in range(120):
                    if ready.wait(1):
                        break
                    if process.poll() is not None:
                        raise RuntimeError('Browser exited; see browser.log')
                if not ready.is_set():
                    raise TimeoutError('WebGPU verification timed out; see browser.log')
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                server.shutdown()
                thread.join(timeout=5)
    result.update(browser=subprocess.check_output([browser, '--version'], text=True).strip(),
                  browser_arguments=list(arguments))
    (evidence/'summary.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))
    if result.get('status') != 'PASS':
        raise AssertionError(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--browser')
    parser.add_argument('--browser-arg', action='append', default=[])
    args = parser.parse_args()
    check(args.runtime.resolve(), args.evidence.resolve(), args.browser, args.browser_arg)


if __name__ == '__main__':
    main()
