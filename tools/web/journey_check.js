import * as app from './app.js';
import { JourneyRecorder } from './journey_recorder.js';
const assert=(condition,message)=>{if(!condition)throw new Error(message);};
const recorder=new JourneyRecorder(app,{renderEvery:13});
const canvas=document.querySelector('#game');let clock=0;let frames=0;
function state(){return recorder.read().state;}
async function advance(predicate,limit=4000) {
    for(let i=0;i<limit;i++) {
        if(predicate())return;
        const callbacks=Array.from(window.__frames.values());window.__frames.clear();
        clock+=[7,11,20,53,63][frames++%5];for(const callback of callbacks)callback(clock);
        if(i%20===0)await new Promise(r=>setTimeout(r,0));
    }
    throw new Error('journey did not reach boundary');
}
function pointer(kind,x,y,buttons=0) {
    const r=canvas.getBoundingClientRect();canvas.dispatchEvent(new PointerEvent('pointer'+kind,{isPrimary:true,pointerId:1,button:0,buttons,clientX:r.left+(x+.1)*r.width/320,clientY:r.top+(y+.1)*r.height/200,bubbles:true}));
}
function click(x,y){pointer('move',x,y);pointer('down',x,y,1);pointer('up',x,y);}
function key(code){canvas.dispatchEvent(new KeyboardEvent('keydown',{code,bubbles:true,cancelable:true}));}
function frozen() {
    const before=JSON.stringify(recorder.read());const callbacks=Array.from(window.__frames.values());window.__frames.clear();clock+=10000;
    for(const callback of callbacks)callback(clock);
    assert(JSON.stringify(recorder.read())===before,'frozen result advanced');
}
async function win(toolbar) {
    await advance(()=>state().session.actors[0]?.action==='Walking'&&state().session.control_step>=75);
    click(40,190);const actor=state().session.actors[0];click(actor.x-recorder.read().view.camera,actor.y-5);
    if(toolbar) {
        click(164,190);await advance(()=>state().session.paused);
        click(24,190);pointer('down',24,190,1);const step=state().session.control_step;await advance(()=>state().session.control_step>=step+4);pointer('up',24,190);
        click(300,190);click(208,190);key('Home');click(164,190);
    }
    await advance(()=>state().completion.phase!=='Playing');assert(state().completion.phase==='Won'&&state().completion.rescued_percent===100,'full rescue required');
}
try {
    const scene=document.querySelector('#scene');scene.value='win';scene.dispatchEvent(new Event('change'));
    await win(true);frozen();key('KeyR');
    await advance(()=>state().session.control_step>=80);
    click(148,190);let actor=state().session.actors[0];click(actor.x,actor.y-5);
    await advance(()=>state().session.control_step>=90);
    click(180,190);assert(!state().inputs.includes('Nuke'),'single click nuked');
    // Keep the single click distinct from the following double click.
    await new Promise(r=>setTimeout(r,550));click(180,190);click(180,190);
    await advance(()=>state().completion.phase!=='Playing');assert(state().completion.phase==='Lost','Nuke did not fail');
    frozen();click(160,130);await win(false);frozen();key('Enter');
    await advance(()=>state().session.control_step>=80);key('Escape');
    const data=recorder.stop();
    assert(data.attempts.length===4,'four attempts required');
    // The observer must see every tick even when an animation frame admits a batch.
    const {FramePump}=await import('./host.js');const {WebGame}=await import('./pkg/lemmings_web.js');
    for(const rate of [60,120,144]) {
        const game=new WebGame('builder','reference');const seen=[];const pump=new FramePump(game,3,(phase,g)=>{if(phase==='before')seen.push(JSON.parse(g.state()).session.control_step);});
        pump.frame(0);for(let i=1;i<=rate;i++)pump.frame(i*1000/rate);while(pump.pending)pump.frame(1000);
        assert(JSON.stringify(seen)===JSON.stringify(Array.from({length:17},(_,i)=>i)),'observer lost tick at '+rate);game.free();
    }
    const {replayJourney}=await import('./journey_replay.js');
    data.browser_replays=await replayJourney(data);
    await fetch('/__journey',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
    document.body.dataset.journey='PASS';
} catch(error) {
    await fetch('/__journey',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({error:String(error),stack:error.stack})});
    document.body.dataset.journey='FAIL';
}
