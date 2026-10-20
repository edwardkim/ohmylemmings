import init, { WebGame, WebAssets } from './pkg/lemmings_web.js';
import { FramePump, present } from './host.js';
import { checkDom } from './dom_check.js';

const assert = (condition, label) => { if (!condition) throw new Error(label); };
const throws = (fn, label) => { let did = false; try { fn(); } catch { did = true; } assert(did, label); };
const sha = async bytes => Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), b => b.toString(16).padStart(2,'0')).join('');
const report = document.querySelector('#result');
try {
    await init();
    const expected = await (await fetch('./expected.json')).json();
    const context = document.querySelector('#screen').getContext('2d', { willReadFrequently: true });
    const bank = new WebAssets();
    for (const name of [...Array.from({length:10},(_,i)=>`LEVEL${String(i).padStart(3,'0')}.DAT`),'GROUND0O.DAT','VGAGR0.DAT','MAIN.DAT','ODDTABLE.DAT']) bank.insert(name,new Uint8Array(await (await fetch('./fixture/'+name)).arrayBuffer()));
    const originals=new WebAssets();
    for (const name of expected.original_files || []) originals.insert(name,new Uint8Array(await (await fetch('./original/'+name)).arrayBuffer()));
    function operation(game, row) {
        const [,name,...args]=row;
        switch(name) {
            case 'Key': game.key(args[0],Boolean(args[1])); break;
            case 'Pointer': game.pointer(...args); break;
            case 'Release': game.release_input(); break;
            case 'Restart': game.restart_level(); break;
            case 'Continue': game.transition('continue'); break;
            case 'Level': game.start_level(args[0]); break;
            default: game.command(name,args[0]);
        }
    }
    let states = 0, frames = 0, readbacks = 0;
    const rows = [];
    for (const test of expected.cases) {
        for (const render of [1, 7]) {
            const game = new WebGame(test.scene.startsWith('original:') ? 'builder' : test.scene, test.pacing);
            if (test.scene.startsWith('original:')) game.load_original(test.original ? originals : bank,Number(test.scene.split(':')[1]));
            for (let step = 0; step < test.rows.length; step++) {
                for (const row of test.commands) if (row[0] === step) operation(game,row);
                const row = test.rows[step];
                const state = JSON.parse(game.state());
                assert(JSON.stringify(state) === JSON.stringify(row.state), `${test.name}/${render}/${step}: full state`);
                for (const name of ['terrain','bricks','objects']) {
                    assert(await sha(game[name]()) === row[name], `${test.name}/${render}/${step}: ${name}`);
                }
                assert(JSON.stringify(JSON.parse(game.view_state()))===JSON.stringify(row.view),`${test.name}/${render}/${step}: presentation state`);
                states++;
                if (step % render === 0 || step === test.rows.length - 1) {
                    const before = game.state();
                    const frame = game.frame();
                    assert(await sha(frame) === row.frame, `${test.name}/${render}/${step}: all RGB bytes`);
                    present(context, frame, game.width(), game.height());
                    const rgba = context.getImageData(0,0,320,200).data;
                    const rgb = new Uint8Array(320*200*3);
                    for (let i=0,j=0;i<rgba.length;i+=4,j+=3) {
                        assert(rgba[i+3] === 255, 'Canvas alpha');rgb.set(rgba.subarray(i,i+3),j);
                    }
                    assert(await sha(rgb) === row.frame, `${test.name}/${render}/${step}: Canvas readback`);
                    assert(game.state() === before, 'render changed game');frames++;readbacks++;
                    frame.fill(17); // Exported arrays must not alias the live Rust frame.
                    assert(await sha(game.frame()) === row.frame, 'frame copy aliases game');
                }
                if (step < test.rows.length - 1) assert(game.step(1) === (state.completion && state.completion.phase!=='Playing' ? 0 : 1), 'wrong explicit tick count');
            }
            rows.push({name:test.name,render,states:test.rows.length});game.free();
        }
    }
    const game = new WebGame('builder','reference');
    const original = game.state();
    for (const bad of [NaN,Infinity,-1,0.5,3600000001]) throws(()=>game.due(bad), 'invalid elapsed accepted');
    for (const bad of [NaN,Infinity,-1,0.5,1001,4294967296]) throws(()=>game.step(bad), 'invalid step accepted');
    for (const [name,value] of [['unknown',0],['Pause',1],['SetReleaseRate',49],['SetReleaseRate',100],['AssignDigger',-1],['AssignDigger',0.5],['AssignDigger',4294967296],['Nuke',NaN]]) throws(()=>game.command(name,value),'invalid command accepted');
    throws(()=>game.reset('unknown','reference'),'invalid reset accepted');
    assert(game.state() === original, 'invalid input changed state');
    const toggle = new WebGame('builder','reference');
    toggle.command('Pause',0);assert(toggle.paused(),'queued Pause missing');
    toggle.command('Resume',0);assert(!toggle.paused(),'same-boundary Resume missing');toggle.free();
    assert(game.due(58823) === 0 && game.due(1) === 1, 'microsecond phase');
    assert(game.state() === original, 'due executed simulation');
    const pumpGame = new WebGame('builder','reference');
    const pump = new FramePump(pumpGame, 3);
    assert(pump.frame(0) === 0, 'first frame ticks');
    assert(pump.frame(1000) === 3 && pump.pending === 14, 'frame budget lost debt');
    pump.suspend();assert(pump.pending === 14, 'hidden lost admitted debt');
    pump.frame(36000000); // Hidden wall time excluded, pending work retained.
    while (pump.pending) pump.frame(36000000);
    assert(JSON.parse(pumpGame.state()).session.control_step === 17, 'wrong admitted ticks');
    const before=pumpGame.state();const last=pump.lastUs;
    throws(()=>pump.frame(NaN),'invalid animation timestamp accepted');
    throws(()=>pump.frame(0),'backwards animation timestamp accepted');
    assert(pumpGame.state()===before && pump.lastUs===last,'invalid animation timestamp changed state');
    for (const rate of [60,120,144]) {
        const partitioned=new WebGame('builder','reference');const clock=new FramePump(partitioned);
        clock.frame(0);for(let i=1;i<=rate;i++) clock.frame(i*1000/rate);
        assert(JSON.parse(partitioned.state()).session.control_step===17,`refresh ${rate} changed logical rate`);partitioned.free();
    }
    game.reset('win','reference');assert(game.step(1000)>0,'win did not tick');const frozen=game.state();
    assert(JSON.parse(frozen).completion.phase==='Won','synthetic win did not finish');
    assert(game.step(1000)===0,'finished ticked');throws(()=>game.command('Nuke',0),'finished accepted input');assert(game.state()===frozen,'finished changed');
    game.free();pumpGame.free();
    const dom = await checkDom(assert,throws,bank);
    bank.free();originals.free();
    const result={dom,status:'PASS',cases:expected.cases.length,runs:rows.length,states,frames,canvas_readbacks:readbacks,boundary_checks:true,refresh_rates:[60,120,144],runs_detail:rows};
    report.textContent=JSON.stringify(result);await fetch('/__result',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(result)});
} catch (error) {
    const result={status:'FAIL',error:String(error),stack:error.stack};report.textContent=JSON.stringify(result);
    await fetch('/__result',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(result)});
}
