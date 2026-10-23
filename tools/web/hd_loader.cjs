const fs=require('fs'),assert=require('assert/strict');
(async()=>{
 const source=fs.readFileSync('apps/web/www/hd-loader.js','utf8');const {HdLoader,catalogEntries}=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
 function crc(b){let c=0xffffffff;for(const v of b){c^=v;for(let n=0;n<8;n++)c=(c>>>1)^((c&1)?0xedb88320:0);}return (c^0xffffffff)>>>0;}
 const parts=[1,2,3].map(n=>new Uint8Array(64).fill(n)),catalog=new Uint8Array(48),view=new DataView(catalog.buffer);catalog.set(new TextEncoder().encode('LHDIDX01'));view.setUint16(8,3,true);
 for(let i=0;i<3;i++){view.setUint16(12+i*12,1<<i,true);view.setUint32(16+i*12,64,true);view.setUint32(20+i*12,crc(parts[i]),true);}
 assert.deepEqual(catalogEntries(catalog,3).map(r=>r.index),[0,1]);for(const scope of [0,1,7,1024,195])assert.throws(()=>catalogEntries(catalog,scope));
 let scope=3,commits=0,fail=false;const calls=[];const game={hd_scope:()=>scope,load_hd:b=>{assert.equal(new TextDecoder().decode(b.subarray(0,8)),'LHDSET01');commits++;}};
 const loader=new HdLoader(async url=>{calls.push(url);if(url.endsWith('catalog.hdi'))return new Response(catalog);const i=Number(url.match(/atlas-(\d+)/)[1]);return new Response(fail&&i===2?new Uint8Array(64):parts[i]);});
 await loader.load(game);assert.equal(commits,1);assert.deepEqual([...loader.cache.keys()],[0,1]);
 scope=5;fail=true;await assert.rejects(loader.load(game),/checksum/);assert.equal(commits,1);assert.deepEqual([...loader.cache.keys()],[0,1]);
 fail=false;await loader.load(game);assert.equal(commits,2);assert.deepEqual([...loader.cache.keys()],[0,2]);assert.equal(calls.filter(s=>s.endsWith('atlas-000.hdp')).length,1);
 scope=3;game.load_hd=()=>{throw Error('invalid pack');};await assert.rejects(loader.load(game),/invalid pack/);assert.deepEqual([...loader.cache.keys()],[0,2]);
 const legacy=new HdLoader(async url=>url.endsWith('catalog.hdi')?new Response('',{status:404}):new Response(parts[0]));let legacyCalls=0;await legacy.load({load_hd:b=>{assert.deepEqual(b,parts[0]);legacyCalls++;}});assert.equal(legacyCalls,1);
 const broken=new HdLoader(async()=>new Response('',{status:500}));await assert.rejects(broken.load(game),/download failed/);
 console.log(JSON.stringify({status:'PASS',scope_selection:true,common_reused:true,old_theme_evicted:true,failed_load_atomic:true,legacy_404_only:true}));
})().catch(e=>{console.error(e);process.exit(1);});
