import init,{WebGame,WebAudio,WebAssets} from './pkg/lemmings_web.js';
import {AudioHost} from './audio.js';
const assert=(ok,label)=>{if(!ok)throw new Error(label);};
const throws=(fn,label)=>{let failed=false;try{fn();}catch{failed=true;}assert(failed,label);};
const sha=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(n=>n.toString(16).padStart(2,'0')).join('');
// SHA-256 below also works on insecure LAN HTTP through a tiny byte digest
// supplied by the host only for test comparison (production never hashes audio).
async function digest(bytes) {
    if(globalThis.crypto?.subtle)return sha(bytes);
    const result=await fetch('/__sha',{method:'POST',body:bytes});if(!result.ok)throw new Error('digest failed');return result.text();
}
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
async function mockChecks() {
    let created=0,active=true;const states=[];
    class Pipe {
        constructor(rate){this.rate=rate;this.resume=true;this.rendered=0;}
        set_volume(n){this.volume=n;}suspend(){this.resume=true;}
        prepare(_game,audible){const flush=this.resume;this.resume=false;this.audible=audible;return flush;}
        render(n){this.rendered+=n;return new Int16Array(n).fill(1234);}free(){this.freed=true;}
    }
    class Context {
        constructor(){created++;this.state='suspended';this.sampleRate=created===2?44100:48000;this.currentTime=0;this.destination={};this.stopped=0;}
        resume(){this.state='running';return Promise.resolve();}suspend(){this.state='suspended';return Promise.resolve();}close(){this.state='closed';return Promise.resolve();}
        createBuffer(_c,n){return {getChannelData:()=>new Float32Array(n)};}
        createBufferSource(){const c=this;return {connect(){},disconnect(){},stop(){c.stopped++;},start(at){this.at=at;}};}
    }
    const host=new AudioHost(()=>({}),Pipe,s=>states.push(s.state),()=>active,()=>new Context());
    try {
        assert(created===0,'constructor started autoplay');await host.activate();assert(!host.muted&&host.sources.size>0,'activation did not schedule');
        for(let i=0;i<100;i++)host.service();assert(host.sources.size<=8&&host.nextTime<=0.09,'unbounded scheduling');
        const oldVolume=host.volume;throws(()=>host.setVolume(NaN),'NaN volume accepted');assert(host.volume===oldVolume,'bad volume mutated');
        host.setVolume(33);assert(host.sources.size===0&&host.pipe.volume===33,'old gain remained queued');host.service();
        host.context.currentTime=20;host.service();assert(host.underruns===1&&host.sources.size<=8&&host.nextTime<20.1,'clock jump queued debt');
        active=false;host.service();assert(host.sources.size===0,'inactive game retained sources');active=true;
        host.hold(true);assert(host.sources.size===0&&host.context.state==='suspended','hidden kept playing');host.hold(false);await sleep(0);
        assert(!host.muted&&host.context.state==='running','visible did not resume');
        host.mute();assert(host.sources.size===0&&host.timer===null,'mute retained queue/timer');
        host.context.state='closed';await host.activate();assert(host.pipe.rate===44100&&!host.muted,'closed context not recreated');
    }finally{await host.dispose();}
    const failed=new AudioHost(()=>({}),Pipe,()=>{},()=>true,()=>({state:'suspended',resume:()=>Promise.reject(new Error('blocked')),suspend:()=>Promise.resolve(),close:()=>Promise.resolve()}));
    await failed.activate();assert(failed.muted&&failed.error&&failed.sources.size===0,'blocked activation was not recoverable');await failed.dispose();
    let resolve;
    const race=new AudioHost(()=>({}),Pipe,()=>{},()=>true,()=>({state:'suspended',sampleRate:48000,resume(){return new Promise(r=>{resolve=()=>{this.state='running';r();};});},suspend:()=>Promise.resolve(),close:()=>Promise.resolve()}));
    const waiting=race.activate();race.mute();resolve();await waiting;assert(race.muted&&race.sources.size===0&&race.timer===null,'stale activation restarted audio');await race.dispose();
    return {bounded:true,blocked:true,activation_race:true,mute_hide_resume:true,volume:true,clock_jump:true,context_recreation:true};
}
async function localMp3Checks() {
    const {LocalMusic}=await import('./local-music.js');
    const context=new AudioContext();await context.resume();
    const player=new LocalMusic(()=>context);
    const bytes=await(await fetch('./local-tone.mp3')).arrayBuffer();
    const until=async check=>{for(let n=0;n<200;n++){if(check())return;await sleep(25);}throw Error('local MP3 timeout: '+player.state);};
    try {
        player.choose(new File([bytes],'local-tone.mp3',{type:'audio/mpeg'}));
        await until(()=>player.media.readyState>=2);player.sync(true,0,true);
        await until(()=>player.state==='playing');
        const splitter=context.createChannelSplitter(2),analysers=[context.createAnalyser(),context.createAnalyser()];
        player.source.connect(splitter);analysers.forEach((a,i)=>{a.fftSize=2048;splitter.connect(a,i);});
        await sleep(650);
        const waves=analysers.map(a=>{const values=new Float32Array(2048);a.getFloatTimeDomainData(values);return values;});
        assert(waves.every(a=>a.some(v=>v!==0))&&waves[0].some((v,i)=>Math.abs(v-waves[1][i])>0.001),'local stereo graph silent');
        assert(player.media.loop,'local MP3 does not repeat');
        player.setVolume(23);player.setMasterVolume(40);assert(player.gain.gain.value===Math.fround(.092),'independent local gain');
        player.sync(false,0,true);assert(player.media.paused&&player.media.currentTime===0,'local hide did not stop');
        player.sync(true,1,true);await until(()=>player.state==='playing');
        player.choose(new File(['broken audio'],'broken.mp3',{type:'audio/mpeg'}));await until(()=>player.state==='error');
        player.dispose();assert(!player.media&&!player.url,'local file URL retained');
        return {stereo:true,loop:true,independent_gain:true,hide_resume:true,corrupt_file:true,cleanup:true};
    } finally {player.dispose();await context.close();}
}
try {
    await init();
    const delivery=await(await fetch('./audio-source.json')).json();
    for(const [name,key] of [['audio-source.tar.gz','archive_sha256'],['LGPL-2.1-or-later.txt','license_sha256'],['pkg/lemmings_web_bg.wasm','wasm_sha256']])assert(await digest(new Uint8Array(await(await fetch('./'+name)).arrayBuffer()))===delivery[key],'source/license/WASM delivery '+name);
    assert(delivery.license==='LGPL-2.1-or-later'&&(await(await fetch('./audio-license.html')).text()).includes('audio-source.tar.gz'),'license notice unavailable');
    const expected=await(await fetch('./expected.json')).json();const bank=new WebAssets();
    for(const name of expected.original_files||[])bank.insert(name,new Uint8Array(await(await fetch('./original/'+name)).arrayBuffer()));
    const results=[];
    for(const test of expected.cases) {
        const a=new WebGame('builder','reference'),b=new WebGame('builder','reference');
        if(expected.original_files?.length){a.load_original(bank,1);b.load_original(bank,1);}
        const output=new WebAudio(test.rate);output.set_volume(test.volume);let flushes=Number(output.prepare(a,true));const all=[];
        for(let step=1;step<=80;step++) {
            for(const game of [a,b]) {if(step===5)game.command('Pause',0);if(step===8)game.command('Resume',0);if(step===16)game.key('KeyD',false);if(step===17)game.key('KeyD',true);if(step===78)game.restart_level();game.step(1);}
            flushes+=Number(output.prepare(a,!(step>=65&&step<68)));
            all.push(output.render([1,127,512,3,1024,7,2048][(step-1)%7]));
            assert(a.state()===b.state(),test.name+' audio changed gameplay');
        }
        const frames=all.reduce((n,v)=>n+v.length,0),bytes=new Uint8Array(frames*2),view=new DataView(bytes.buffer);let at=0;
        for(const chunk of all)for(const sample of chunk){view.setInt16(at,sample,true);at+=2;}
        assert(frames===test.frames&&await digest(bytes)===test.pcm_sha256,test.name+' native/WASM PCM');
        assert(JSON.stringify(JSON.parse(a.state()))===JSON.stringify(test.state),test.name+' native/WASM state');
        assert(JSON.stringify(JSON.parse(output.state()))===JSON.stringify(test.audio)&&flushes===test.flushes,test.name+' audio state/flush');
        for(const [method,key] of [['frame','rgb_sha256'],['terrain','terrain_sha256'],['objects','objects_sha256']])assert(await digest(a[method]())===test[key],test.name+' '+method);
        assert(await digest(a.frame())===await digest(b.frame()),test.name+' audio changed RGB');
        if(test.rate===44100&&test.volume===100) {
            const samples=new Float32Array(frames);for(let i=0;i<frames;i++)samples[i]=view.getInt16(i*2,true)/32768;
            const context=new OfflineAudioContext(1,frames,44100),buffer=context.createBuffer(1,frames,44100);buffer.copyToChannel(samples,0);
            const source=context.createBufferSource();source.buffer=buffer;source.connect(context.destination);source.start();
            const rendered=(await context.startRendering()).getChannelData(0);
            assert(await digest(new Uint8Array(rendered.buffer))===await digest(new Uint8Array(samples.buffer)),'actual OfflineAudioContext changed PCM');
        }
        results.push({name:test.name,frames,sha256:test.pcm_sha256});output.free();a.free();b.free();
    }
    const mocks=await mockChecks();const game=new WebGame('builder','reference');if(expected.original_files?.length)game.load_original(bank,1);game.step(55);const before=game.state(),pixels=game.frame();
    const live=new AudioHost(()=>game,WebAudio);
    let liveResult;
    try {
        await live.activate();assert(!live.muted&&live.context.state==='running'&&live.sources.size>0,'real AudioContext output unavailable');
        const analyser=live.context.createAnalyser();analyser.fftSize=2048;const connected=new WeakSet();const values=new Float32Array(2048);
        for(let attempt=0;attempt<100;attempt++){for(const s of live.sources)if(!connected.has(s)){s.connect(analyser);connected.add(s);}await sleep(20);analyser.getFloatTimeDomainData(values);if(values.some(v=>v!==0))break;}
        assert(values.some(v=>v!==0),'real AudioContext graph stayed silent');
        assert(game.state()===before&&game.frame().every((v,i)=>v===pixels[i]),'device changed game');
        liveResult={state:live.context.state,nonzero:values.filter(v=>v!==0).length,queued_nodes:live.sources.size,queue_seconds:Math.max(0,live.nextTime-live.context.currentTime)};
        live.mute();assert(live.sources.size===0,'real mute retained nodes');
    }finally{await live.dispose();game.free();bank.free();}
    const local_mp3=await localMp3Checks();
    const result={status:'PASS',local_mp3,cases:results,offline_context:true,source_delivery:{license:true,archive:true,wasm_pair:true},mocks,live:liveResult,secure_context:isSecureContext,worklet_required:false};
    document.querySelector('#result').textContent=JSON.stringify(result);await fetch('/__result',{method:'POST',body:JSON.stringify(result)});
}catch(error){const result={status:'FAIL',error:String(error),stack:error.stack};document.querySelector('#result').textContent=JSON.stringify(result);await fetch('/__result',{method:'POST',body:JSON.stringify(result)});}
