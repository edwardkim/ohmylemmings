import assert from 'node:assert/strict';
import fs from 'node:fs';
const source = name => fs.readFileSync(`apps/web/www/${name}.js`, 'utf8');
const url = text => 'data:text/javascript;base64,' + Buffer.from(text).toString('base64');
const gpuUrl = url(source('webgpu'));
const { WebRenderer } = await import(url(source('renderer').replace('./host.js',url(source('host'))).replace('./webgpu.js',gpuUrl)));
const { WebGpu, compileCommands, premultiply } = await import(gpuUrl);

function canvas() {
    return { width:320,height:200,style:{opacity:''},dataset:{},className:'',
        getContext:()=>({}),setAttribute(){},remove(){this.removed=true;},
        ownerDocument:{createElement:()=>canvas()},after(c){this.sibling=c;} };
}
const base={secure:true,gpu:{},timeoutMs:10};
let r=await WebRenderer.create(canvas(),{...base,gpu:null});
assert.equal(r.kind,'canvas2d');assert.match(r.reason,/unavailable/);
r=await WebRenderer.create(canvas(),{...base,secure:false});
assert.equal(r.kind,'canvas2d');assert.match(r.reason,/secure/);
let attempts=0;
r=await WebRenderer.create(canvas(),{...base,preference:'canvas2d',createGpu:()=>{attempts++;}});
assert.equal(attempts,0);assert.equal(r.kind,'canvas2d');
r=await WebRenderer.create(canvas(),{...base,createGpu:async()=>{throw Error('adapter failed');}});
assert.equal(r.kind,'canvas2d');assert.match(r.reason,/adapter failed/);
r=await WebRenderer.create(canvas(),{...base,createGpu:async(c,g,onLost)=>{onLost('lost during initialization');return {dispose(){}};}});
assert.equal(r.kind,'canvas2d');assert.match(r.reason,/lost during initialization/);
await assert.rejects(WebGpu.create(canvas(),{requestAdapter:async()=>null},()=>{}),/adapter/);
let late,disposed=0;
r=await WebRenderer.create(canvas(),{...base,createGpu:()=>new Promise(resolve=>late=resolve)});
assert.equal(r.kind,'canvas2d');assert.match(r.reason,/timed out/);
late({dispose(){disposed++;}});await new Promise(resolve=>setTimeout(resolve,0));assert.equal(disposed,1);
let lost,draws=0,changes=0,clears=0;
const c=canvas();
r=await WebRenderer.create(c,{...base,onChange:()=>changes++,createGpu:async(surface,gpu,onLost)=>{
    lost=onLost;return {drawHd(){draws++;},drawOriginal(){throw Error('draw failed');},
        invalidate(){clears++;},dispose(){disposed++;}};
}});
assert.equal(r.kind,'webgpu');assert.equal(c.style.opacity,'0');
assert.equal(r.surface,c.sibling);assert.equal(r.surface.width,320);
r.resize(1920,1080);assert.equal(r.surface.width,1920);assert.equal(r.surface.height,1080);
r.invalidate();assert.equal(clears,1);
r.draw({},true,{});assert.equal(draws,1);
lost('device lost');assert.equal(r.kind,'canvas2d');assert.equal(c.style.opacity,'');
assert.equal(r.surface,c);assert.equal(changes,1);
lost('duplicate');assert.equal(changes,1);
const rgbGame={frame:()=>new Uint8Array(3),width:()=>1,height:()=>1};
const cpu=canvas();cpu.getContext=()=>({createImageData:()=>({data:new Uint8Array(4)}),putImageData(){draws++;}});
r=await WebRenderer.create(cpu,{...base,createGpu:async()=>({drawOriginal(){throw Error('draw failed');},dispose(){}})});
r.draw(rgbGame,false,null);assert.equal(r.kind,'canvas2d');assert.equal(draws,2);

const texture={width:4,height:4};
const commands=new Int32Array([
    0,0,0,1920,1080,10,20,30,255,0,0,0,
    2,1,0,0,0,0,0,0,0,0,0,0,
    1,7,1,1,2,2,-6,12,18,24,0,0,
    2,0,0,0,0,0,0,0,0,0,0,0,
    0,2,3,4,5,100,50,25,128,0,0,0,
]);
const compiled=compileCommands(commands,1920,1080,()=>texture);
assert.deepEqual(compiled.draws.map(d=>d.id),[null,7,null]);
assert.deepEqual(compiled.draws.map(d=>d.clip),[1080,960,1080]);
assert.equal(compiled.vertices.length,3*6*8);
const v=compiled.vertices;assert.ok(Math.abs(v[0]+1)<1e-6);assert.equal(v[1],1);
assert.equal(v[6*8+2],.25);assert.equal(v[6*8+3],.25);
assert.ok(Math.abs(v[12*8+4]-(100/255)*(128/255))<1e-6);
assert.deepEqual([...premultiply(new Uint8Array([200,100,50,128,250,60,30,0]))],[100,50,25,128,0,0,0,0]);
assert.throws(()=>compileCommands(new Int32Array(1),1,1,()=>texture),/length/);
assert.throws(()=>compileCommands(new Int32Array([9,...Array(11).fill(0)]),1,1,()=>texture),/command/);
globalThis.GPUTextureUsage={TEXTURE_BINDING:4,COPY_DST:2};
let uploads=0,destroyed=0,reads=0,revision=1;
const device={limits:{maxTextureDimension2D:8192},createSampler:()=>({}),
    createTexture:()=>({createView:()=>({}),destroy:()=>destroyed++}),
    createBindGroup:()=>({}),queue:{writeTexture:()=>uploads++},destroy(){}};
const gpuHost=new WebGpu(canvas(),{unconfigure(){}},device);
gpuHost.pipeline={getBindGroupLayout:()=>({})};
const texGame={hd_texture_revision:()=>revision,hd_texture_width:()=>1,hd_texture_height:()=>1,
    hd_texture:()=>{reads++;return new Uint8Array([100,50,25,128]);}};
gpuHost.texture(texGame,7);gpuHost.texture(texGame,7);
assert.equal(uploads,1);assert.equal(reads,1);
revision++;gpuHost.texture(texGame,7);assert.equal(uploads,2);assert.equal(destroyed,1);
gpuHost.invalidate();assert.equal(gpuHost.cache.size,0);assert.equal(destroyed,2);
gpuHost.texture(texGame,7);gpuHost.dispose();gpuHost.dispose();assert.equal(destroyed,3);
console.log('PASS: selection, timeout cleanup, device loss, synchronous recovery, ordered quads, clip, premultiplied alpha');
