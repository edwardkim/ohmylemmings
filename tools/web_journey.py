"""Validate complete browser journeys and reconstruct every ordered map boundary."""
import base64
import hashlib

NAMES=('terrain','bricks','objects')


def decode(value):
    try: return base64.b64decode(value,validate=True)
    except (ValueError,TypeError) as error: raise ValueError('invalid base64 bytes') from error


def expanded(attempt):
    initial=attempt['initial'];maps={name:bytearray(decode(initial['maps'][name])) for name in NAMES}
    if not all(maps.values()) or len(maps['terrain'])!=len(maps['bricks']):raise ValueError('invalid initial maps')
    for step,row in enumerate(attempt['rows']):
        if type(row['state']['session']['control_step']) is not int or row['state']['session']['control_step']!=step:raise ValueError('missing/duplicate/unordered boundary')
        if set(row['changes'])!=set(NAMES):raise ValueError('missing map changes')
        for name in NAMES:
            previous=-1
            for change in row['changes'][name]:
                if type(change) is not list or len(change)!=3:raise ValueError('invalid map delta')
                index,before,after=change
                if any(type(n) is not int for n in change) or not previous<index<len(maps[name]) or not 0<=before<=255 or not 0<=after<=255 or before==after or maps[name][index]!=before:
                    raise ValueError('invalid ordered map delta')
                if name!='objects' and after not in (0,1):raise ValueError('invalid terrain material')
                maps[name][index]=after;previous=index
        if step==0 and any(row['changes'][n] for n in NAMES):raise ValueError('initial boundary changed maps')
        yield dict(state=row['state'],**{n:hashlib.sha256(maps[n]).hexdigest() for n in NAMES})
    if any(bytes(maps[n])!=decode(attempt['final']['maps'][n]) for n in NAMES):raise ValueError('final bytes differ from map history')


def validate_journey(data):
    if type(data.get('version')) is not int or data.get('version')!=1 or type(data.get('original')) is not bool or len(data.get('attempts',[]))!=4:raise ValueError('four attempts required')
    attempts=data['attempts'];first=attempts[0]['initial'];counts=[]
    if [a['level'] for a in attempts]!=[1,1,1,2] or [a['reason'] for a in attempts]!=['load' if data['original'] else 'synthetic','restart','continue','continue']:raise ValueError('wrong level transition')
    for index,a in enumerate(attempts):
        initial=a['initial'];s=initial['state']['session'];v=initial['view'];n=initial['state']['nuke']
        rank=a.get('difficulty','fun')
        if rank not in ['fun','tricky','taxing','mayhem'] or v.get('difficulty','fun')!=rank or v['level']!=a['level']:raise ValueError('level/rank identity differs')
        if any(type(s[k]) is not int for k in ['control_step','iteration','released']) or type(s['paused']) is not bool or type(n['is_nuked']) is not bool or s['control_step']!=0 or s['iteration']!=0 or s['paused'] or s['released'] or s['actors'] or n['is_nuked'] or n['assigning'] or n['next_index']:raise ValueError('attempt not fresh')
        if v['right'] or v['dragging'] or v['held']!='None' or v['pointer']!='None' or v['skill']!='Digger' or v['error']:raise ValueError('old view input leaked')
        if index<3 and initial!=first:raise ValueError('restart initial state differs')
        rows=list(expanded(a));counts.append(len(rows))
        if len(rows)<2 or a['final']['state']!=rows[-1]['state']:raise ValueError('final state missing')
        baseline={k:value for k,value in initial['state'].items() if k!='inputs'}
        if {k:value for k,value in rows[0]['state'].items() if k!='inputs'}!=baseline:raise ValueError('initial state lost')
        born=next((r['state']['session']['actors'] for r in rows if r['state']['session']['actors']),[])
        if not born or born[0]['id']!=0:raise ValueError('actor IDs not reset')
        phases=[r['state']['completion']['phase'] for r in rows]
        if any(p!='Playing' for p in phases[:-1]) or phases[-1]!=['Won','Lost','Won','Playing'][index]:raise ValueError('wrong or unfrozen result')
        result=rows[-1]['state']['completion'];nuke=rows[-1]['state']['nuke']
        if index in (0,2) and (result['rescued_percent']!=100 or result['rescued']!=result['original_total'] or nuke['is_nuked']):raise ValueError('partial rescue cannot pass')
        if index==1 and (not nuke['is_nuked'] or nuke['max_lemmings']>=result['original_total'] or 'Nuke' not in a['final']['inputs']):raise ValueError('early Nuke failure missing')
        previous=-1;markers=[]
        for event in a['events']:
            step=event['step']
            if type(step) is not int or not previous<=step<len(rows):raise ValueError('event outside ordered boundaries')
            previous=step
            if event['op']=='Frame':markers.append((step,event['args']))
        if not a['frames'] or markers!=[(f['step'],[i]) for i,f in enumerate(a['frames'])]:raise ValueError('frame markers differ')
        for i,frame in enumerate(a['frames']):
            if frame['index']!=i or not 0<=frame['step']<len(rows):raise ValueError('frame index outside attempt')
            rgb=decode(frame['rgb']);rgba=decode(frame['canvas'])
            if len(rgb)!=192000 or len(rgba)!=256000 or any(alpha!=255 for alpha in rgba[3::4]):raise ValueError('invalid Canvas bytes')
            packed=bytearray(192000);packed[0::3]=rgba[0::4];packed[1::3]=rgba[1::4];packed[2::3]=rgba[2::4]
            if packed!=rgb:raise ValueError('Canvas differs from logical RGB')
    return dict(status='PASS',attempts=4,states=sum(counts),attempt_states=counts,frames=sum(len(a['frames']) for a in attempts))
