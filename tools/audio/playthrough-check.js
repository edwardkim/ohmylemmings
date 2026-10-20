import init,{WebGame,WebAudio,WebAssets} from './pkg/lemmings_web.js';
const assert=(ok,label)=>{if(!ok)throw new Error(label);};
const same=(a,b,label)=>assert(JSON.stringify(a)===JSON.stringify(b),label);
const digest=async bytes=>(await(await fetch('/__sha',{method:'POST',body:bytes})).json()).sha256;
try {
    await init();const spec=await(await fetch('./expected.json')).json();
    const rows=await(await fetch('./native/trace.json')).json();const bank=new WebAssets();
    for(const name of spec.original_files)bank.insert(name,new Uint8Array(await(await fetch('./original/'+name)).arrayBuffer()));
    const a=new WebGame('win','dos'),b=new WebGame('win','dos');
    if(spec.original_files.length){a.load_original(bank,1);b.load_original(bank,1);}
    const audio=new WebAudio(44100);audio.set_volume(100);audio.prepare(a,true);b.drain_sound_events();
    const bytes=new Uint8Array(spec.pcm.frames*2),view=new DataView(bytes.buffer);let at=0;
    for(const row of rows) {
        const wall=row.wall;
        for(const game of [a,b]) {
            if([5,120].includes(wall))game.command('Pause',0);
            if([8,125].includes(wall))game.command('Resume',0);
            if(wall===90)game.key('KeyD',false);
            if(wall===106)game.command('AssignDigger',1);
            if(wall===160)game.command('Nuke',0);
            if(wall===360)game.restart_level();
            game.step(1);
        }
        if(wall===100||wall===105)audio.set_volume(wall===100?0:100);
        same(JSON.parse(a.state()),row.state,'native/WASM state '+wall);
        assert(a.state()===b.state(),'audio changed game '+wall);
        const active=!(wall>=75&&wall<78)&&JSON.parse(a.state()).completion.phase==='Playing';
        assert(active===row.active&&audio.prepare(a,active)===row.flush,'flush/activation '+wall);
        const count=Math.floor(wall*44100*156250/2190197)-Math.floor((wall-1)*44100*156250/2190197);
        let remaining=count,index=0;
        while(remaining){const n=Math.min(remaining,[1,127,512,3,2048,7][index++%6]);for(const sample of audio.render(n)){view.setInt16(at,sample,true);at+=2;}remaining-=n;}
        same(JSON.parse(b.drain_sound_events()),row.sounds,'sound event order '+wall);
        same(JSON.parse(audio.state()),row.audio,'audio clock '+wall);
        for(const [extension,method] of [['rgb','frame'],['terrain','terrain'],['objects','objects']]) {
            const expected=spec.images[wall+'.'+extension];
            if(expected){const data=a[method](),peer=b[method]();assert(await digest(data)===expected,'native/WASM '+extension+' '+wall);assert(data.every((v,i)=>v===peer[i]),'audio changed '+extension);}
        }
    }
    assert(at===bytes.length,'PCM frame count');
    const pcm=await(await fetch('/__pcm',{method:'POST',body:bytes})).json();assert(pcm.equal,'native/WASM PCM bytes');delete pcm.equal;
    same(pcm,spec.pcm,'PCM metrics');audio.free();a.free();b.free();bank.free();
    const result={status:'PASS',steps:rows.length,pcm,partitioned:true,snapshots:Object.keys(spec.images).length};
    document.querySelector('#result').textContent=JSON.stringify(result);await fetch('/__result',{method:'POST',body:JSON.stringify(result)});
}catch(error){const result={status:'FAIL',error:String(error),stack:error.stack};document.querySelector('#result').textContent=JSON.stringify(result);await fetch('/__result',{method:'POST',body:JSON.stringify(result)});}
