// Exercise the real application, including the user-facing controls and clipboard fallback.
export async function checkFeedback(assert) {
    const html=await(await fetch('./index.html')).text();
    const clock=`<script>
        let now=0,next=0;const pending=new Map();
        Object.defineProperty(performance,'now',{value:()=>now});
        window.requestAnimationFrame=callback=>{pending.set(++next,callback);return next;};
        window.cancelAnimationFrame=id=>pending.delete(id);
        window.feedbackFrame=value=>{now=value;const callbacks=[...pending.values()];pending.clear();for(const cb of callbacks)cb(now);};
        Object.defineProperty(navigator,'clipboard',{value:{writeText:async text=>{window.copiedFeedback=text;}},configurable:true});
    </script>`;
    const frame=document.createElement('iframe');
    frame.srcdoc=html.replace('<script type="module" src="./app.js">',clock+'<script type="module" src="./app.js">');
    const loaded=new Promise(resolve=>frame.addEventListener('load',resolve,{once:true}));
    document.body.append(frame);await loaded;
    const win=frame.contentWindow,doc=win.document;
    const app=await win.eval('import("./app.js")');
    let game,presents=0;const detach=app.observe((type,g)=>{game=g;if(type==='present')presents++;});
    const toggle=doc.querySelector('#fps-toggle'),copy=doc.querySelector('#fps-copy'),report=doc.querySelector('#fps-report');
    const key=(code,extra={})=>doc.querySelector('#game').dispatchEvent(new win.KeyboardEvent('keydown',{code,bubbles:true,cancelable:true,...extra}));
    const read=async()=>{copy.click();await Promise.resolve();return JSON.parse(report.value);};
    try {
        assert(doc.querySelector('#fps-hud').hidden&&copy.disabled,'feedback should start off');
        const state=game.state();key('F3');win.feedbackFrame(100);win.feedbackFrame(116);win.feedbackFrame(132);
        assert(!doc.querySelector('#fps-hud').hidden&&presents===0&&game.state()===state,'FPS display changed game/RGB');
        let result=await read();
        assert(result.summary.fps===62.5&&result.summary.draws_per_second===0,'callback FPS was confused with Canvas writes');
        assert(win.copiedFeedback===report.value,'copy did not use the displayed feedback');
        key('F3',{repeat:true});assert(toggle.getAttribute('aria-pressed')==='true','repeat toggled HUD');
        key('F3',{ctrlKey:true});assert(toggle.getAttribute('aria-pressed')==='true','modified shortcut toggled HUD');
        doc.querySelector('#pause').click();const intervals=(await read()).summary.intervals;
        win.feedbackFrame(100000);assert((await read()).summary.intervals===intervals,'pause time counted as gameplay stutter');
        doc.querySelector('#pause').click();win.feedbackFrame(100016);win.feedbackFrame(100032);
        result=await read();assert(result.summary.slow_intervals===0,'resume joined the paused gap');
        doc.querySelector('#restart').click();assert((await read()).summary.intervals===0,'level reset retained old samples');
        Object.defineProperty(win.navigator,'clipboard',{value:{writeText:async()=>{throw Error('denied');}}});
        key('F2');await new Promise(resolve=>setTimeout(resolve,0));
        assert(!report.hidden&&doc.activeElement===report&&report.selectionEnd===report.value.length,'manual copy unavailable when clipboard denied');
        assert(report.value.includes('체감')&&!report.value.includes('musicFile'),'feedback missing prompt or leaking music metadata');
        key('F3');assert(doc.querySelector('#fps-hud').hidden&&copy.disabled,'F3 did not disable HUD');
        return {status:'PASS',opt_in:true,no_extra_canvas_draw:true,cadence_separate:true,pause_resume:true,reset:true,clipboard_and_manual:true};
    } finally {detach();frame.remove();}
}
