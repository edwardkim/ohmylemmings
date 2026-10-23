"""Exercise a disposable extracted native HD package without repository paths."""
import json
import subprocess

from tools.release_package import require, sha


def check(binary, resources, foreign, environment, evidence):
    if not (resources/'HD-PRESENTATION').exists():
        return dict(status='legacy package; no HD profile')
    def run(label, arguments, success=True):
        result=subprocess.run([str(binary),*map(str,arguments)],cwd=foreign,env=environment,
                              capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=180)
        (evidence/('hd-'+label+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        require((result.returncode==0)==success, f'HD {label}: {result.stderr}')
        return result
    hd=evidence/'hd-default'
    first=run('default',['--steps',2,'--fast','--capture',hd])
    require('LEMMINGS_PRESENTATION mode=hd ' in first.stderr,'Packaged default is not HD')
    info=json.loads(next(line.removeprefix('LEMMINGS_HD ') for line in first.stderr.splitlines() if line.startswith('LEMMINGS_HD ')))
    require(info['ui_glyphs']==95,'HD font was not delivered')
    original=evidence/'hd-original-override'
    result=run('original',['--original','--steps',2,'--fast','--capture',original])
    require('LEMMINGS_PRESENTATION mode=original ' in result.stderr,'Original override failed')
    require((hd/'frame-000000.rgb').read_bytes()==(original/'frame-000000.rgb').read_bytes(),'HD changed logical game pixels')
    require((hd/'frame-000000.ppm').read_bytes()!=(original/'frame-000000.ppm').read_bytes(),'HD display silently fell back to original')
    replay=evidence/'hd-solution.txt';replay.write_text('60 0 AssignDigger 0\n')
    play=evidence/'hd-play';plain=evidence/'hd-plain-play'
    arguments=['--steps',1000,'--fast','--render-every',50,'--replay',replay]
    run('play',arguments+['--capture',play])
    run('plain-play',['--original',*arguments,'--capture',plain])
    states=(play/'full.jsonl').read_bytes()
    require(states==(plain/'full.jsonl').read_bytes(),'Packaged HD changed replay states')
    for path in play.glob('*.jsonl'):
        require(path.read_bytes()==(plain/path.name).read_bytes(),'Packaged HD changed replay trace: '+path.name)
    completion=json.loads((play/'completion.jsonl').read_text().splitlines()[-1])
    require(completion['phase']=='Won','Packaged HD Fun01 solution did not win')
    selected=[]
    for level in [2,4,5,6,22]:
        result=run(f'level-{level}',['--level',level,'--steps',2,'--fast'])
        require('LEMMINGS_PRESENTATION mode=hd ' in result.stderr,'Theme launch lost HD')
        selected.append(level)
    catalog=resources/'hd-assets/catalog.hdi';data=catalog.read_bytes()
    try:
        catalog.unlink()
        missing=run('missing',['--steps',1,'--fast'],False)
        require('catalog.hdi' in missing.stderr,'Missing HD error did not name the file')
        run('original-without-hd',['--original','--steps',1,'--fast'])
        catalog.write_bytes(b'corrupt')
        run('corrupt',['--steps',1,'--fast'],False)
    finally:
        catalog.write_bytes(data)
    marker=resources/'HD-PRESENTATION';marker_data=marker.read_bytes()
    try:
        marker.unlink()
        run('missing-profile',['--steps',1,'--fast'],False)
        run('original-without-profile',['--original','--steps',1,'--fast'])
    finally:
        marker.write_bytes(marker_data)
    original_data=resources/'data';hidden_data=resources/'hidden-data'
    try:
        original_data.rename(hidden_data)
        missing=run('missing-data-directory',['--steps',1,'--fast'],False)
        require(str(original_data) in missing.stderr.replace('\\\\', '\\'),'Missing install data used a development fallback')
    finally:
        hidden_data.rename(original_data)
    report=dict(status='PASS',default='hd',original_override=True,logical_frame_unchanged=True,
                displayed_frame_differs=True,themes_and_special_levels=[1]+selected,
                replay_states=len(states.splitlines()),replay_phase=completion['phase'],
                missing_and_corrupt_rejected=True,original_works_without_hd=True,missing_profile_and_data_directory_rejected=True,
                catalog_sha256=sha(data),info=info)
    (evidence/'hd-summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    return report
