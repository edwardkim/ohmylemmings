#!/usr/bin/env python3
"""Separate compositor dispatch, renderer queueing and Web callback costs."""
import argparse
from bisect import bisect_right
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.compatibility_audit import check_pin, require
from tools.web_deadlines import distribution, frame_summary


def correlate_trace(callbacks, events, start):
    """Match trace pipeline IDs, never infer an OS cause from a late callback."""
    frame_summary(callbacks)  # Validate ordering and finite timestamps first.
    markers=[e for e in events if e['name']=='issue113-before-run']
    require(len(markers)==1, 'one trace clock marker required')
    marker=markers[0]
    origin=marker['ts']-marker['args']['data']['startTime']*1000+start*1000
    events=[e for e in events if e['pid']==marker['pid']]
    fires=sorted((e for e in events if e['name']=='FireAnimationFrame' and e.get('ph')=='X'
                  and e['tid']==marker['tid']),key=lambda e:e['ts'])
    fire_times=[e['ts'] for e in fires]
    main=[e for e in events if e['name']=='ProxyMain::BeginMainFrame' and e.get('ph')=='X'
          and e['tid']==marker['tid']]
    sends={e['args']['main_frame_pipeline']['main_frame_id']:e for e in events
           if e['name']=='SendBeginMainFrame' and e.get('ph')=='I'}
    impls={e['args']['args']['sequence_number']:e for e in events
           if e['name']=='Scheduler::BeginImplFrame' and 'args' in e.get('args',{})}
    acks=sorted((e for e in events if e['name']=='ProxyImpl::DidReceiveCompositorFrameAckOnImplThread'),key=lambda e:e['ts'])
    ack_times=[e['ts'] for e in acks]
    gc=[e for e in events if e['name'] in ('MinorGC','MajorGC') and e.get('ph')=='X' and e['tid']==marker['tid']]
    results=[]
    for index,row in enumerate(callbacks):
        if row[2]-row[0]<=1000/60:
            continue
        result=dict(index=index,callback_ms=row[2]-row[1],completion_late_ms=row[2]-row[0]-1000/60,
                    pre_window_timestamp=row[0]<0,matched=False)
        begin=origin+row[1]*1000
        end=origin+row[2]*1000
        pos=bisect_right(fire_times,begin+200)-1
        fire=fires[pos] if pos>=0 else None
        # Reduced-resolution performance.now() can differ by up to one 0.1ms bin.
        if fire is None or abs(fire['ts']-begin)>500:
            results.append(result);continue
        containing=[e for e in main if e['ts']<=fire['ts']<=e['ts']+e['dur']]
        if len(containing)!=1:
            results.append(result);continue
        main_frame=containing[0]
        send=sends.get(main_frame['args']['main_frame_pipeline']['main_frame_id'])
        impl=impls.get(send['args']['main_frame_pipeline']['begin_frame_id']['sequence_number']) if send else None
        if impl is None:
            results.append(result);continue
        frame_time=impl['args']['args']['frame_time_us']
        require(abs(frame_time-(origin+row[0]*1000))<=500,'rAF timestamp and trace frame ID disagree')
        ack_index=bisect_right(ack_times,send['ts'])-1
        ack=acks[ack_index] if ack_index>=0 else None
        if ack is not None and (ack['tid']!=send['tid'] or ack['ts']<impl['ts']):ack=None
        result.update(matched=True,send_delay_ms=(send['ts']-frame_time)/1000,
                      renderer_queue_ms=(main_frame['ts']-send['ts'])/1000,
                      pre_callback_ms=(begin-main_frame['ts'])/1000,
                      ack_to_send_ms=(send['ts']-ack['ts'])/1000 if ack else None,
                      gc_overlap_ms=sum(max(0,min(end,e['ts']+e['dur'])-max(frame_time,e['ts'])) for e in gc)/1000)
        results.append(result)
    return results


