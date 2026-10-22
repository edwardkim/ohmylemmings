import {DispatchEvents} from './dispatch_events.js';

// Drive the real application with a controlled rAF clock; no original assets.
export async function checkDispatch(assert) {
    const recorder=new DispatchEvents(3);
    recorder.record('input',{op:'Key',args:['ArrowLeft',1]},0,0,false);
    recorder.record('input',{op:'Pointer',args:['down',10,20,2,3,2]},1,0,false);
    recorder.record('present',{},2,0,false);
    const rows=recorder.result();
    assert(rows.keys[rows.events[0][4]]==='ArrowLeft'&&rows.events[0][5]===1,'diagnostic lost key/repeat');
    assert(JSON.stringify(rows.events[1].slice(5))==='[10,20,2,3,2]'&&rows.events[2][3]===0,'diagnostic lost pointer/present arguments');
    let overflow=false;try{recorder.record('present',{},3,0,false);}catch{overflow=true;}
    assert(overflow&&recorder.bytes===240,'diagnostic silently dropped events or changed capacity');
    const frame=document.createElement('iframe');
    const html=await (await fetch('./index.html')).text();
    const clock=`<script>
        let next=0;const pending=new Map();
        window.requestAnimationFrame=callback=>{pending.set(++next,callback);return next;};
        window.cancelAnimationFrame=id=>pending.delete(id);
        window.advanceFrame=timestamp=>{const callbacks=[...pending.values()];pending.clear();for(const callback of callbacks)callback(timestamp);};
    </script>`;
    frame.srcdoc=html.replace('<script type="module" src="./app.js">',clock+'<script type="module" src="./app.js">');
    const loaded=new Promise(resolve=>frame.addEventListener('load',resolve,{once:true}));
    document.body.append(frame);await loaded;
    const win=frame.contentWindow,doc=win.document;
    const app=await win.eval('import("./app.js")');
    const {AudioHost}=await win.eval('import("./audio.js")');
    const service=AudioHost.prototype.service;
    let game,presents=0,ticks=0,services=0;
    AudioHost.prototype.service=function(){services++;return service.apply(this,arguments);};
    const detach=app.observe((event,g)=>{game=g;if(event==='present')presents++;if(event==='after')ticks++;});
    try {
        const initial=game.state();
        win.advanceFrame(0);win.advanceFrame(1);
        assert(presents===0&&game.state()===initial,'unchanged rAF repeated RGB/state presentation');
        assert(services===2,'unchanged rAF starved real-time audio service');
        win.advanceFrame(75);
        assert(presents===1&&ticks===1,'admitted tick did not refresh the scene');
        const before=game.state();win.advanceFrame(76);
        assert(presents===1&&game.state()===before,'no-tick frame changed the scene');
        doc.querySelector('#pause').click();
        assert(game.paused()&&presents===2&&doc.querySelector('#status').textContent==='일시 정지','input did not update paused UI immediately');
        doc.querySelector('#restart').click();
        assert(!game.paused()&&presents===3,'restart rendered its replacement twice');
        const canvas=doc.querySelector('#game'),rect=canvas.getBoundingClientRect();
        canvas.dispatchEvent(new win.PointerEvent('pointerdown',{clientX:rect.left+20,clientY:rect.top+20,button:2,buttons:2,pointerId:1,isPrimary:true,cancelable:true}));
        assert(JSON.parse(game.view_state()).right&&presents===4,'right pointer input did not refresh');
        canvas.dispatchEvent(new win.Event('blur'));
        assert(!JSON.parse(game.view_state()).right,'blur retained right pointer');
        win.advanceFrame(77);
        assert(presents===5,'input release without an admitted tick did not refresh');
        const scene=doc.querySelector('#scene');scene.value='win';scene.dispatchEvent(new win.Event('change'));
        assert(presents===6,'scene replacement rendered twice');
        win.advanceFrame(100);
        assert(presents===6,'first replacement rAF repeated its synchronous presentation');
        // Every real simulation tick still refreshes, and completion stops rAF.
        for(let timestamp=175;timestamp<20000;timestamp+=75) {
            win.advanceFrame(timestamp);
            if(JSON.parse(game.state()).completion?.phase!=='Playing')break;
        }
        assert(JSON.parse(game.state()).completion.phase==='Won'&&!doc.querySelector('#continue').hidden,'completion UI was lost');
        const completed=presents;win.advanceFrame(21000);
        assert(presents===completed,'completed scene kept scheduling callbacks');
        return {status:'PASS',unchanged_callbacks_skip_rgb:true,audio_service_retained:true,ticks_refresh:true,input_immediate:true,single_replacement_draw:true,completion_stops:true};
    } finally {AudioHost.prototype.service=service;detach();frame.remove();}
}
