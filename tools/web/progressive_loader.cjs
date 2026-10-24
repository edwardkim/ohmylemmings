const fs=require('fs'),path=require('path'),zlib=require('zlib'),crypto=require('crypto'),assert=require('assert/strict');
(async()=>{
 const {ProgressiveHdLoader}=await import(path.resolve('apps/web/www/progressive-hd.js'));
 function crc(bytes){let c=0xffffffff;for(const v of bytes){c^=v;for(let i=0;i<8;i++)c=(c>>>1)^((c&1)?0xedb88320:0);}return (c^0xffffffff)>>>0;}
 const raw=[Buffer.alloc(64,1),Buffer.alloc(64,2)],wire=raw.map(b=>zlib.gzipSync(b));
 const pages=wire.map((b,i)=>{const sha=crypto.createHash('sha256').update(b).digest('hex');return {path:`web-hd/${sha}.hdp.gz`,download_bytes:b.length,sha256:sha,wire_crc32:crc(b),bytes:raw[i].length,crc32:crc(raw[i]),keys:[[0,i,0]]};});
 const info={schema_version:1,codec:'gzip',catalog_sha256:'a'.repeat(64),transport_sha256:'b'.repeat(64),logical_keys:2,actor_frames:2,glyphs:95,pages};
 let current=0,commits=0,broken=false,slow=false;const calls=[],events=[];
 const saved=new Map(),cache={match:async key=>saved.get(String(key))?.clone(),put:async(key,r)=>{saved.set(String(key),r.clone());},delete:async key=>saved.delete(String(key))};
 const storage={open:async()=>cache,keys:async()=>[],delete:async()=>true};
 const game={hd_download_plan:next=>JSON.stringify({keys:[[0,next?1:current]]}),load_hd:bytes=>{assert.equal(Buffer.from(bytes.subarray(0,8)).toString(),'LHDSET01');commits++;}};
 const fetcher=async(url,options={})=>{
  calls.push({url,priority:options.priority});
  if(url==='./HD-WEB.json')return new Response(JSON.stringify(info));
  const i=pages.findIndex(p=>url.endsWith(p.path));assert.notEqual(i,-1);
  if(slow&&i===1)await new Promise((resolve,reject)=>{options.signal.addEventListener('abort',()=>reject(new DOMException('aborted','AbortError')),{once:true});});
  const data=broken?Buffer.alloc(wire[i].length):wire[i];
  return new Response(new ReadableStream({start(c){const half=Math.floor(data.length/2);c.enqueue(data.subarray(0,half));c.enqueue(data.subarray(half));c.close();}}));
 };
 const loader=new ProgressiveHdLoader({fetcher,cacheStorage:storage,onProgress:event=>events.push(event)});
 await loader.load(game);assert.equal(commits,1);assert.equal(calls.filter(c=>c.url.endsWith('.gz')).length,1);
 const progress=events.filter(e=>e.phase==='foreground'&&e.total===wire[0].length&&e.status==='loading');
 assert(progress.some(e=>e.loaded>0&&e.loaded<e.total),'missing intermediate byte progress');
 assert.equal(events.at(-1).status,'ready');assert.equal(events.at(-1).loaded,wire[0].length);
 await loader.prefetch(game);assert.equal(commits,1,'background changed renderer');assert.equal(calls.at(-1).priority,'low');
 current=1;const before=calls.length;await loader.load(game);assert.equal(calls.length,before,'warm cache fetched again');assert.equal(events.at(-1).cached,wire[1].length);
 saved.clear();loader.memory.clear();current=0;broken=true;await assert.rejects(loader.load(game),/checksum|digest/);assert.equal(commits,2,'bad download committed');broken=false;
 await loader.load(game);assert.equal(commits,3,'retry failed');
 saved.clear();loader.memory.clear();slow=true;const pending=loader.prefetch(game);await new Promise(r=>setTimeout(r,10));current=0;
 await loader.load(game);await pending;assert.equal(commits,4,'foreground failed to preempt background');slow=false;
 const noStorage=new ProgressiveHdLoader({fetcher,cacheStorage:{open:async()=>{throw Error('quota');},keys:async()=>[]}});
 await noStorage.load(game);assert.equal(commits,5,'storage failure stopped play');
 const streaming=async(url,options={})=>{
  if(url==='./HD-WEB.json')return new Response(JSON.stringify(info));
  let cancelled=false;return new Response(new ReadableStream({async start(c){for(let at=0;at<wire[0].length&&!cancelled;at+=5){c.enqueue(wire[0].subarray(at,at+5));await new Promise(r=>setTimeout(r,40));}if(!cancelled)c.close();},cancel(){cancelled=true;}}));
 };
 const slowLoader=new ProgressiveHdLoader({fetcher:streaming,cacheStorage:null,idleTimeoutMs:80});
 await slowLoader.load(game);assert.equal(commits,6,'continuously slow transfer hit an absolute timeout');
 const stalled=new ProgressiveHdLoader({fetcher:async url=>url==='./HD-WEB.json'?new Response(JSON.stringify(info)):new Response(new ReadableStream({start(c){c.enqueue(wire[0].subarray(0,3));}})),cacheStorage:null,idleTimeoutMs:30});
 await assert.rejects(stalled.load(game),/timed out/);assert.equal(commits,6,'stalled resource committed');
 console.log(JSON.stringify({status:'PASS',initial_assets_only:true,byte_progress:true,cache_reuse:true,background_does_not_commit:true,foreground_preempts:true,corrupt_rejected:true,retry:true,storage_optional:true,continuous_slow_transfer:true,stalled_transfer_timeout:true}));
})().catch(e=>{console.error(e);process.exitCode=1;});
