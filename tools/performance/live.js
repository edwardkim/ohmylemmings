// Attach to the normal application; simulation, drawing, UI and sound stay on its production path.
import {observe} from './app.js';
import {AudioHost} from './audio.js';
let game,config=null,data=null,step=0,start=0,tickStart=0,lastPresent=null,host=null,resolveRun;
const clock=()=>performance.now();
const sha=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
const originalService=AudioHost.prototype.service;
AudioHost.prototype.service=function(){host=this;const t=clock();const before=this.context?{lead:this.nextTime?this.nextTime-this.context.currentTime:null,state:this.context.state,rate:this.context.sampleRate,sources:this.sources.size}:null;originalService.call(this);if(data&&before)data.audioSamples.push([clock()-start,before.lead,before.sources,this.underruns,clock()-t,before.state]);};
const raf=window.requestAnimationFrame.bind(window);
window.requestAnimationFrame=callback=>raf(time=>{const begin=clock();callback(time);if(data)data.callbacks.push([begin-start,clock()-begin]);});
observe((event,g)=>{
 game=g;
 if(event==='reset'){step=0;if(config){start=clock();lastPresent=null;data={ticks:[],frames:[],audioSamples:[],callbacks:[],visibility:[],startState:JSON.parse(g.state()),startSound:JSON.parse(g.sound_state()),initialAudioUnderruns:host?.underruns||0};}}
 if(event==='before'&&config){for(const c of config.commands)if(c[0]===step)g.command(c[2],c[3]);tickStart=clock();}
 if(event==='after'){step++;if(data)data.ticks.push([clock()-start,step,clock()-tickStart]);}
 if(event==='present'&&data){const at=clock()-start;data.frames.push([at,lastPresent===null?0:at-lastPresent]);lastPresent=at;}
});
const visibilityEvents=[];
document.addEventListener('visibilitychange',()=>{visibilityEvents.push([clock(),document.hidden]);if(data)data.visibility.push([clock()-start,document.hidden]);});
window.__perfLive={
 get ready(){return Boolean(game?.original()&&host?.context?.state==='running');},
 get state(){return {original:game?.original(),level:game?.level(),step,audio:host?.context?.state,hidden:document.hidden};},
 async run(c){
  if(config)throw Error('measurement already running');if(!game?.original()||host?.context?.state!=='running'||document.hidden)throw Error('visible original game with active sound required');
  config=c;const [rank,n]=c.id.split('-');document.querySelector('#difficulty').value=rank;document.querySelector('#level').value=Number(n);document.querySelector('#open-level').click();
  if(!data)throw Error('reset not observed');
  return new Promise(resolve=>{resolveRun=resolve;setTimeout(()=>{
   const finalStateRaw=game.state();const outputs=[new TextEncoder().encode(finalStateRaw),game.terrain(),game.bricks(),game.objects(),game.frame()];
   const result={status:'PASS',id:c.id,seconds:(clock()-start)/1000,finalStateRaw,...data,finalState:JSON.parse(game.state()),finalSound:JSON.parse(game.sound_state()),audio:{state:host.context.state,rate:host.context.sampleRate,baseLatency:host.context.baseLatency,outputLatency:host.context.outputLatency,submitted:host.submitted,underruns:host.underruns,sources:host.sources.size,scheduledChunk:512,hardwareQuantum:'not exposed by AudioBufferSource API'},environment:{userAgent:navigator.userAgent,width:320,height:200,devicePixelRatio,heap:performance.memory?{usedJSHeapSize:performance.memory.usedJSHeapSize,totalJSHeapSize:performance.memory.totalJSHeapSize,jsHeapSizeLimit:performance.memory.jsHeapSizeLimit}:null}};
   config=null;data=null;game.command('Pause',0);Promise.all(outputs.map(sha)).then(hashes=>resolveRun({...result,outputHashes:hashes}),error=>resolveRun({status:'FAIL',error:String(error)}));
  },c.seconds*1000);});
 },
 async visibilityProbe(){const before=game.state();return {state:before,sound:game.sound_state(),hidden:document.hidden,audioState:host?.context?.state,sources:host?.sources.size,visibilityEvents:[...visibilityEvents]};}
};
