import {checkDispatch} from './__dispatch_check.js';
import init,{WebGame,WebAudio} from './pkg/lemmings_web.js';
import {AudioHost} from './audio.js';

const assert=(condition,message)=>{if(!condition)throw Error(message);};
let result;
try {
    const dispatch=await checkDispatch(assert);
    await init();
    const game=new WebGame('builder','reference');
    game.step(55);
    const audio=new AudioHost(()=>game,WebAudio);
    try {
        await audio.activate();
        assert(!audio.error&&audio.context?.state==='running'&&audio.submitted>0,'packaged Web Audio did not submit samples');
        const sound={state:audio.context.state,submitted:audio.submitted};
        audio.mute();
        await audio.activate();
        assert(!audio.error&&audio.context.state==='running','packaged sound could not resume');
        const packageInfo=await (await fetch('./PACKAGE-MANIFEST.json')).json();
        for(const name of ['audio-license.html','LGPL-2.1-or-later.txt','audio-source.json',...(packageInfo.source_delivery?[]:['audio-source.tar.gz'])]) {
            const response=await fetch('./'+name);
            assert(response.ok&&(await response.arrayBuffer()).byteLength>0,'packaged source/license HTTP delivery failed: '+name);
        }
        result={status:'PASS',dispatch,audio:sound,source_delivery:true,scope:'extracted package; real WASM/DOM and Web Audio; automatic activation flag; no human audibility or performance qualification'};
    } finally {
        audio.mute();
        if(audio.context)await audio.context.close();
        audio.pipe?.free();game.free();
    }
} catch(error) {result={status:'FAIL',error:String(error),stack:error.stack};}
document.querySelector('#result').textContent=JSON.stringify(result);
await fetch('/__result',{method:'POST',body:JSON.stringify(result)});
