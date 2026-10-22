// Observe the normal application; output serialization/hashing stays outside the interval.
import {observe} from './app.js';
import init,{WebGame} from './pkg/lemmings_web.js';
import {AudioHost} from './audio.js';
import {Series,timeMethod} from './measure.js';
import {DispatchEvents} from './dispatch_events.js';
const wasm=await init();
let game,config=null,data=null,step=0,start=0,tickStart=0,lastPresent=null,host=null,current=null;
const clock=()=>performance.now();
const sha=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
const costs=new Float64Array(6);
// Nested calls are exclusive categories here: step/frame/state/putImageData do not call service.
function instrument(proto,name,index){
 timeMethod(proto,name,clock,elapsed=>{if(current){const begin=clock();costs[index]+=elapsed;costs[5]+=clock()-begin;}});
}
instrument(WebGame.prototype,'frame',0);instrument(WebGame.prototype,'state',1);
instrument(CanvasRenderingContext2D.prototype,'putImageData',3);instrument(WebGame.prototype,'step',4);
const originalService=AudioHost.prototype.service;
AudioHost.prototype.service=function(){
 host=this;const begin=clock();originalService.call(this);const end=clock();
 if(current)costs[2]+=end-begin;
 if(data&&this.context){const t=clock();data.audio.push(t-start,this.nextTime?this.nextTime-this.context.currentTime:0,this.sources.size,this.underruns,end-begin,this.context.state==='running'?1:0);if(current)costs[5]+=clock()-t;}
};
const raf=window.requestAnimationFrame.bind(window);
window.requestAnimationFrame=callback=>raf(timestamp=>{
 const begin=clock();const captured=data;
 if(captured){costs.fill(0);current=captured;}
 try{callback(timestamp);}
 finally{
  const end=clock();current=null;
  if(captured&&data===captured)captured.callbacks.push(timestamp-start,begin-start,end-start,costs[0],costs[1],costs[2],costs[3],costs[4],costs[5],step);
 }
});
observe((event,g,detail)=>{
 const observed=clock();game=g;
 if(event==='reset'){
  step=0;
  if(config){
   const capacity=Math.ceil(config.seconds*240)+1000;
   data={ticks:new Series(capacity,3),frames:new Series(capacity,2),audio:new Series(capacity*3,6),callbacks:new Series(capacity,10),visibility:[],startState:JSON.parse(g.state()),startSound:JSON.parse(g.sound_state()),initialAudioUnderruns:host?.underruns||0,wasmStart:wasm.memory.buffer.byteLength};
   data.diagnostic=config.diagnostic?new DispatchEvents(capacity):null;
   start=clock();lastPresent=null;
  }
 }
 if(data?.diagnostic)data.diagnostic.record(event,detail,clock()-start,step,Boolean(current));
 if(event==='before'&&config){for(const c of config.commands)if(c[0]===step)g.command(c[2],c[3]);tickStart=clock();}
 if(event==='after'){step++;if(data)data.ticks.push(clock()-start,step,clock()-tickStart);}
 if(event==='present'&&data){const at=clock()-start;data.frames.push(at,lastPresent===null?0:at-lastPresent);lastPresent=at;}
 if(current)costs[5]+=clock()-observed;
});
const visibilityEvents=[];
document.addEventListener('visibilitychange',()=>{visibilityEvents.push([clock(),document.hidden]);if(data)data.visibility.push([clock()-start,document.hidden]);});
window.__perfLive={
 get ready(){return Boolean(game?.original()&&host?.context?.state==='running');},
 get state(){return {original:game?.original(),level:game?.level(),step,audio:host?.context?.state,hidden:document.hidden,measuring:Boolean(config),at_ms:config?clock()-start:null};},
 async run(c){
  if(config)throw Error('measurement already running');if(!game?.original()||host?.context?.state!=='running'||document.hidden)throw Error('visible original game with active sound required');
  config=c;const [rank,n]=c.id.split('-');document.querySelector('#difficulty').value=rank;document.querySelector('#level').value=Number(n);document.querySelector('#open-level').click();
  if(!data)throw Error('reset not observed');
  return new Promise(resolve=>setTimeout(async()=>{
   // Freeze observations and end-memory before endpoint diagnostics allocate anything.
   const seconds=(clock()-start)/1000,wasmEnd=wasm.memory.buffer.byteLength,recorded=data;
   config=null;data=null;
   const finalStateRaw=game.state();const outputs=[new TextEncoder().encode(finalStateRaw),game.terrain(),game.bricks(),game.objects(),game.frame()];
   const frameRows=recorded.callbacks.rows();
   const result={status:'OBSERVED',id:c.id,seconds,finalStateRaw,ticks:recorded.ticks.rows(),frames:recorded.frames.rows(),audioSamples:recorded.audio.rows(),callbacks:frameRows.map(r=>[r[1],r[2]-r[1]]),callbackDetails:frameRows,callbackColumns:['raf_timestamp_ms','begin_ms','end_ms','rgb_ms','state_ms','audio_ms','put_image_ms','step_ms','observer_bookkeeping_ms','control_step'],visibility:recorded.visibility,startState:recorded.startState,startSound:recorded.startSound,initialAudioUnderruns:recorded.initialAudioUnderruns,wasmStart:recorded.wasmStart,wasmEnd,recorderReservedBytes:recorded.ticks.bytes+recorded.frames.bytes+recorded.audio.bytes+recorded.callbacks.bytes,finalState:JSON.parse(finalStateRaw),finalSound:JSON.parse(game.sound_state()),audio:{state:host.context.state,rate:host.context.sampleRate,baseLatency:host.context.baseLatency,outputLatency:host.context.outputLatency,submitted:host.submitted,underruns:host.underruns,sources:host.sources.size,scheduledChunk:512,hardwareQuantum:'not exposed by AudioBufferSource API'},environment:{userAgent:navigator.userAgent,width:320,height:200,devicePixelRatio,viewport:[innerWidth,innerHeight],canvasCss:[document.querySelector("#game").getBoundingClientRect().width,document.querySelector("#game").getBoundingClientRect().height],heap:performance.memory?{usedJSHeapSize:performance.memory.usedJSHeapSize,totalJSHeapSize:performance.memory.totalJSHeapSize,jsHeapSizeLimit:performance.memory.jsHeapSizeLimit}:null}};
   result.clockStart=start;result.clockOrigin=performance.timeOrigin;
   result.diagnostic=recorded.diagnostic?.result()||null;
   result.recorderReservedBytes+=recorded.diagnostic?.bytes||0;
   game.command('Pause',0);
   try{resolve({...result,outputHashes:await Promise.all(outputs.map(sha))});}catch(error){resolve({...result,status:'FAIL',error:String(error)});}
  },c.seconds*1000));
 },
 async visibilityProbe(){return {state:game.state(),sound:game.sound_state(),hidden:document.hidden,audioState:host?.context?.state,sources:host?.sources.size,visibilityEvents:[...visibilityEvents]};}
};
