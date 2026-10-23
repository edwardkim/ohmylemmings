import { WebGame } from './pkg/lemmings_web.js';
import { HdCanvas } from './hd.js';
export async function checkHd(assert,throws) {
    const response=await fetch('./hd-assets.hdp');assert(response.ok,'HD pack delivery');
    const bytes=new Uint8Array(await response.arrayBuffer());
    const canvas=document.createElement('canvas');canvas.width=1920;canvas.height=1080;
    const context=canvas.getContext('2d',{willReadFrequently:true});
    const seen=new Set(),slots=new Set();let frames=0;
    for(const job of ['AssignBuilder','AssignDigger','AssignBomber']) {
        const game=new WebGame('builder','reference'),plain=new WebGame('builder','reference');
        try {
            const before=game.state();game.load_hd(bytes);assert(game.state()===before,'HD loading changed simulation');
            const host=new HdCanvas(game);
            for(let step=0;step<=260;step++) {
                if(step===80){game.command(job,0);plain.command(job,0);}
                assert(game.state()===plain.state(),'HD changed deterministic state '+job+'/'+step);
                if(step%5===0 || (job==='AssignBomber'&&step>=160&&step<=215)) {
                    const old=game.state(),sound=game.sound_state();
                    const commands=game.hd_commands();
                    assert(commands.length%12===0,'HD wire shape');
                    for(let i=0;i<commands.length;i+=12)if(commands[i]===1&&commands[i+1]<5)seen.add(commands[i+1]);
                    host.draw(context);for(const [slot] of JSON.parse(game.hd_info()).hd_actor_frames)slots.add(slot);assert(game.state()===old&&game.sound_state()===sound,'HD rendering changed state/audio');frames++;
                }
                if(step<260){game.step(1);plain.step(1);}
            }
            assert(Array.from(game.terrain()).join(',')===Array.from(plain.terrain()).join(','),'HD terrain differs');
            assert(Array.from(game.bricks()).join(',')===Array.from(plain.bricks()).join(','),'HD bricks differ');
            const cached=JSON.parse(game.hd_info()).terrain_rebuilds;host.draw(context);assert(JSON.parse(game.hd_info()).terrain_rebuilds===cached,'unchanged HD terrain rebuilt');
            const state=game.state();const broken=bytes.slice();broken[0]^=1;throws(()=>game.load_hd(broken),'corrupt HD pack accepted');assert(game.state()===state,'failed HD load changed state');host.draw(context);
            game.hd_pointer('down',1430,1000,0,1,1);plain.key('Space',false);assert(game.state()===plain.state(),'HD pause differs from keyboard');
            const stable=game.state(),view=game.view_state();throws(()=>game.hd_pointer('down',NaN,20,0,1,1),'invalid HD pointer accepted');assert(game.state()===stable&&game.view_state()===view,'invalid HD pointer mutated state');
            assert(game.hd_pointer('down',1600,1000,0,1,1)==='restart-level','HD restart target');
            game.restart_level();host.invalidate();host.draw(context);assert(JSON.parse(game.state()).session.control_step===0,'HD restart did not reset');
        } finally{game.free();plain.free();}
    }
    assert([0,4,10,26,27].every(id=>slots.has(id)),'HD action slots were not all observed');
    assert([0,2,3,4].every(id=>seen.has(id)),'walking/digging/building/bombing atlas not drawn');
    const pixels=context.getImageData(0,0,1920,1080).data;assert(pixels.some((v,i)=>i%4!==3&&v!==0),'empty HD canvas');
    return {status:'PASS',wire_version:1,scenarios:3,frames,states:783,unchanged_logic_masks_audio:true,dirty_chunks:true,corrupt_pack:true,input:true,scope:'shared WASM HD renderer/Canvas and synthetic state pairs; actual Fun01 tracked separately'};
}
