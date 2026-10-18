"""Independent synthetic coordinate-set oracle; not an executed DOS/Lemmix trace."""
import argparse
import json
import re
from pathlib import Path
import struct

OFFSET=0xcbf29ce484222325
MASK=(1<<64)-1

def fnv(data, value=OFFSET):
    for byte in data:
        value=((value^byte)*0x100000001b3)&MASK
    return value

# State: x,y,dx,action,frame,end,fallen,removed. Actions W/J/F/S=0/1/2/3.
def motion(state, solid, starts_three):
    x,y,dx,action,frame,end,fallen,removed=state
    if removed:
        return state,(0,0,0)
    last=(7,0,3,15)[action]
    end=frame==last
    frame=frame+1 if frame<last else (0 if action in (0,2) else frame)
    def hit(xx,yy,minimum):
        return (xx,max(yy,minimum)) in solid
    def transition(new):
        nonlocal action,frame,end,fallen,dx
        if action==new:return
        action=new;frame=0;end=False;fallen=3 if new==2 and starts_three else 0
        if new==3:dx=0
    def top():
        nonlocal x,y,dx
        if y<5:
            y=3;dx=-dx
            if action==1:transition(0)
    checking=1;splat=0
    if action==0:
        x+=dx
        if not 0<=x<=1647:
            dx=-dx
        elif hit(x,y,0):
            rise=next((d for d in range(7) if not hit(x,y-d-1,-d-1)),7)
            if rise==7:dx=-dx
            else:
                if rise>=3:transition(1);y-=2
                else:y-=rise
                top()
        else:
            gap=next((d for d in range(1,4) if hit(x,y+d,d)),4)
            y+=gap
            if gap==4:transition(2)
            if y>163:removed=1;checking=0
    elif action==1:
        rise=next((d for d in range(2) if not hit(x,y-d-1,-d-1)),2)
        y-=rise
        if rise<2:transition(0)
        top()
    elif action==2:
        distance=next((d for d in range(3) if hit(x,y+d,d)),3)
        death=next((d for d in range(1,distance+1) if y+d>163),None)
        if death is not None:y+=death;removed=1;checking=0
        else:
            y+=distance
            if distance==3:fallen+=3
            elif fallen>60:transition(3);splat=1
            else:transition(0)
    else:
        checking=0
        if end:removed=2
    return (x,y,dx,action,frame,int(end),fallen,removed),(checking,removed,splat)

def state_bytes(state):
    return struct.pack('<iibBBBI B',*state)

def actor_bytes(actor_id,born,state):
    return struct.pack('<IQ',actor_id,born)+state_bytes(state)

