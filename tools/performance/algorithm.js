import init,{WebGame,WebAssets} from './pkg/lemmings_web.js';
import initPcm,{PerformancePcm} from './pcm/audio_probe.js';
import {present} from './host.js';
const hash=async data=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',data)),b=>b.toString(16).padStart(2,'0')).join('');
const assert=(v,s)=>{if(!v)throw Error(s);};
const ns=start=>Math.max(1,Math.round((performance.now()-start)*1e6));
const now=()=>performance.now();
const tick=(g,commands,at,count)=>{for(let i=at;i<at+count;i++){for(const c of commands)if(c[0]===i)g.command(c[2],c[3]);assert(g.step(1)===1,'frozen work cannot count as a tick');}};
window.__performanceResult=null;
try {
 const wasm=await init();const pcmWasm=await initPcm();const plan=await(await fetch('./plan.json')).json();const bank=new WebAssets();
 for(const name of plan.files)bank.insert(name,new Uint8Array(await(await fetch('./files/'+name)).arrayBuffer()));
 const rows=[];const report=document.querySelector('#status');const canvas=document.querySelector('#game');const context=canvas.getContext('2d',{alpha:false});
 for(const id of plan.levels){
  const [rank,n]=id.split('-');const samples=[];
  for(let i=0;i<33;i++){
   const g=new WebGame('builder','dos');const start=now();g.load_original_rank(bank,rank,Number(n));const time=ns(start);
   const digest=await hash(new TextEncoder().encode(g.asset_info()));samples.push([time,1,digest,0]);g.free();
  }
  rows.push({kind:'wasm_load',id,operations:1,samples});report.textContent=`로딩 측정 ${id}`;await new Promise(r=>setTimeout(r,0));
 }
 for(const c of plan.cases){
  const [rank,n]=c.id.split('-');const times=[],frames=[],copies=[];let final;
  for(let i=0;i<33;i++){
   const initStart=now();const g=new WebGame('builder','dos');g.load_original_rank(bank,rank,Number(n));tick(g,c.commands,0,c.start);const initNs=ns(initStart);
   const start=now();tick(g,c.commands,c.start,c.steps);const tickNs=ns(start);
   const state=g.state(),maps=[g.terrain(),g.bricks(),g.objects()];
   const hashes=await Promise.all([hash(new TextEncoder().encode(state)),...maps.map(hash)]);
   assert(JSON.stringify(hashes)===JSON.stringify(c.expected.slice(0,4)),c.id+' state/map comparison');
   times.push([tickNs,c.steps,hashes.join(':'),initNs]);
   let rgb;const renderStart=now();for(let j=0;j<32;j++)rgb=g.frame();const renderNs=ns(renderStart);const rgbHash=await hash(rgb);
   assert(rgbHash===c.expected[4],c.id+' RGB comparison');frames.push([renderNs,32,rgbHash,0]);
   const copyStart=now();for(let j=0;j<32;j++)present(context,rgb,320,200);const copyNs=ns(copyStart);
   const rgba=context.getImageData(0,0,320,200).data;for(let s=0,d=0;s<rgb.length;s+=3,d+=4)assert(rgba[d]===rgb[s]&&rgba[d+1]===rgb[s+1]&&rgba[d+2]===rgb[s+2]&&rgba[d+3]===255,'Canvas bytes');
   copies.push([copyNs,32,rgbHash,0]);final=JSON.parse(state).completion;g.free();
  }
  rows.push({kind:'wasm_tick',id:c.id,operations:c.steps,samples:times,final},{kind:'wasm_cpu_frame',id:c.id,operations:32,samples:frames},{kind:'canvas_transfer',id:c.id,operations:32,samples:copies});
  report.textContent=`틱·그림 측정 ${c.id}`;await new Promise(r=>setTimeout(r,0));
 }
 const image=new Uint8Array(await(await fetch('./adlib-image.bin')).arrayBuffer());
 for(const c of plan.sounds){
  const samples=[];
  for(let i=0;i<33;i++){
   const initStart=now();const engine=new PerformancePcm(image,c.music,new Uint8Array(c.effects));const initNs=ns(initStart);const chunks=[];
   const start=now();for(let left=44100;left;){const n=Math.min(512,left);chunks.push(engine.render(n));left-=n;}const time=ns(start);
   const pcm=new Int16Array(44100);let at=0;for(const chunk of chunks){pcm.set(chunk,at);at+=chunk.length;}const digest=await hash(pcm.buffer);
   assert(digest===c.sha256,c.id+' PCM comparison');samples.push([time,44100,digest,initNs]);engine.free();
  }
  rows.push({kind:'wasm_pcm',id:c.id,operations:44100,samples});report.textContent=`음향 측정 ${c.id}`;await new Promise(r=>setTimeout(r,0));
 }
 bank.free();window.__performanceResult={status:'PASS',rows,environment:{userAgent:navigator.userAgent,hardwareConcurrency:navigator.hardwareConcurrency,crossOriginIsolated,clock:'performance.now',wasmMemoryBytes:wasm.memory.buffer.byteLength,pcmMemoryBytes:pcmWasm.memory.buffer.byteLength,heap:performance.memory?{usedJSHeapSize:performance.memory.usedJSHeapSize,totalJSHeapSize:performance.memory.totalJSHeapSize,jsHeapSizeLimit:performance.memory.jsHeapSizeLimit}:null}};report.textContent='알고리즘 성능 측정 완료';
}catch(e){window.__performanceResult={status:'FAIL',error:String(e),stack:e.stack};document.querySelector('#status').textContent=String(e);}
