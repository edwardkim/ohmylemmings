// Real production statistics module, without a browser or original assets.
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const source=fs.readFileSync('apps/web/www/frame-feedback.js','utf8');
 const {FrameSamples,feedbackText}=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
 const s=new FrameSamples();
 assert.equal(s.summary().fps,null);
 s.frame(0);for(let i=1;i<=60;i++)s.frame(i*1000/60, i%4===0?1:0);
 let r=s.summary();assert.ok(Math.abs(r.fps-60)<1e-9);assert.ok(Math.abs(r.draws_per_second-15)<1e-9);
 assert.equal(r.intervals,60);assert.equal(r.slow_intervals,0);assert.ok(Math.abs(r.p95_ms-1000/60)<1e-9);
 s.hold();s.frame(100000);s.frame(100050);assert.equal(s.summary().slow_intervals,0);
 s.frame(100101);assert.equal(s.summary().slow_intervals,1);
 for(let i=0;i<1000;i++)s.frame(100118+i*17);
 r=s.summary();assert.equal(r.recent_intervals,240);assert.equal(r.intervals,1062);assert.equal(r.max_ms,17);assert.equal(r.slow_intervals,1);
 s.reset();s.frame(1);s.frame(11);assert.equal(s.summary().intervals,1);assert.equal(s.summary().fps,100);
 assert.throws(()=>s.frame(NaN));assert.throws(()=>s.frame(0));
 const text=feedbackText({version:'0.3.1',platform:'Web',level:'Fun 01',presentation:'HD',viewport:'960x540',runtime:'test browser',musicFile:'/private/name.mp3'},s.summary());
 assert.ok(text.includes('Fun 01')&&text.includes('0.3.1')&&text.includes('rAF'));
 assert.ok(!text.includes('/private')&&!text.includes('musicFile'));assert.ok(text.includes('체감'));
 console.log('PASS: rates, fixed buffer, holds, reset, long gaps, feedback allowlist');
})().catch(error=>{console.error(error);process.exitCode=1;});
