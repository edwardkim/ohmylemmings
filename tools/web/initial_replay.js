import init,{WebGame,WebAssets} from './pkg/lemmings_web.js';
import {FramePump,present} from './host.js';
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const encode=s=>new TextEncoder().encode(s);
const assert=(v,label)=>{if(!v)throw new Error(label);};
const post=(path,value)=>fetch('/__initial/'+path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(value)});
let location={};
try {
    await init();const index=await(await fetch('./initial-index.json')).json();const bank=new WebAssets();
    for(const name of index.files)bank.insert(name,new Uint8Array(await(await fetch('./files/'+name)).arrayBuffer()));
    const context=document.querySelector('#screen').getContext('2d',{willReadFrequently:true});
    for(const id of index.ids) {
        const expected=await(await fetch('./'+id+'.json')).json();const [rank,number]=id.split('-');
        for(const [render,rate] of index.variants) {
            location={id,render};const game=new WebGame('builder','reference');game.load_original_rank(bank,rank,Number(number));
            try {
                assert(JSON.stringify(JSON.parse(game.asset_info()))===JSON.stringify(expected.initial.info),'initial metadata/palette differs');
                assert(await sha(game.materials())===expected.initial.materials,'initial steel/one-way material differs');
                let probe=0,packets=[],frames=0,checked=0,hidden=false,time=0;const frameHashes=[];
                function capture() {
                    const state=game.state();const maps=[game.terrain(),game.bricks(),game.objects()];let rgb=null;
                    if(probe%render===0||probe===index.steps) {
                        rgb=game.frame();present(context,rgb,320,200);const read=context.getImageData(0,0,320,200).data;
                        for(let i=0,j=0;i<read.length;i+=4,j+=3)assert(read[i]===rgb[j]&&read[i+1]===rgb[j+1]&&read[i+2]===rgb[j+2]&&read[i+3]===255,'Canvas readback differs');
                        frames++;
                    }
                    const view=JSON.parse(game.view_state());
                    assert(view.difficulty===rank&&view.level===Number(number)&&view.camera===Math.max(0,Math.min(1264,expected.initial.camera)),'rank/level/camera differs');
                    packets.push({probe,state,maps,rgb});
                }
                async function check() {
                    for(const packet of packets) {
                        location={id,render,probe:packet.probe};
                        const actual=await Promise.all([sha(encode(packet.state)),...packet.maps.map(sha)]);
                        for(let field=0;field<4;field++)if(actual[field]!==expected.rows[packet.probe][field]) {
                            location={...location,field:['state','terrain','bricks','objects'][field],actual,expected:expected.rows[packet.probe],actual_state:JSON.parse(packet.state)};
                            throw new Error('full boundary state/map differs');
                        }
                        if(packet.rgb) {
                            const digest=await sha(packet.rgb);assert(digest===expected.frames[String(packet.probe)],'full RGB differs at '+packet.probe);frameHashes.push([packet.probe,digest]);
                        }
                        checked++;
                    }
                    packets=[];
                }
                capture();await check();
                const driver={due:us=>game.due(us),step:()=>{
                    if(probe>=index.steps)return 0;
                    const count=game.step(1);
                    if(count===0)assert(JSON.parse(game.state()).completion.phase!=='Playing','unexplained frozen step');
                    // Count attempted probes separately from simulation control_step.
                    // Finished games are called again and must preserve the last state.
                    return 1;
                }};
                const pump=new FramePump(driver,3,phase=>{if(phase==='after'){probe++;capture();}});pump.frame(time);
                for(let deliveries=0;probe<index.steps&&deliveries<100000;deliveries++) {
                    time+=1000/rate;pump.frame(time);await check();
                    if(!hidden&&probe>=100) {
                        const before=game.state();pump.suspend();time+=3_600_000;pump.frame(time);assert(game.state()===before,'hidden time advanced session');hidden=true;
                    }
                    if(deliveries%100===0)await new Promise(resolve=>setTimeout(resolve,0));
                }
                assert(probe===index.steps&&checked===index.steps+1,'missing boundary');assert(game.input_log()==='','no-input replay queued commands');
                await post('progress',{id,render,rate,probes:checked,frames,hidden_resume:hidden,frame_hashes_sha256:await sha(encode(JSON.stringify(frameHashes))),final:JSON.parse(game.state()).completion});
                document.querySelector('#status').textContent=`${id} render${render}: PASS`;
            } finally {game.free();}
        }
    }
    bank.free();await post('done',{status:'PASS'});
} catch(error) {await post('fail',{status:'FAIL',...location,error:String(error),stack:error.stack});}
