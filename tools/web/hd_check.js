import { WebGame } from './pkg/lemmings_web.js';
import { HdCanvas } from './hd.js';
export async function checkHd(assert,throws) {
    const response=await fetch('./hd-assets.hdp');assert(response.ok,'HD pack delivery');
    const bytes=new Uint8Array(await response.arrayBuffer());
    const canvas=document.createElement('canvas');canvas.width=1920;canvas.height=1080;
    const context=canvas.getContext('2d',{willReadFrequently:true});
    const game=new WebGame('builder','reference'),plain=new WebGame('builder','reference');
    const host=new HdCanvas(game),slots=new Set();let frames=0,states=0;
    // Reuse the immutable atlas set across scenes; load/CRC/copy only once.
    // This also exercises scene reset and dynamic host cache invalidation.
    const scenarios=[['builder','AssignBuilder',260],['builder','AssignDigger',260],['builder','AssignBomber',260],
        ['builder','AssignMiner',500],['builder','AssignBlocker',300],['basher','AssignBasher',650],
        ['climber','AssignClimber',1050],['floater','AssignFloater',1000],['floater',null,650],
        ['builder','AssignBuilder',1150],['win',null,500],['loss',null,500],['fire',null,500]];
    try {
        const before=game.state();game.load_hd(bytes);assert(game.state()===before,'HD loading changed simulation');
        assert(JSON.parse(game.hd_info()).actor_frames_available===337,'complete character pack unavailable');
        const ui=JSON.parse(game.hd_info());assert(ui.ui_glyphs===95&&ui.ui_font_atlases===3,'HD UI font coverage');
        for(let page=0;page<3;page++){const pixels=game.hd_texture(ui.ui_font_first+page);assert(pixels.length===1024*384*4&&pixels.some((v,i)=>i%4===3&&v>0&&v<255),'HD font lost antialiasing');}
        for(const [scene,job,total] of scenarios) {
            game.reset(scene,'reference');plain.reset(scene,'reference');host.invalidate();
            for(let step=0;step<=total;step++) {
                if(step===80&&job){game.command(job,0);plain.command(job,0);}
                assert(game.state()===plain.state(),'HD changed deterministic state '+scene+'/'+job+'/'+step);states++;
                if(step%10===0||(job==='AssignBomber'&&step>=160&&step<=215)) {
                    const old=game.state(),sound=game.sound_state();host.draw(context);
                    const info=JSON.parse(game.hd_info());
                    assert(info.missing_actor_frames.length===0,'original actor fallback in complete pack');
                    for(const [slot] of info.hd_actor_frames)slots.add(slot);
                    assert(game.state()===old&&game.sound_state()===sound,'HD rendering changed state/audio');frames++;
                }
                if(step<total){game.step(1);plain.step(1);}
            }
            for(const name of ['terrain','bricks','objects'])assert(Array.from(game[name]()).join(',')===Array.from(plain[name]()).join(','),'HD map differs '+name);
            const cached=JSON.parse(game.hd_info()).terrain_rebuilds;host.draw(context);
            assert(JSON.parse(game.hd_info()).terrain_rebuilds===cached,'unchanged HD terrain rebuilt');
        }
        assert([0,4,10,26,27].every(id=>slots.has(id)),'previous action slots missing');
        for(const pair of [[12,13],[14,15],[5,6],[18,19],[23],[21],[7],[22],[20]])assert(pair.some(id=>slots.has(id)),'new character action not observed '+pair);
        const state=game.state();const broken=bytes.slice();broken[0]^=1;
        throws(()=>game.load_hd(broken),'corrupt HD pack accepted');assert(game.state()===state,'failed HD load changed state');host.draw(context);
        game.reset('builder','reference');plain.reset('builder','reference');host.invalidate();
        game.hd_pointer('down',1430,1000,0,1,1);plain.key('Space',false);assert(game.state()===plain.state(),'HD pause differs');
        const stable=game.state(),view=game.view_state();throws(()=>game.hd_pointer('down',NaN,20,0,1,1),'invalid HD pointer accepted');assert(game.state()===stable&&game.view_state()===view,'invalid HD pointer mutated state');
        assert(game.hd_pointer('down',1600,1000,0,1,1)==='restart-level','HD restart target');
        game.restart_level();host.invalidate();host.draw(context);assert(JSON.parse(game.state()).session.control_step===0,'HD restart did not reset');
        const pixels=context.getImageData(0,0,1920,1080).data;assert(pixels.some((v,i)=>i%4!==3&&v!==0),'empty HD canvas');
        return {status:'PASS',wire_version:1,scenarios:scenarios.length,frames,states,actor_frames_available:337,observed_slots:[...slots].sort((a,b)=>a-b),no_actor_fallback:true,unchanged_logic_masks_audio:true,dirty_chunks:true,corrupt_pack:true,input:true,scope:'full character pack in actual WASM HD renderer/Canvas with synthetic state pairs; not a performance or all-level qualification'};
    } finally{game.free();plain.free();}
}
