const assert=require('node:assert/strict');
(async()=>{
 const {LocalMusic}=await import('../../apps/web/www/local-music.js');
 const media=[];const revoked=[];let fail=false,resolvePlay;
 const context={destination:{},createMediaElementSource(){return {connect(){},disconnect(){}};},createGain(){return {gain:{value:0},connect(){},disconnect(){}};}};
 const createMedia=()=>{const listeners={};const m={currentTime:0,paused:true,readyState:0,addEventListener(k,f){listeners[k]=f;},emit(k){listeners[k]?.();},load(){},removeAttribute(){},pause(){this.paused=true;},play(){this.paused=false;return fail?Promise.reject(new Error('blocked')):new Promise(r=>{resolvePlay=r;});}};media.push(m);return m;};
 const states=[];const p=new LocalMusic(()=>context,s=>states.push(s),{createMedia,createURL:()=>`blob:${media.length}`,revokeURL:u=>revoked.push(u)});
 assert.equal(media.length,0); // no autoplay or file access before selection
 p.choose({name:'私の曲.MP3',size:4000}); assert.equal(p.state,'loading');
 const first=media[0];first.readyState=3;first.emit('canplay');assert.equal(p.state,'ready');
 p.sync(true,0,true);assert.equal(first.loop,true);assert.equal(first.paused,false);
 p.sync(false,0,true);resolvePlay();await Promise.resolve();await Promise.resolve();assert.equal(first.paused,true);assert.notEqual(p.state,'playing');
 fail=true;p.sync(true,0,true);await Promise.resolve();await Promise.resolve();assert.equal(p.state,'blocked');
 fail=false;p.retry();resolvePlay();await Promise.resolve();await Promise.resolve();assert.equal(p.state,'playing');
 p.setVolume(17);assert.equal(p.gain.gain.value,0.17);
 p.sync(true,1,false);assert.equal(first.currentTime,0);assert.equal(first.paused,true);
 assert.throws(()=>p.choose({name:'bad.exe',size:8}));assert.equal(media.length,1);
 p.choose({name:'next.mp3',size:100});assert.equal(first.paused,true);assert.equal(revoked.length,1);first.emit('error');assert.equal(p.state,'loading');
 const second=media[1];second.emit('error');assert.equal(p.state,'error');
 p.dispose();assert.equal(revoked.length,2);assert.equal(p.media,null);
 console.log('PASS local MP3 lifecycle, blocked/retry, generation, independent gain, stale events, cleanup, no upload');
})().catch(e=>{console.error(e);process.exitCode=1;});
