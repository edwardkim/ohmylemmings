import { WebGame, WebAssets } from './pkg/lemmings_web.js';
import { GameInput, logicalPoint, readFiles } from './controls.js';

async function checkProductRanks(assert) {
    const frame=document.createElement('iframe');
    const html=await (await fetch('./index.html')).text();
    frame.srcdoc=html.replace('<script type="module" src="./app.js">','<script>window.requestAnimationFrame=()=>0;</script><script type="module" src="./app.js">');
    const loaded=new Promise(resolve=>frame.addEventListener('load',resolve,{once:true}));
    document.body.append(frame);await loaded;
    const win=frame.contentWindow,doc=win.document;
    const app=await win.eval('import("./app.js")');
    let game;const detach=app.observe((type,g)=>{game=g;});
    const rank=doc.querySelector('#difficulty'),level=doc.querySelector('#level');
    const names=[...Array.from({length:10},(_,i)=>`LEVEL${String(i).padStart(3,'0')}.DAT`),'GROUND0O.DAT','VGAGR0.DAT','MAIN.DAT','ODDTABLE.DAT'];
    const files=await Promise.all(names.map(async name=>new win.File([await (await fetch('./winning/'+name)).arrayBuffer()],name)));
    async function selectFiles(values) {
        const transfer=new win.DataTransfer();for(const file of values)transfer.items.add(file);
        const input=doc.querySelector('#files');input.files=transfer.files;input.dispatchEvent(new win.Event('change',{bubbles:true}));
        for(let i=0;i<200&&input.disabled;i++)await new Promise(resolve=>setTimeout(resolve,10));
        assert(!input.disabled,'product file load did not finish');
    }
    try {
        assert(rank.options.length===4 && level.options.length===30,'product rank/level inventory');
        await selectFiles(files);assert(game.original(),'product file input did not load');
        for(const name of ['fun','tricky','taxing','mayhem'])for(const number of [1,30]) {
            rank.value=name;level.value=number;doc.querySelector('#open-level').click();
            assert(game.difficulty()===name&&game.level()===number,'product rank selection lost ID');
            const initial=game.state();game.step(80);doc.querySelector('#restart').click();
            assert(game.difficulty()===name&&game.level()===number&&game.state()===initial,'product restart lost rank/state');
            assert(doc.querySelector('#source').textContent.toLowerCase().includes(`${name} ${String(number).padStart(2,'0')}`),'product source label lost rank');
        }
        for(const [name,next] of [['fun','tricky'],['tricky','taxing'],['taxing','mayhem'],['mayhem',null]]) {
            rank.value=name;level.value=30;doc.querySelector('#open-level').click();game.step(1000);
            assert(JSON.parse(game.state()).completion.phase==='Won','invented exit fixture did not win');
            doc.dispatchEvent(new win.Event('visibilitychange'));
            const proceed=doc.querySelector('#continue');assert(!proceed.hidden,'result button hidden');
            assert(proceed.disabled===!next,'terminal Continue availability');
            if(next) {proceed.click();assert(game.difficulty()===next&&game.level()===1&&rank.value===next&&level.value==='1','product cross-rank Continue');}
        }
        const before=[game.state(),game.view_state(),game.sound_state()];
        await selectFiles([new win.File([new Uint8Array([0])],'LEVEL009.DAT')]);
        assert(JSON.stringify(before)===JSON.stringify([game.state(),game.view_state(),game.sound_state()]),'failed product file replacement changed current rank/session');
        assert(doc.querySelector('#error').textContent.includes('불러오기 실패'),'product file error missing');
        return {status:'PASS',rank_selections:8,restarts:8,cross_rank_continues:3,terminal_mayhem:true,atomic_file_failure:true};
    } finally {detach();frame.remove();}
}

