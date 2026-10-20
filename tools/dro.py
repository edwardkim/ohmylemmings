#!/usr/bin/env python3
"""DOSBox DRO v2 OPL2 capture and relative musical timing comparison, not PCM equality."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
try:
    from .sound_playthrough import guard_original
except ImportError:
    from sound_playthrough import guard_original


def parse(data):
    if len(data)<26:raise ValueError('truncated DRO header')
    magic,major,minor,count,duration,hardware,fmt,compression,short,long,size=struct.unpack_from('<8sHHIIBBBBBB',data)
    if (magic,major,minor,hardware,fmt,compression)!=(b'DBRAWOPL',2,0,0,0,0):
        raise ValueError('expected uncompressed interleaved DRO v2 OPL2')
    if not 1<=size<=127 or short==long or short<size or long<size or count>1_000_000 or len(data)!=26+size+count*2:
        raise ValueError('invalid DRO table/count/length')
    mapping=data[26:26+size];elapsed=0;writes=[]
    for offset in range(26+size,len(data),2):
        code,value=data[offset:offset+2]
        if code==short:elapsed+=value+1
        elif code==long:elapsed+=(value+1)*256
        else:
            index=code&127
            if index>=size:raise ValueError('DRO register index exceeds table')
            writes.append([elapsed,mapping[index]+(256 if code&128 else 0),value])
        if elapsed>duration:raise ValueError('DRO delay exceeds declared duration')
    if elapsed!=duration:raise ValueError('DRO duration mismatch')
    return dict(milliseconds=duration,writes=writes,sha256=hashlib.sha256(data).hexdigest())


def keyed_frequencies(writes, channel):
    cache={};events=[]
    for time,register,value in writes:
        previous=cache.get(register,0);cache[register]=value
        # DOSBox drops repeated register values. Apply that rule to both peers.
        if previous!=value and register==0xb0+channel and value&32:
            events.append((time,((value&31)<<8)|cache.get(0xa0+channel,0)))
    return events


def compare_notes(actual, reference, *, channels=(0,1,2,3), tolerance_ms=1000*31250/2190197+1):
    result=[]
    for channel in channels:
        a=keyed_frequencies(actual,channel);b=keyed_frequencies(reference,channel)
        if not a or len(a)>len(b) or [note for _,note in a]!=[note for _,note in b[:len(a)]]:
            raise ValueError(f'OPL keyed frequency sequence differs on channel {channel}')
        offset=a[0][0]-b[0][0]
        error=max(abs((at-bt)-offset) for (at,_),(bt,_) in zip(a,b))
        if error>tolerance_ms:raise ValueError(f'OPL relative timing differs on channel {channel}')
        result.append(dict(channel=channel,notes=len(a),offset_ms=offset,max_relative_error_ms=error))
    return dict(status='PASS',matched_notes=sum(row['notes'] for row in result),tolerance_ms=tolerance_ms,channels=result,
        scope='Exact keyed frequency sequence and relative timing prefix; per-channel start alignment; excludes PCM, physical OPL and subjective listening')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--capture',type=Path,required=True);p.add_argument('--native',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);args=p.parse_args()
    guard_original(args.evidence);args.evidence.mkdir(parents=True,exist_ok=False)
    capture=parse(args.capture.read_bytes());native=[]
    for line in args.native.read_text().splitlines():
        row=json.loads(line)
        native.extend([row['step']*1000*31250/2190197,reg,value] for reg,value in row['writes'])
    result=compare_notes(capture['writes'],native)
    if capture['milliseconds']<20000 or result['matched_notes']<200:raise ValueError('insufficient original game music coverage')
    result.update(dro_sha256=capture['sha256'],dro_milliseconds=capture['milliseconds'],dro_writes=len(capture['writes']),native_sha256=hashlib.sha256(args.native.read_bytes()).hexdigest())
    (args.evidence/'summary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))


if __name__=='__main__':main()
