import io
import json
import tarfile
import unittest

from tools.publish_release import validate_run, verify_sources, validate_inventory, JOBS
from tools.release_package import sha


class PublishReleaseTests(unittest.TestCase):
    def test_failed_foreign_or_different_tree_ci_cannot_be_promoted(self):
        run=dict(name='CI',conclusion='success',head_repository=dict(full_name='owner/game'))
        jobs=[dict(name=n,conclusion='success') for n in JOBS]
        validate_run(run,jobs,'owner/game','same','same')
        for change in [dict(conclusion='failure'),dict(name='Other'),dict(head_repository=dict(full_name='fork/game'))]:
            with self.assertRaises(ValueError):validate_run(dict(run,**change),jobs,'owner/game','same','same')
        with self.assertRaises(ValueError):validate_run(run,jobs,'owner/game','other','same')
        with self.assertRaises(ValueError):validate_run(run,jobs[:-1],'owner/game','same','same')
        jobs[0]['conclusion']='failure'
        with self.assertRaises(ValueError):validate_run(run,jobs,'owner/game','same','same')

    def source(self,data=b'code',extra=False):
        pins=[dict(path='app.rs',bytes=len(data),sha256=sha(data))]
        out=io.BytesIO()
        with tarfile.open(fileobj=out,mode='w:gz') as archive:
            for name,body in [('app.rs',data),('SOURCE-MANIFEST.json',json.dumps(pins).encode())]+([('extra',b'x')] if extra else []):
                entry=tarfile.TarInfo(name);entry.size=len(body);archive.addfile(entry,io.BytesIO(body))
        return out.getvalue()

    def test_corresponding_source_must_equal_tag_contents_and_inventory(self):
        expected=[dict(path='app.rs',bytes=4,sha256=sha(b'code'))]
        self.assertEqual(verify_sources(self.source(),expected),1)
        for source in [self.source(b'evil'),self.source(extra=True)]:
            with self.assertRaises(ValueError):verify_sources(source,expected)
        with self.assertRaises(ValueError):verify_sources(self.source(),[])

    def test_release_requires_all_four_exact_version_archives(self):
        targets=['windows-x86_64','macos-universal2','linux-x86_64','web']
        rows=[dict(target=t,version='0.1.0') for t in targets]
        validate_inventory(rows,'0.1.0')
        with self.assertRaises(ValueError):validate_inventory(rows[:-1],'0.1.0')
        with self.assertRaises(ValueError):validate_inventory(rows+[rows[0]],'0.1.0')
        rows[0]['version']='0.0.9'
        with self.assertRaises(ValueError):validate_inventory(rows,'0.1.0')


class DraftPublicationTests(unittest.TestCase):
    def setUp(self):
        from pathlib import Path
        import tempfile
        from tools.release_package import encoded
        from tools.release_version import package_name, TARGETS
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.directory=Path(self.temp.name)
        self.files={package_name(t,'0.1.0'):b'archive' for t in TARGETS}
        self.files.update({name+'.sha256':b'checksum' for name in list(self.files)})
        self.files.update({'ohmylemmings-0.1.0-source.tar.gz':b'source','PLAY.md':b'notes'})
        info=dict(tag='v0.1.0',version='0.1.0',release_commit='head',source_tree='tree',repository='owner/game',
                  packages=[dict(target=t,version='0.1.0') for t in TARGETS],
                  files=[dict(path=n,bytes=len(b),sha256=sha(b)) for n,b in self.files.items()])
        self.files['RELEASE-MANIFEST.json']=encoded(info)
        self.files['SHA256SUMS']=''.join(f'{sha(b)}  {n}\n' for n,b in sorted(self.files.items())).encode()
        for name,data in self.files.items():(self.directory/name).write_bytes(data)
        self.calls=[];self.corrupt=False;self.tag_type='tag'

    def command(self,*args):
        from pathlib import Path
        self.calls.append(args)
        if args[:3]==('git','cat-file','-t'):return self.tag_type
        if args[:3]==('git','rev-parse','HEAD'):return 'head'
        if args[:2]==('git','rev-parse'):return 'head' if args[2].endswith('^{}') else 'object'
        if args[:2]==('git','ls-remote'):return 'object refs/tags/v0.1.0'
        if args[:3]==('gh','release','download'):
            directory=Path(args[args.index('--dir')+1])
            for name,data in self.files.items():(directory/name).write_bytes(b'bad' if self.corrupt else data)
        if args[:3]==('gh','release','view'):
            return json.dumps(dict(url='https://example.test/release',isDraft=False,isPrerelease=False,
                                   tagName='v0.1.0',assets=[dict(name=n) for n in self.files]))
        return ''

    def run_publish(self):
        from unittest.mock import patch
        from contextlib import redirect_stdout
        from tools.publish_release import publish
        assets=[dict(name=n,browser_download_url=n) for n in self.files]
        with patch('tools.publish_release.command',side_effect=self.command),patch('tools.publish_release.tree',return_value='tree'),\
             patch('tools.publish_release.check_repository',return_value=dict(version='0.1.0',prerelease=False)),\
             patch('tools.publish_release.api',return_value=dict(assets=assets)),\
             patch('tools.publish_release.urlopen',side_effect=lambda name,timeout:io.BytesIO(self.files[name])),redirect_stdout(io.StringIO()):
            publish(self.directory)

    def test_corrupt_uploaded_draft_is_never_published(self):
        self.corrupt=True
        with self.assertRaises(ValueError):self.run_publish()
        self.assertFalse(any(call[:3]==('gh','release','edit') for call in self.calls))

    def test_lightweight_tag_is_rejected_before_creating_a_release(self):
        self.tag_type='commit'
        with self.assertRaises(ValueError):self.run_publish()
        self.assertFalse(any(call[:2]==('gh','release') for call in self.calls))

    def test_matching_draft_is_published_and_public_downloads_are_checked(self):
        self.run_publish()
        create=next(i for i,call in enumerate(self.calls) if call[:3]==('gh','release','create'))
        download=next(i for i,call in enumerate(self.calls) if call[:3]==('gh','release','download'))
        edit=next(i for i,call in enumerate(self.calls) if call[:3]==('gh','release','edit'))
        self.assertLess(create,download);self.assertLess(download,edit)
        self.assertTrue((self.directory/'published.json').exists())
