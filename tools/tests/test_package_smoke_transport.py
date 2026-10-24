"""Exercise the package check's real HTTP result transport with a tiny browser stand-in."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from tools.check_release_package import check_web


class PackageSmokeTransportTests(unittest.TestCase):
    def run_browser(self, payload, declared=None):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            browser = root / 'browser'
            browser.write_text(f'''#!{sys.executable}
import http.client, sys, urllib.parse
url=urllib.parse.urlsplit(sys.argv[-1])
body={payload!r}.encode()
c=http.client.HTTPConnection(url.hostname,url.port,timeout=10)
c.request('POST','/__result',body,{{'Content-Length':str({declared!r} if {declared!r} is not None else len(body))}})
r=c.getresponse();r.read();c.close()
''')
            browser.chmod(0o755)
            return check_web(root, str(browser))

    def test_full_hd_capture_larger_than_two_megabytes_is_received(self):
        # Actual #215 PNG base64 was2,628,976 bytes, plus256,000 original RGB.
        result = self.run_browser(json.dumps({'status':'PASS','hd_png':'A'*2628976,'frame_rgb':'B'*256000}))
        self.assertEqual(len(result['hd_png']),2628976)
        self.assertEqual(len(result['frame_rgb']),256000)

    def test_oversized_result_reports_transport_error_instead_of_timeout(self):
        with self.assertRaisesRegex(ValueError,'result size'):
            self.run_browser('',declared=16*1024*1024+1)

    def test_malformed_result_reports_transport_error_instead_of_timeout(self):
        with self.assertRaisesRegex(ValueError,'result JSON'):
            self.run_browser('{broken')