def summarize_pairs(before, after):
    require(len(before)==len(after)==3,'three before/after host pairs required')
    reference=before[0]['hashes']
    require(len(reference)==5,'five output hashes required')
    summaries=[]
    for group in [before,after]:
        result=[]
        for run in group:
            require(60<=run['seconds']<62 and run['ticks']==841 and run['hashes']==reference,
                    'workload duration/state/map/RGB identity changed')
            require(run['underruns']==run['visibility_events']==0,'audio/visibility interrupted')
            result.append(frame_summary(run['callback_ms']))
        summaries.append(result)
    return dict(before_misses=[r['completion_misses'] for r in summaries[0]],
                after_misses=[r['completion_misses'] for r in summaries[1]],
                before_callback_over_budget=[r['callback_over_budget'] for r in summaries[0]],
                after_callback_over_budget=[r['callback_over_budget'] for r in summaries[1]],
                web_deadline_ready=all(r['completion_misses']==r['callback_over_budget']==0 for r in summaries[1]),
                before_presents=[r['presents'] for r in before],after_presents=[r['presents'] for r in after],
                before_callback_ms=[r['callback_ms'] for r in summaries[0]],
                after_callback_ms=[r['callback_ms'] for r in summaries[1]])


def audit(path, local=False):
    data=json.loads(path.read_text())
    require(data['schema_version']==1 and data['status']=='MEASURED-WITH-DEFERRED-QUALIFICATION','dispatch evidence schema')
    for pin in data['source']:check_pin(ROOT,pin)
    vectors=[json.loads(line) for line in check_pin(ROOT,data['observations']).decode().splitlines()]
    require([r['id'] for r in vectors]==['before-1','after-1','before-2','after-2','before-3','after-3'],'paired order')
    reference=json.loads(check_pin(ROOT,data['reference']))['web'][0]['output_sha256']
    require(all(r['hashes']==reference for r in vectors),'paired result differs from the fixed reference')
    for vector,run in zip(vectors,data['runs'],strict=True):
        require(vector['id']==run['id'],'run identity')
        if local:
            raw=json.loads(check_pin(ROOT,run['raw']))
            require(vector['callback_ms']==[r[:3] for r in raw['callbackDetails']] and vector['hashes']==raw['outputHashes']
                    and vector['presents']==len(raw['frames']) and vector['ticks']==len(raw['ticks'])
                    and vector['underruns']==raw['audio']['underruns']-raw['initialAudioUnderruns']
                    and vector['visibility_events']==len(raw['visibility']) and vector['seconds']==raw['seconds'],'raw observations differ')
    result=summarize_pairs(vectors[::2],vectors[1::2])
    require(result==data['comparison'],'recorded comparison differs')
    trace=json.loads(check_pin(ROOT,data['trace_observations']))
    matched=[row for row in trace['rows'] if row['matched']]
    acks=[row for row in matched if row['ack_to_send_ms'] is not None]
    require(trace['misses']==len(trace['rows']) and trace['matched']==len(matched)
            and trace['ack_before_send']==len(acks)
            and trace['gc_overlap']==sum(row['gc_overlap_ms']>0 for row in matched)
            and trace['pre_window']==sum(row['pre_window_timestamp'] for row in trace['rows']),'trace totals differ')
    for field,rows in [('send_delay_ms',matched),('renderer_queue_ms',matched),('ack_to_send_ms',acks)]:
        require(trace[field]==distribution([row[field] for row in rows]),'trace stage distribution differs')
    require(data['g11_ready'] is False and data['g12_entry_ready'] is False,'overall review remains separate')
    if local:
        for pin in data['local_evidence']:check_pin(ROOT,pin)
        for pin in data['recorded_producer']:check_pin(ROOT,pin)
    return dict(evidence_valid=True,**result,g11_ready=False,g12_entry_ready=False)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=ROOT/'assets/manifests/web_dispatch.json')
    parser.add_argument('--local',action='store_true')
    args=parser.parse_args()
    print(json.dumps(audit(args.manifest,args.local)))


if __name__=='__main__':main()
