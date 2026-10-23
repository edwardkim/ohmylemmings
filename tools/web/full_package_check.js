import {checkBundled} from './__bundled_check.js';
import {loadBundled} from './bundled.js';
const assert=(ok,message)=>{if(!ok)throw Error(message);};
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
let result;
try {
    const loader=await checkBundled(loadBundled,assert);
    const html=await (await fetch('./index.html')).text();
    const clock='<script>window.requestAnimationFrame=()=>1;window.cancelAnimationFrame=()=>{};</script>';
    async function open(broken=null) {
        const frame=document.createElement('iframe');
        const fault=broken?`<script>const originalFetch=window.fetch;window.fetch=(url,options)=>String(url).endsWith(${JSON.stringify(broken)})?Promise.resolve(new Response('',{status:404})):originalFetch(url,options);</script>`:'';
        frame.srcdoc=html.replace('<script type="module" src="./app.js">',clock+fault+'<script type="module" src="./app.js">');
        const loaded=new Promise(resolve=>frame.onload=resolve);
        document.body.append(frame);await loaded;
        const win=frame.contentWindow,doc=win.document;
        const app=await win.eval('import("./app.js")');
        let game;const detach=app.observe((_,g)=>{game=g;});
        if(broken)assert(doc.querySelector('#error').textContent&&doc.querySelector('#restart').disabled,'missing packaged file did not stop startup');
        else assert(!doc.querySelector('#error').textContent&&game?.original(),'packaged default did not load original: '+doc.querySelector('#error').textContent);
        return {frame,win,doc,game,detach};
    }
    const first=await open();
    let {win,doc,game}=first;
    assert(game.difficulty()==='fun'&&game.level()===1&&!doc.querySelector('#open-level').disabled,'default is not original Fun01');
    const hdDefault=doc.querySelector('meta[name="game-presentation"][content="hd"]')!==null;
    let hdInfo=null,hd_png=null,hdPlay=null;const hdLevels=[];
    async function settled(){for(let n=0;n<3000&&doc.querySelector('#presentation').disabled;n++)await pause(10);assert(!doc.querySelector('#presentation').disabled&&!doc.querySelector('#error').textContent,'HD loading failed: '+doc.querySelector('#error').textContent);}
    if(hdDefault){
        assert(doc.querySelector('#presentation').value==='hd'&&doc.querySelector('#game').width===1920,'package did not start in HD');
        hdInfo=JSON.parse(game.hd_info());assert(hdInfo.ui_glyphs===95&&hdInfo.actor_frames_available===337,'HD character/font delivery incomplete');
        hd_png=doc.querySelector('#game').toDataURL('image/png').split(',')[1];
        const api=await win.eval('import("./pkg/lemmings_web.js")');
        const bank=await loadBundled(api.WebAssets),plain=new api.WebGame('builder','reference');
        try{
            plain.load_original_rank(bank,'fun',1);let frames=0;
            for(let step=0;step<664;step++){
                if(step===60){game.command('AssignDigger',0);plain.command('AssignDigger',0);}
                game.step(1);plain.step(1);assert(game.state()===plain.state(),'packaged HD changed state at '+step);
                if(step%32===0||step===139||step===663){
                    doc.dispatchEvent(new Event('visibilitychange'));frames++;
                    assert(!JSON.parse(game.hd_info()).missing_actor_frames.length,'packaged HD actor fallback');
                }
            }
            const state=JSON.parse(game.state());assert(state.completion.phase==='Won'&&state.exit.saved===10,'packaged HD Fun01 solution failed');
            hdPlay={states:665,frames,saved:state.exit.saved,phase:state.completion.phase};
        }finally{plain.free();bank.free();}
        for(const number of [1,2,4,5,6,22]){
            doc.querySelector('#level').value=number;doc.querySelector('#open-level').click();await settled();
            assert(doc.querySelector('#presentation').value==='hd','HD theme silently fell back');
            hdLevels.push({number,scope:game.hd_scope()});
        }
        doc.querySelector('#level').value=1;doc.querySelector('#open-level').click();await settled();
        doc.querySelector('#presentation').value='original';doc.querySelector('#presentation').dispatchEvent(new Event('change'));await settled();
        assert(doc.querySelector('#game').width===320,'original presentation switch failed');
    }
    const pixels=game.frame();
    const digest=[...new Uint8Array(await crypto.subtle.digest('SHA-256',pixels))].map(n=>n.toString(16).padStart(2,'0')).join('');
    assert(digest==='1539ca2afdcc9cfbe4fa5de51036c45b881014ec14a0ab030c191ca41f988c4d','original graphics/panel differ');
    const painted=doc.querySelector('#game').getContext('2d').getImageData(0,0,320,200).data;
    assert(pixels.every((v,i)=>painted[Math.floor(i/3)*4+i%3]===v),'initial original Canvas differs');
    let binary='';for(const value of pixels)binary+=String.fromCharCode(value);
    const frame_rgb=btoa(binary);
    let loaded=0;
    function select(rank,number) {
        doc.querySelector('#difficulty').value=rank;doc.querySelector('#level').value=number;
        doc.querySelector('#open-level').click();
        assert(!doc.querySelector('#error').textContent&&game.original()&&game.difficulty()===rank&&game.level()===number,'rank/level selection failed');
    }
    for(const rank of ['fun','tricky','taxing','mayhem'])for(let n=1;n<=30;n++){select(rank,n);loaded++;}
    select('fun',1);
    const {WebGame,WebAssets,WebAudio}=await win.eval('import("./pkg/lemmings_web.js")');
    const {AudioHost}=await win.eval('import("./audio.js")');
    let host,nonzero=0;
    const activate=AudioHost.prototype.activate,render=WebAudio.prototype.render;
    AudioHost.prototype.activate=function(){host=this;return activate.apply(this,arguments);};
    WebAudio.prototype.render=function(){const pcm=render.apply(this,arguments);nonzero+=pcm.filter(v=>v!==0).length;return pcm;};
    game.step(60);
    doc.querySelector('#sound').click();
    for(let n=0;n<100&&(!host?.submitted||!nonzero);n++){await pause(10);host?.service();}
    assert(host?.context?.state==='running'&&host.submitted>0&&nonzero>0,'original WebAudio PCM missing: '+JSON.stringify({host:!!host,error:host?.error,state:host?.context?.state,submitted:host?.submitted,nonzero}));
    const audio={state:host.context.state,submitted:host.submitted,nonzero};
    host.mute();await host.activate();assert(!host.error,'original sound failed to reactivate');host.mute();
    AudioHost.prototype.activate=activate;WebAudio.prototype.render=render;
    const bank=await loadBundled(WebAssets);
    const baseline=new WebGame('builder','reference');
    const transitions=[];
    for(const [action,target] of [['restart',1],['continue',2]]) {
        select('fun',1);game.step(60);game.command('AssignDigger',0);game.step(1000);
        assert(JSON.parse(game.state()).completion.phase==='Won','recorded original Fun01 solution failed');
        // Refresh product UI without changing simulation.
        doc.querySelector('#pause').click();
        doc.querySelector(action==='restart'?'#restart':'#continue').click();
        assert(game.level()===target&&JSON.parse(game.state()).completion.phase==='Playing','result UI transition failed');
        baseline.load_original_rank(bank,'fun',target);
        const expected=baseline.frame();
        assert(game.state()===baseline.state()&&game.frame().every((v,i)=>v===expected[i]),'transition differs from fresh original level');
        transitions.push({action,target,status:'PASS'});
    }
    baseline.free();bank.free();
    await host.dispose();first.detach();first.frame.remove();
    const refreshed=await open();assert(refreshed.game.level()===1,'fresh page did not reload bundled game');
    refreshed.detach();refreshed.frame.remove();
    const faults=['GAME-DATA.json','data/MAIN.DAT',...(hdDefault?['hd-assets/catalog.hdi','hd-assets/atlas-000.hdp']:[])];
    for(const broken of faults){const failed=await open(broken);failed.detach();failed.frame.remove();}
    const packageInfo=await (await fetch('./PACKAGE-MANIFEST.json')).json();
    const licenseFiles=packageInfo.project_license==='MIT'?['LICENSE','THIRD_PARTY_NOTICES.txt']:[];
    for(const name of ['audio-license.html','LGPL-2.1-or-later.txt','audio-source.json','audio-source.tar.gz',...licenseFiles]) {
        const response=await fetch('./'+name);assert(response.ok&&(await response.arrayBuffer()).byteLength>0,'source/license HTTP delivery failed');
    }
    result={status:'PASS',loader,original_levels_loaded:loaded,first_frame_sha256:digest,frame_rgb,audio,transitions,hd_png,
        hd:{default:hdDefault,info:hdInfo,levels:hdLevels,play:hdPlay,original_switch:true},
        source_delivery:true,project_license:packageInfo.project_license??'not declared in historical package',refresh:true,subdirectory:true,missing_file_errors:faults.length,scope:'Extracted full game; actual WASM/DOM/Canvas and original WebAudio PCM, automatic activation flag; no physical audibility or performance qualification'};
} catch(error){result={status:'FAIL',error:String(error),stack:error.stack};}
document.querySelector('#result').textContent=JSON.stringify({...result,frame_rgb:undefined,hd_png:undefined});
await fetch('/__result',{method:'POST',body:JSON.stringify(result)});
