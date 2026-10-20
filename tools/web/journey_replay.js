import { WebGame } from './pkg/lemmings_web.js';
import { FramePump } from './host.js';
const assert=(value,message)=>{if(!value)throw new Error(message);};

export async function replayJourney(data,bank=null) {
    const decode=value=>Uint8Array.from(atob(value),c=>c.charCodeAt(0));
    let replayStates=0,replayFrames=0;
    for(const [render,rate] of [[1,60],[7,120],[13,144]])for(let a=0;a<data.attempts.length;a++) {
        const attempt=data.attempts[a],game=new WebGame('win','reference');
        if(data.original)game.load_original(bank,attempt.level);
        else if(a===3){game.step(1000);game.transition('continue');}
        const maps=Object.fromEntries(['terrain','bricks','objects'].map(n=>[n,decode(attempt.initial.maps[n])]));
        let seen=0;const last=attempt.rows.length-1;
        function boundary() {
            if(seen>last)return;
            const step=JSON.parse(game.state()).session.control_step;
            assert(step===seen,'replay lost control boundary');
            for(const event of attempt.events.filter(e=>e.step===step)) {
                if(event.op==='Key')game.key(event.args[0],Boolean(event.args[1]));
                else if(event.op==='Pointer')game.pointer(...event.args);
                else if(event.op==='Release')game.release_input();
                else if(event.op==='Frame') {
                    if(step%render===0||step===last) {
                        const frame=attempt.frames[event.args[0]],rgb=decode(frame.rgb),actual=game.frame();
                        assert(actual.length===rgb.length&&actual.every((v,i)=>v===rgb[i]),'render interval changed pixels');replayFrames++;
                    }
                } else game.command(event.op,event.args[0]);
            }
            assert(game.state()===JSON.stringify(attempt.rows[step].state),'elapsed partition changed full state');
            for(const name of ['terrain','bricks','objects']) {
                for(const [i,before,after] of attempt.rows[step].changes[name]){assert(maps[name][i]===before,'bad replay delta');maps[name][i]=after;}
                const actual=game[name]();assert(actual.length===maps[name].length&&actual.every((v,i)=>v===maps[name][i]),'render/clock changed map bytes');
            }
            seen++;replayStates++;
        }
        const host={due:us=>game.due(us),step:()=>JSON.parse(game.state()).session.control_step<last?game.step(1):0};
        const replayPump=new FramePump(host,3,phase=>{
            if(phase==='before')boundary();
            else if(JSON.parse(game.state()).session.control_step===last)boundary();
        });
        let time=0;replayPump.frame(time);
        for(let n=0;seen<=last&&n<100000;n++){time+=1000/rate;replayPump.frame(time);if(n%200===0)await new Promise(r=>setTimeout(r,0));}
        assert(seen===attempt.rows.length,'browser replay incomplete');game.free();
    }
    return {runs:12,states:replayStates,frames:replayFrames,render_intervals:[1,7,13],refresh_rates:[60,120,144]};
}
