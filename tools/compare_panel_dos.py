#!/usr/bin/env python3
"""Compare declared VGA toolbar regions using existing, verified DOS recordings.

A matching playfield chooses an anchor, not a DOS input tick. Cursor and different
skill selection occupy the three explicitly excluded button slots (2,8,9).
"""
import argparse
import json
from pathlib import Path

from capture_dos import identity, rgb_bytes, sha
from dos_observation import game_frame
from reference_assets import ROOT, verify_assets
from verify_native_view import png_from_ppm

REGIONS = [dict(name='info', rect=[0,160,320,176]),
           dict(name='rate', rect=[0,176,32,200]),
           dict(name='five_skills', rect=[48,176,128,200]),
           dict(name='pause_nuke', rect=[160,176,192,200]),
           dict(name='map_top', rect=[192,176,320,178]),
           dict(name='minimap', rect=[208,178,312,198]),
           dict(name='map_left', rect=[192,178,208,200]),
           dict(name='map_right', rect=[312,178,320,200]),
           dict(name='map_bottom', rect=[208,198,312,200])]


def normalize(width,height,rgb):
    if (width,height) not in [(320,200),(640,400)] or len(rgb)!=width*height*3:
        raise ValueError('complete logical or exact doubled VGA frame required')
    if width==640:
        reduced=bytearray()
        for y in range(200):
            row=rgb[y*3840:y*3840+1920]
            if row!=rgb[y*3840+1920:(y+1)*3840] or any(row[c::6]!=row[c+3::6] for c in range(3)):
                raise ValueError('frame is not exact nearest 2x')
            for x in range(320):reduced.extend(row[x*6:x*6+3])
        rgb=bytes(reduced)
    return rgb


def quantize(rgb):
    result=bytearray(len(rgb))
    for c,shift in enumerate([3,2,3]):result[c::3]=rgb[c::3].translate(bytes(v>>shift for v in range(256)))
    return bytes(result)


def region(rgb,rect):
    x0,y0,x1,y1=rect
    return b''.join(rgb[(y*320+x0)*3:(y*320+x1)*3] for y in range(y0,y1))


def compare(native,recordings,output):
    import av
    if av.__version__!='14.2.0':raise ValueError('pinned PyAV required')
    manifest=json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest,ROOT/manifest['local_directory'])
    if len({p.resolve() for p in recordings})!=len(recordings):raise ValueError('independent recordings required')
    labels=[121,122,123,199,200,216,220,260]
    expected={sha(game_frame(320,200,(native/f'frame-{n:06}.rgb').read_bytes())):n for n in labels}
    output.mkdir()
    runs=[]
    for number,recording in enumerate(recordings,1):
        verify_assets(manifest,recording/'drive')
        provenance=json.loads((recording/'provenance.json').read_text())
        if identity(recording/'dosbox.conf')!=provenance['config'] or identity(recording/'mapper.map')!=provenance['mapper']:
            raise ValueError('recording configuration changed')
        anchors=[];seen=set();videos=[];diagnostics={};poses=set()
        for path in sorted((recording/'capture').glob('*.avi')):
            with av.open(str(path)) as container:
                for index,frame in enumerate(container.decode(video=0)):
                    if (frame.width,frame.height)!=(640,400):continue
                    raw=rgb_bytes(frame)
                    label=expected.get(sha(game_frame(640,400,raw))) if len(poses)<len(labels) else None
                    if label is not None:poses.add(label)
                    if label is None or label in seen:continue
                    dos=normalize(640,400,raw);candidate=(native/f'frame-{label:06}.rgb').read_bytes()
                    a,b=quantize(dos),quantize(candidate)
                    rows=[]
                    for item in REGIONS:
                        ra,rb=region(a,item['rect']),region(b,item['rect'])
                        differences=sum(ra[i:i+3]!=rb[i:i+3] for i in range(0,len(ra),3))
                        rows.append(item|dict(pixels=len(ra)//3,sha256=sha(ra),differences=differences))
                    score=sum(r['differences'] for r in rows)
                    if score:
                        if label not in diagnostics or score<diagnostics[label]['score']:
                            diagnostics[label]=dict(score=score,regions=rows,video_frame=index)
                        continue
                    whole_a,whole_b=region(a,[0,160,320,200]),region(b,[0,160,320,200])
                    difference=sum(whole_a[i:i+3]!=whole_b[i:i+3] for i in range(0,len(whole_a),3))
                    pair=b''.join(dos[y*960:(y+1)*960]+candidate[y*960:(y+1)*960] for y in range(160,200))
                    ppm=output/f'run-{number}-{label:06}.ppm';ppm.write_bytes(b'P6\n640 40\n255\n'+pair);png_from_ppm(ppm)
                    anchors.append(dict(control_step=label,video=path.name,video_frame=index,regions=rows,full_hud_differences=difference))
                    seen.add(label)
            videos.append(identity(path))
        if seen!=set(labels):
            (output/f'diagnostics-{number}.json').write_text(json.dumps(diagnostics,indent=2)+'\n')
            raise AssertionError(f'missing DOS anchors: {set(labels)-seen}')
        runs.append(dict(recording=str(recording),videos=videos,anchors=anchors))
    report=dict(status='PASS_DECLARED_REGIONS',issue=56,recordings_reused=True,
                excluded='Climber/Digger selection differs; DOS cursor lies in Miner slot. Slots 2,8,9 excluded explicitly. No full HUD/atomic DOS state/input tick certification.',
                normalized='Exact nearest 2x; both sources RGB565 component codes; no shifts/color tolerance',runs=runs)
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native',type=Path,required=True)
    parser.add_argument('--recording',type=Path,nargs='+',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    report=compare(args.native,args.recording,args.output)
    print(json.dumps(dict(status=report['status'],runs=len(report['runs']),anchors=sum(len(r['anchors']) for r in report['runs']))))


if __name__=='__main__':main()
