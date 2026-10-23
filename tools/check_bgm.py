#!/usr/bin/env python3
"""Check note retriggers, bounded OPL delivery and sustained original music."""
import argparse,array,json,math,struct,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
if __package__ in (None,''):sys.path.insert(0,str(ROOT))
from tools.check_audio import build,native,pcm_info,check_browser,guard_original
from tools.check_opl_reference import verify_sources,timeline


def register_trace(rate,frames,events):
    return struct.pack('<III',rate,frames,len(events))+b''.join(struct.pack('<IBB',*event) for event in events)


def note_trace(rate):
    events=[(0,r,v) for r,v in [(0x20,1),(0x23,1),(0x40,0x3f),(0x43,0),(0x60,0xf4),(0x63,0xf4),(0x80,0x44),(0x83,0x44),(0xc0,0),(0xa0,0x98)]]
    for note in range(10):events.extend([(note*rate//2,0xb0,0x11),(note*rate//2,0xb0,0x31)])
    return register_trace(rate,rate*5,events)


def ac_rms(raw,rate,start,seconds):
    samples=array.array('h');samples.frombytes(raw)
    if sys.byteorder!='little':samples.byteswap()
    values=samples[round(start*rate):round((start+seconds)*rate)]
    if not values:raise ValueError('missing sustained PCM window')
    mean=sum(values)/len(values)
    return math.sqrt(max(0,sum(x*x for x in values)/len(values)-mean*mean))


def require_sustained_notes(raw,rate):
    levels=[ac_rms(raw,rate,n*.5,.1) for n in range(10)]
    if min(levels)<1000 or min(levels)<levels[0]*.5:raise ValueError('repeated note attack decayed')
    return levels


def driver_actions(actions,rate):
    """Commands arrive on the first rational driver boundary at/after the sample."""
    result=[];frames=0;previous=0
    for action in actions.split(','):
        command,count=action.split(':');frames+=int(count)
        calls=(frames*2190197+rate*31250-1)//(rate*31250)
        result.append(command+':'+str(calls-previous));previous=calls
    return ','.join(result),frames


def check(directory,wasm=False,original=None):
    if original:guard_original(directory)
    directory.mkdir(parents=True,exist_ok=False);verify_sources();build(wasm,directory)
    subprocess.run(['cargo','build','-p','lemmings-audio-probe','--bin','opl_trace','--release','--locked','--offline'],cwd=ROOT,check=True)
    oracle=directory/'c-reference'
    subprocess.run(['cc','-std=c99','-O2','-Wall','-Wextra','-Werror','-I',str(ROOT/'tools/audio/reference'),str(ROOT/'tools/audio/nuked_reference.c'),str(ROOT/'tools/audio/reference/opl3.c'),'-o',str(oracle)],check=True)
    cases=[];rows=[]
    for rate in [8000,22050,44100,48000,49716,96000,192000]:
        data=note_trace(rate);name=f'retrigger-{rate}';path=directory/(name+'.bin');path.write_bytes(data)
        c=subprocess.check_output([str(oracle)],input=data)
        rust=subprocess.check_output([str(ROOT/'target/release/opl_trace')],input=data)
        if rust!=c:raise ValueError('buffered C register mismatch: '+name)
        levels=require_sustained_notes(rust,rate)
        rows.append(dict(name=name,**pcm_info(rust),note_ac_rms=levels))
        cases.append(dict(kind='registers',name=name,image=path.name,expected=pcm_info(c)))
    for count in [1023,1024,1025,4096]:
        data=register_trace(192000,12000,[(0,n%256,(n//7)%256) for n in range(count)])
        name=f'burst-{count}';path=directory/(name+'.bin');path.write_bytes(data)
        c=subprocess.check_output([str(oracle)],input=data)
        if subprocess.check_output([str(ROOT/'target/release/opl_trace')],input=data)!=c:raise ValueError('buffered C overflow mismatch')
        rows.append(dict(name=name,**pcm_info(c)));cases.append(dict(kind='registers',name=name,image=path.name,expected=pcm_info(c)))
    music=[]
    if original:
        from tools.state_replay import verified_assets
        before=verified_assets(original)
        subprocess.run(['cargo','build','-p','lemmings-resources','--example','trace_adlib','--example','unpack_dat','--release','--locked','--offline'],cwd=ROOT,check=True)
        subprocess.run([str(ROOT/'target/release/examples/unpack_dat'),str(original/'ADLIB.DAT'),str(directory/'decoded')],check=True,capture_output=True)
        image='decoded/part-000.bin';rate=22050
        for number in ([5,21] if wasm else range(22)):
            mixed=number==21;index=5 if mixed else number
            actions=f'm{index}:{rate*60}' if not mixed else f'm5:{rate*20},e0:{rate*5},e17:{rate*5},n:{rate*30}'
            commands,frames=driver_actions(actions,rate)
            trace=subprocess.check_output([str(ROOT/'target/release/examples/trace_adlib'),str(directory/image),commands],text=True)
            wire=timeline([json.loads(line) for line in trace.splitlines()],rate,frames)
            c=subprocess.check_output([str(oracle)],input=wire)
            case=dict(name='music-effects-60s' if mixed else f'music-{index}-60s',image=image,rate=rate,num=2190197,den=31250,actions=actions)
            rust=native(directory,case,'pcm','1,127,512,3,2048,7,8192')
            if rust!=c:raise ValueError('sustained C PCM mismatch: '+case['name'])
            levels=[ac_rms(rust,rate,second,1) for second in range(60)]
            if (mixed or index==5) and min(levels[35:])<1000:raise ValueError('default music faded after retriggers/effects')
            music.append(dict(name=case['name'],**pcm_info(rust),second_ac_rms=levels))
            if index==5:
                (directory/(case['name']+'.pcm')).write_bytes(rust)
                cases.append(dict(**case,expected=pcm_info(c)))
        if verified_assets(original)!=before:raise ValueError('original inputs changed')
    result=dict(status='PASS',register_case_count=len(rows),original_music_case_count=sum(x['name']!='music-effects-60s' for x in music),original_mixed_case_count=sum(x['name']=='music-effects-60s' for x in music),register_cases=rows,music=music,c_buffered_byte_exact=True,physical_listening_verified=False)
    if wasm:result['wasm']=check_browser(directory,cases)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--evidence',type=Path);parser.add_argument('--wasm',action='store_true');parser.add_argument('--original-assets',type=Path);args=parser.parse_args()
    if args.original_assets and not args.evidence:parser.error('original evidence must remain ignored')
    if args.evidence:result=check(args.evidence.resolve(),args.wasm,args.original_assets.resolve() if args.original_assets else None)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-bgm-') as folder:result=check(Path(folder)/'evidence',args.wasm)
    print(json.dumps({k:v for k,v in result.items() if k not in ['register_cases','music']}))