def corpus():
    groups=[];total=0;aggregate=OFFSET
    variants=[]
    for action in range(4):
        for dx in ((0,) if action==3 else (-1,1)):
            for frame in ((0,7) if action==0 else (0,) if action==1 else (0,3) if action==2 else (0,14,15)):
                for fallen in ((0,60,61) if action==2 else (0,)):
                    variants.append((action,dx,frame,fallen))
    for pattern in range(512):
        solid={(i%3,i//3) for i in range(9) if pattern&(1<<i)}
        for rule in (0,1):
            stream=bytearray()
            for x in range(3):
                for y in range(-1,4):
                    for action,dx,frame,fallen in variants:
                        state=(x,y,dx,action,frame,0,fallen,0)
                        after,outcome=motion(state,solid,rule)
                        stream.extend(struct.pack('<HBiiBbBI',pattern,rule,x,y,action,dx,frame,fallen))
                        stream.extend(state_bytes(after));stream.extend(outcome)
                        total+=1
            groups.append(fnv(stream))
            aggregate=fnv(stream,aggregate)
    assert total==322560 and len(groups)==1024
    return groups,total,aggregate

def scene(kind):
    floor=100 if kind==1 else 4 if kind==5 else 60
    solid=set()
    for y in range(160):
        for x in range(96):
            height=56 if kind==3 and 48<=x<64 else floor
            if kind!=2 and y>=height:solid.add((x,y))
            if kind==4 and 40<=x<48 and 40<=y<60:solid.add((x,y))
    return solid

def integration():
    aggregate=OFFSET;cases=0;updates=0
    for kind in range(6):
        solid=scene(kind)
        for scenario in range(3):
            for glitch in (0,1):
                stream=bytearray((8,kind,scenario,glitch))
                actors=[];old_iter=old_released=0
                for control in range(1,341):
                    step=control-1
                    paused=scenario==2 and 16<=step<34
                    iteration=min(control,16) if scenario==2 and control<=34 else control-(18 if scenario==2 else 0)
                    if scenario==1 and step==54:stream.extend(struct.pack('<BQIB',0,step,0,1))
                    if scenario==2 and step in (16,34):stream.extend(struct.pack('<BQIB',0,step,0,1))
                    rr=99 if scenario==1 and step>=54 else 50
                    interval=4 if scenario==1 else 28
                    if iteration<35:countdown=20
                    elif iteration<54:countdown=54-iteration
                    elif iteration<82:countdown=82-iteration
                    else:countdown=82+((iteration-82)//interval+1)*interval-iteration
                    opportunities=0 if iteration<54 else 1+(0 if iteration<82 else 1+(iteration-82)//interval)
                    released=min(4,opportunities)
                    if iteration!=old_iter:
                        if iteration in (15,34,35):stream.append({15:1,34:2,35:3}[iteration])
                        if released>old_released:
                            entrance=(0,1,1,0)[old_released%4]
                            stream.extend(struct.pack('<BIBQ',4,old_released,entrance,iteration))
                            actors.append([old_released,iteration,(24+40*entrance,0 if kind==5 else 14,1,2,0,0,3,0)])
                        stream.extend(struct.pack('<BQ',5,iteration))
                        for actor in actors:
                            if actor[2][-1]:continue
                            actor[2],outcome=motion(actor[2],solid,1)
                            stream.append(9);stream.extend(actor_bytes(*actor));stream.extend(outcome)
                            updates+=1
                    clock_ticks=iteration-(min(max(iteration-16,0),18) if scenario==2 and glitch else 0)
                    remaining=max(34-iteration,0) if scenario==2 and glitch and step>=16 else 0
                    stream.extend(struct.pack('<BQQBIBBBBBI',6,control,iteration,paused,300-clock_ticks//17,
                        clock_ticks%17,remaining,rr,countdown,iteration>=35,released))
                    stream.append(10);stream.extend(struct.pack('<I',len(actors)))
                    for actor in actors:stream.extend(actor_bytes(*actor))
                    old_iter,old_released=iteration,released
                aggregate=fnv(stream,aggregate);cases+=1
    assert cases==36
    return aggregate,updates

def generated(groups,corpus_digest,integration_digest):
    lines=['// Independent Python oracle: tests/fixtures/motion_reference.py.',
           '// Synthetic expectations computed before the Rust motion implementation.',
           f'pub const CORPUS_DIGEST: u64 = 0x{corpus_digest:016x};',
           f'pub const INTEGRATION_DIGEST: u64 = 0x{integration_digest:016x};',
           'pub const GROUP_DIGESTS: [u64; 1024] = [']
    for i in range(0,len(groups),4):lines.append('    '+', '.join(f'0x{v:016x}' for v in groups[i:i+4])+',')
    return '\n'.join(lines+ ['];',''])

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--emit-golden',action='store_true',help='print expectations; never reads Rust output')
    args=parser.parse_args()
    groups,total,digest=corpus();integrated,updates=integration()
    expected=generated(groups,digest,integrated)
    if args.emit_golden:print(expected,end='')
    else:
        golden=Path(__file__).resolve().parents[2]/'src/simulation/motion/golden.rs'
        actual=golden.read_text()
        assert int(re.search(r'CORPUS_DIGEST: u64 = 0x([0-9a-f]+)',actual)[1],16)==digest
        assert int(re.search(r'INTEGRATION_DIGEST: u64 = 0x([0-9a-f]+)',actual)[1],16)==integrated
        observed=[int(v,16) for v in re.findall(r'0x([0-9a-f]+)',actual.split('GROUP_DIGESTS:')[1])]
        assert observed==groups,'golden differs from independently recomputed expectations'
        print(json.dumps({'transitions':total,'groups':len(groups),'corpus_fnv1a64':f'{digest:016x}',
            'integration_cases':36,'integration_replays':108,'integration_control_calls':36720,
            'actor_updates_per_pattern':updates,'integration_fnv1a64':f'{integrated:016x}'}))
