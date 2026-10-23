// Review every actor frame and preview control using a disposable blank CDP tab.
const fs=require('fs'),path=require('path');
const [base,cdp,directory]=process.argv.slice(2);
if(!base||!cdp||!directory)throw Error('usage: node tools/hd_assets_check.cjs PREVIEW_URL DISPOSABLE_CDP_URL NEW_EVIDENCE_DIR');
const out=path.resolve(directory),allowed=path.resolve(__dirname,'../assets/local');
if(path.relative(allowed,out).startsWith('..')||fs.existsSync(out))throw Error('use a new evidence directory under assets/local');
fs.mkdirSync(out,{recursive:true});
(async()=>{
 let tabs;for(let n=0;!tabs;n++){try{tabs=await(await fetch(cdp+'/json/list')).json();}catch(error){if(n>=100)throw error;await new Promise(r=>setTimeout(r,100));}}
 const pages=tabs.filter(t=>t.type==='page');
 if(pages.length!==1||pages[0].url!=='about:blank')throw Error('use a disposable blank QA browser');
 const ws=new WebSocket(pages[0].webSocketDebuggerUrl);await new Promise(r=>ws.addEventListener('open',r,{once:true}));
 let id=0;const pending=new Map(),errors=[];
 ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.method==='Runtime.exceptionThrown')errors.push(m.params);if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(Error(JSON.stringify(m.error))):p.resolve(m.result);}};
 const cmd=(method,params={})=>new Promise((resolve,reject)=>{pending.set(++id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
 const evaluate=async expression=>{const r=await cmd('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));return r.result.value;};
 try {
  await cmd('Runtime.enable');await cmd('Page.enable');await cmd('Page.navigate',{url:base});
  for(let n=0;!(await evaluate("document.getElementById('status')?.textContent.includes('개 자산을 불러왔습니다')"));n++){
   if(n>300)throw Error('preview load timeout '+await evaluate('document.body.innerText'));
   await new Promise(r=>setTimeout(r,100));
  }
  const controls=await evaluate(`(()=>{const select=document.getElementById('animation'),slider=document.getElementById('frame');document.getElementById('play').click();let groups=0,endpoints=0;for(const option of select.options){select.value=option.value;select.dispatchEvent(new Event('change'));for(const index of [0,Number(slider.max)]){slider.value=String(index);slider.dispatchEvent(new Event('input'));if(Number(document.getElementById('number').value)!==index)throw Error('frame endpoint');endpoints++;}groups++;}for(const bg of ['checker','light','dark']){const s=document.getElementById('background');s.value=bg;s.dispatchEvent(new Event('change'));}return {groups,endpoints,icons:document.querySelectorAll('#icons figure').length,backgrounds:3};})()`);
  const result=await evaluate(`(async()=>{const m=await(await fetch('./manifest.json')).json();const images=await Promise.all(m.atlases.map(async a=>{const i=new Image();i.src=a.path;await i.decode();const c=document.createElement('canvas');c.width=i.width;c.height=i.height;const x=c.getContext('2d',{willReadFrequently:true});x.drawImage(i,0,0);const p=x.getImageData(0,0,c.width,c.height);for(let n=3;n<p.data.length;n+=4)if(p.data[n]<=a.alpha_cutoff)p.data[n]=0;x.putImageData(p,0,0);return c;}));const actors=m.frames.filter(f=>f.key.actor);if(actors.length!==337||new Set(actors.map(f=>f.key.actor[0])).size!==28)throw Error('actor coverage');const c=document.createElement('canvas');c.width=256;c.height=180;const x=c.getContext('2d',{willReadFrequently:true});for(const f of actors){x.clearRect(0,0,256,180);x.drawImage(images[f.atlas],...f.rect,128-f.anchor[0],150-f.anchor[1],...f.display);if(!x.getImageData(0,0,256,180).data.some((v,i)=>i%4===3&&v>0))throw Error('empty actor '+f.key.actor);}const names=['Walk R','Jump R','Walk L','Jump L','Dig','Climb R','Climb L','Drown','Hoist R','Hoist L','Build R','Build L','Bash R','Bash L','Mine R','Mine L','Fall R','Fall L','Umbrella R','Umbrella L','Splat','Exit','Vaporize','Block','Shrug R','Shrug L','Oh No','Explosion'];const representative=[0,0,0,0,8,4,4,6,4,4,6,6,4,4,5,5,1,1,4,4,12,4,8,5,4,4,8,0];const sheet=document.createElement('canvas');sheet.width=1200;sheet.height=1120;const g=sheet.getContext('2d');g.fillStyle='#142026';g.fillRect(0,0,1200,1120);for(let slot=0;slot<28;slot++){const f=actors.find(v=>v.key.actor[0]===slot&&v.key.actor[1]===representative[slot]);if(!f)throw Error('representative');const px=slot%4*300,py=Math.floor(slot/4)*160;g.fillStyle='#24383d';g.fillRect(px+4,py+4,292,152);g.fillStyle='#d4e8df';g.font='15px sans-serif';g.fillText(slot+' '+names[slot],px+14,py+25);g.strokeStyle='#52796a';g.beginPath();g.moveTo(px+40,py+148.5);g.lineTo(px+260,py+148.5);g.stroke();const scale=slot===27?Math.min(1,116/f.display[1],260/f.display[0]):1.25;g.save();g.beginPath();g.rect(px+4,py+30,292,126);g.clip();g.drawImage(images[f.atlas],...f.rect,px+150-(slot===27?f.display[0]/2:f.anchor[0])*scale,slot===27?py+32+(116-f.display[1]*scale)/2:py+148-f.anchor[1]*scale,f.display[0]*scale,f.display[1]*scale);g.restore();}document.body.replaceChildren(sheet);document.body.style.cssText='margin:0;background:#142026';return {status:'PASS',actor_animations:28,actor_frames:actors.length,canvas_readbacks:actors.length,atlas_count:m.atlases.length};})()`);
  await cmd('Emulation.setDeviceMetricsOverride',{width:1200,height:1120,deviceScaleFactor:1,mobile:false});
  const screenshot=await cmd('Page.captureScreenshot',{format:'png',clip:{x:0,y:0,width:1200,height:1120,scale:1}});fs.writeFileSync(out+'/actors.png',Buffer.from(screenshot.data,'base64'));
  if(errors.length)throw Error('console exceptions '+JSON.stringify(errors));
  const report={...result,controls,console_errors:errors.length,scope:'all actor PNG/Canvas frames and preview UI; gameplay equivalence tracked separately'};
  fs.writeFileSync(out+'/summary.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }finally{await cmd('Browser.close').catch(()=>{});ws.close();}
})().catch(error=>{console.error(error.stack);process.exit(1);});