export async function checkDom(assert, throws, bank) {
    const canvas=document.createElement('canvas');canvas.width=320;canvas.height=200;canvas.tabIndex=0;
    canvas.style.cssText='width:640px;height:400px;display:block';document.body.append(canvas);
    let game=new WebGame('builder','reference');
    const actions=[];
    const input=new GameInput(canvas,()=>game,name=>actions.push(name),()=>{},error=>{throw error;});
    const key=(code,repeat=false)=>canvas.dispatchEvent(new KeyboardEvent('keydown',{code,repeat,bubbles:true,cancelable:true}));
    const pointer=(kind,x,y,button=0,buttons=0)=>{
        const r=canvas.getBoundingClientRect();
        canvas.dispatchEvent(new PointerEvent('pointer'+kind,{clientX:r.left+x*r.width/320+0.1,clientY:r.top+y*r.height/200+0.1,button,buttons,pointerId:1,isPrimary:true,bubbles:true,cancelable:true}));
    };
    const reset=()=>{input.release();game.free();game=new WebGame('builder','reference');};
    try {
        assert(JSON.stringify(logicalPoint(canvas,{clientX:canvas.getBoundingClientRect().left-1,clientY:canvas.getBoundingClientRect().top}))==='[-1,0]','outside coordinate clamped into game');
        for (const [i,code,skill] of [[0,'KeyC','Climber'],[1,'KeyF','Floater'],[2,'KeyE','Bomber'],[3,'KeyK','Blocker'],[4,'KeyU','Builder'],[5,'KeyB','Basher'],[6,'KeyM','Miner'],[7,'KeyD','Digger']]) {
            reset();pointer('down',40+i*16,185,0,1);pointer('up',40+i*16,185);
            assert(JSON.parse(game.view_state()).skill===skill,'panel skill '+skill);
            key(code);game.step(80);
            const actor=JSON.parse(game.state()).session.actors[0];
            pointer('move',actor.x,actor.y-5);pointer('down',actor.x,actor.y-5,0,1);pointer('up',actor.x,actor.y-5);
            assert(JSON.parse(game.state()).inputs.includes('Assign'+skill+'(0)'), 'CSS-scaled DOM assignment '+skill);
        }
        reset();game.step(80);key('KeyU');
        let actor=JSON.parse(game.state()).session.actors[0];
        pointer('down',actor.x,actor.y-5,0,1);pointer('up',actor.x,actor.y-5);game.step(1);key('KeyE');
        actor=JSON.parse(game.state()).session.actors[0];
        assert(actor.action==='Building','right-click fixture not prioritized');
        pointer('down',actor.x,actor.y-5,2,2);
        // Mouse chords arrive as pointermove, because the pointer is already down.
        pointer('move',actor.x,actor.y-5,0,3);
        assert(JSON.parse(game.state()).inputs.includes('AssignPointer'),'left-while-right-held pointer chord lost');
        assert(JSON.parse(game.view_state()).right,'chord cleared right button');
        pointer('move',actor.x,actor.y-5,0,2);pointer('up',actor.x,actor.y-5,2,0);
        reset();key('Space');key('Space',true);assert(game.paused(),'repeat toggled pause');key('Space');assert(!game.paused(),'queued resume lost');
        pointer('down',24,185,0,1);game.step(3);pointer('up',-5,-5);
        const count=JSON.parse(game.state()).inputs.match(/SetReleaseRate/g).length;game.step(2);
        assert(JSON.parse(game.state()).inputs.match(/SetReleaseRate/g).length===count,'outside release retained hold');
        pointer('down',280,185,0,1);pointer('move',250,185,0,1);assert(JSON.parse(game.view_state()).camera>0,'minimap drag');
        pointer('down',70,80,2,3);window.dispatchEvent(new Event('blur'));
        assert(!JSON.parse(game.view_state()).right && JSON.parse(game.view_state()).held==='None','blur retained input');
        reset();
        pointer('down',184,185,0,1);pointer('up',184,185);canvas.dispatchEvent(new Event('lostpointercapture'));
        assert(!JSON.parse(game.state()).inputs.includes('Nuke'),'single click nuked');
        pointer('down',184,185,0,1);pointer('up',184,185);canvas.dispatchEvent(new Event('lostpointercapture'));
        assert(JSON.parse(game.state()).inputs.includes('Nuke'),'double click lost history at automatic capture release');
        reset();game.load_original(bank,1);const before=game.state(),view=game.view_state();
        const broken=new WebAssets();broken.insert('LEVEL009.DAT',new Uint8Array([0]));
        throws(()=>game.load_original(broken,1),'damaged selected file accepted');broken.free();
        assert(game.state()===before && game.view_state()===view,'failed file load discarded game');
        const files=[new File([new Uint8Array([1])],'MAIN.DAT'),new File([new Uint8Array([2])],'main.dat')];
        let duplicate=false;try { const b=await readFiles(files);b.free(); } catch { duplicate=true; }
        assert(duplicate,'case-insensitive duplicate selected files accepted');
        const sourceBytes=new Uint8Array([3,4]);const one=await readFiles([new File([sourceBytes],'LEVEL009.DAT'),new File(['ignore'],'README.TXT')]);
        assert(one.count()===1,'unrelated selected files not ignored');one.free();
        game.start_level(30);assert(game.level()===30 && JSON.parse(game.view_state()).next===1 && JSON.parse(game.view_state()).next_difficulty==='tricky','rank boundary lost next difficulty');
        reset();game.reset('win','reference');game.step(1000);key('Enter',true);assert(actions.at(-1)==='none','repeated result key accepted');key('Enter');assert(actions.at(-1)==='continue','result key action missing');
        game.transition('continue');assert(game.level()===2,'win did not advance');game.step(1000);game.transition('continue');assert(game.level()===2 && JSON.parse(game.state()).completion.phase==='Playing','loss did not retry');
        const ranks=await checkProductRanks(assert);
        return {status:'PASS',ranks,scaled_assignments:8,panel_skills:8,held_release:true,focus_release:true,double_click_capture:true,right_chord_fallback:true,local_files:true,atomic_replacement:true,result_lifecycle:true};
    } finally { input.release();game.free();game=null;canvas.remove(); }
}
