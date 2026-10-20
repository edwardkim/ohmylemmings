import init,{game_sound_trace,audio_pcm} from './pkg/audio_probe.js';
const sha=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(x=>x.toString(16).padStart(2,'0')).join('');
function actions(test){
    const start=test.start_control_step,end=Math.floor((test.steps+2)*44100*156250/2190197);
    const events=test.sounds.events.filter(e=>e.command!==null).map(e=>({frame:Math.floor((e.control_step-start)*44100*156250/2190197),label:e.command[0]==='stop'?'s':(e.command[0]==='music'?'m':'e')+e.command[1]}));
    if(!events.length)return `n:${end}`;
    const result=events[0].frame?[`n:${events[0].frame}`]:[];
    for(let i=0;i<events.length;i++)result.push(`${events[i].label}:${(events[i+1]?.frame??end)-events[i].frame}`);
    return result.join(',');
}
try{
    await init();const expected=await (await fetch('./expected.json')).json();let trace;
    for(const parts of [[1],[7,3,1000]]){
        const raw=game_sound_trace(new Uint32Array(parts));
        if(await sha(new TextEncoder().encode(raw))!==expected.trace_sha256)throw new Error('game sound trace/state/map/RGB mismatch');
        trace=JSON.parse(raw);
    }
    const data=new Uint8Array(await (await fetch('./synthetic.bin')).arrayBuffer());const pcm=[];
    for(const want of expected.pcm){
        const test=trace.cases.find(c=>c.name===want.name);let info;
        for(const parts of [[8192],[1,31,1024,7,8192]]){
            const samples=audio_pcm(data,44100,2190197,31250,actions(test),new Uint32Array(parts));
            const bytes=new Uint8Array(samples.length*2),view=new DataView(bytes.buffer);
            for(let i=0;i<samples.length;i++)view.setInt16(i*2,samples[i],true);
            info={name:want.name,frames:samples.length,sha256:await sha(bytes)};
            if(JSON.stringify(info)!==JSON.stringify(want))throw new Error('game commands PCM mismatch: '+want.name);
        }
        pcm.push(info);
    }
    await fetch('/__result',{method:'POST',body:JSON.stringify({status:'PASS',trace_sha256:expected.trace_sha256,pcm})});
}catch(error){await fetch('/__result',{method:'POST',body:JSON.stringify({status:'FAIL',error:String(error)})});}
