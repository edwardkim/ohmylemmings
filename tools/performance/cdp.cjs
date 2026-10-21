// Existing local Chromium CDP session; no browser install or remote policy changes.
const fs=require('fs');const path=require('path');
const wait=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
 const [mode,output,reuse,repeats='3']=process.argv.slice(2);if(!['algorithm','live','lifecycle'].includes(mode)||!output||!['1','3'].includes(repeats))throw Error('cdp.cjs algorithm|live|lifecycle NEW_OUTPUT_DIR [reuse|new] [1|3]');if(reuse!=="reuse")fs.mkdirSync(output);
 const url='http://127.0.0.1:8080/performance/'+(mode!=='algorithm'?'live.html':'algorithm.html');
 const target=reuse==='reuse'?JSON.parse(fs.readFileSync(path.join(output,'tab.json'))):await(await fetch('http://127.0.0.1:19222/json/new?'+encodeURIComponent(url),{method:'PUT'})).json();
 fs.writeFileSync(path.join(output,'tab.json'),JSON.stringify(target));const ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise(r=>ws.addEventListener('open',r,{once:true}));let id=0;const pending=new Map();
 ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(Error(JSON.stringify(m.error))):p.resolve(m.result);}};
 const command=(method,params={})=>new Promise((resolve,reject)=>{pending.set(++id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
 ws.onclose=()=>{for(const p of pending.values())p.reject(Error('CDP target closed'));pending.clear();};
 const evaluate=async(expression,extra={})=>{const r=await command('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true,...extra});if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));return r.result.value;};
 await command('Page.bringToFront');
 if(mode==='algorithm'){
  for(let n=0;n<2400;n++){const r=await evaluate('window.__performanceResult');if(r){fs.writeFileSync(path.join(output,'result.json'),JSON.stringify(r));if(r.status!=='PASS')throw Error(JSON.stringify(r));console.log('PASS algorithm',r.rows.length,'rows');break;}if(n%30===0)console.log('algorithm poll',n);await wait(1000);if(n===2399)throw Error('algorithm deadline');}
 }else{
  if(mode==='live'){
  for(let n=0;!(await evaluate('Boolean(window.__perfLive)'));n++){if(n>60)throw Error('live page not ready');await wait(250);}
  await evaluate("document.querySelector('#difficulty').value='mayhem';document.querySelector('#level').value='22';");
  const d=await command('DOM.getDocument');const selector=await command('DOM.querySelector',{nodeId:d.root.nodeId,selector:'#files'});
  const plan=JSON.parse(fs.readFileSync('assets/local/web/performance/plan.json'));const files=plan.files.map(n=>path.resolve('assets/local/dos-original',n));
  await command('DOM.setFileInputFiles',{nodeId:selector.nodeId,files});
  for(let n=0;!(await evaluate('window.__perfLive.state.original'));n++){if(n>120)throw Error('original load deadline');await wait(250);}
  await evaluate("document.querySelector('#sound').scrollIntoView({block:'center'})");
  const box=await evaluate("(()=>{const r=document.querySelector('#sound').getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}})()");
  for(const type of ['mousePressed','mouseReleased'])await command('Input.dispatchMouseEvent',{type,...box,button:'left',clickCount:1});
  for(let n=0;!(await evaluate('window.__perfLive.ready'));n++){if(n>40)throw Error('audio activation deadline');await wait(250);}
  await evaluate("document.querySelector('#game').scrollIntoView({block:'center'})");
  const c=plan.cases.find(c=>c.id==='mayhem-22');
  for(let repeat=1;repeat<=Number(repeats);repeat++){
   console.log('live',repeat,'start');await command('Page.bringToFront');
   const result=await evaluate('window.__perfLive.run('+JSON.stringify({...c,seconds:60})+')',{awaitPromise:true,userGesture:true});
   fs.writeFileSync(path.join(output,'run-'+repeat+'.json'),JSON.stringify(result));
   if(result.visibility.length||result.audio.state!=='running')throw Error('visible interval interrupted; raw run retained');
   console.log('live',repeat,'PASS',result.ticks.length,'ticks',result.frames.length,'frames');await wait(1500);
  }
  }
  // Exercise the application's real hidden/resume handler separately from timed intervals.
  const before=await evaluate('window.__perfLive.visibilityProbe()');const blank=await command('Target.createTarget',{url:'about:blank'});await command('Target.activateTarget',{targetId:blank.targetId});await wait(100);const enter=await evaluate('window.__perfLive.visibilityProbe()');await wait(1000);
  const hidden=await evaluate('window.__perfLive.visibilityProbe()');await command('Target.activateTarget',{targetId:target.id});await wait(150);const after=await evaluate('window.__perfLive.visibilityProbe()');await command('Target.closeTarget',{targetId:blank.targetId});
  fs.writeFileSync(path.join(output,'lifecycle.json'),JSON.stringify({status:'OBSERVED',before,enter,hidden,after}));
  if(!hidden.hidden||hidden.audioState!=='suspended'||hidden.sources!==0||after.hidden||after.audioState!=='running')throw Error('hidden/resume device policy; observations retained');
  const a=JSON.parse(enter.state),b=JSON.parse(hidden.state),resumed=JSON.parse(after.state);if(b.session.control_step!==a.session.control_step||resumed.session.control_step-b.session.control_step>4)throw Error('hidden time admitted');
  fs.writeFileSync(path.join(output,'lifecycle.json'),JSON.stringify({status:'PASS',before,enter,hidden,after,method:'real tab visibility/audio hold; paused control counter proves hidden wall time excluded'}));
  await evaluate("document.querySelector('#sound').click()");
 }
 const metrics=await command('Performance.getMetrics');fs.writeFileSync(path.join(output,'metrics.json'),JSON.stringify(metrics));ws.close();
})().catch(e=>{console.error(e.stack);process.exit(1);});
