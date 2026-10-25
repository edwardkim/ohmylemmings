import init, { WebGame } from './pkg/lemmings_web.js';
import { HdCanvas } from './hd.js';
import { WebRenderer } from './renderer.js';

const assert = (value, message) => {if (!value) throw Error(message);};
const createCanvas = (width, height) => {
    const c = document.createElement('canvas'); c.width = width; c.height = height;
    document.body.append(c); return c;
};
const read = canvas => {
    const c = createCanvas(canvas.width,canvas.height),ctx = c.getContext('2d',{willReadFrequently:true});
    ctx.drawImage(canvas,0,0); const pixels = ctx.getImageData(0,0,c.width,c.height).data; c.remove();return pixels;
};
function enableReadback(renderer) {
    renderer.backend.context.configure({device:renderer.backend.device,format:navigator.gpu.getPreferredCanvasFormat(),
        alphaMode:'opaque',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
}
async function readGpu(renderer) {
    const {device,context}=renderer.backend,w=renderer.surface.width,h=renderer.surface.height;
    const stride=Math.ceil(w*4/256)*256;
    const buffer=device.createBuffer({size:stride*h,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
    const encoder=device.createCommandEncoder();
    encoder.copyTextureToBuffer({texture:context.getCurrentTexture()},{buffer,bytesPerRow:stride},[w,h]);
    device.queue.submit([encoder.finish()]);
    await buffer.mapAsync(GPUMapMode.READ);
    const raw=new Uint8Array(buffer.getMappedRange()),pixels=new Uint8Array(w*h*4);
    const bgra=navigator.gpu.getPreferredCanvasFormat().startsWith('bgra');
    for(let y=0;y<h;y++)for(let x=0;x<w;x++) {
        const s=y*stride+x*4,d=(y*w+x)*4;
        pixels[d]=raw[s+(bgra?2:0)];pixels[d+1]=raw[s+1];pixels[d+2]=raw[s+(bgra?0:2)];pixels[d+3]=raw[s+3];
    }
    buffer.unmap();buffer.destroy();return pixels;
}
const compare = (a,b,tolerance,label) => {
    assert(a.length===b.length,`${label}: dimensions`);
    let max=0,over=0,total=0;
    for(let i=0;i<a.length;i++){const delta=Math.abs(a[i]-b[i]);max=Math.max(max,delta);total+=delta;if(delta>tolerance)over++;}
    return {label,max,over,mean:total/a.length,first_gpu:[...a.slice(0,4)],first_cpu:[...b.slice(0,4)]};
};
function assertFixture(a,b) {
    for(let i=0;i<a.length;i++) {
        const x=Math.floor(i/4)%1920,y=Math.floor(i/4/1920);
        // Skia's low-quality bilinear weights differ from WebGPU's sampler.
        // Geometry, alpha fills, unscaled texels and clipping remain strict.
        const filtered=(x>=100&&x<116&&y>=60&&y<76)||(x<10&&y>=90&&y<100);
        assert(Math.abs(a[i]-b[i])<=(filtered?16:2),`fixture pixel ${x},${y},${i%4}: ${a[i]} != ${b[i]}`);
    }
}
let result;
try {
    const cpu = createCanvas(1920,1080),c = createCanvas(1920,1080);
    const renderer = await WebRenderer.create(c);
    assert(renderer.kind==='webgpu',`WebGPU required for this check: ${renderer.reason}`);
    enableReadback(renderer);
    const adapter = await navigator.gpu.requestAdapter();
    const info = adapter.info;
    const comparisons = [];
    // Deliberately overlap translucent and transparent samples, crop, scale,
    // negative destinations, and clip across the 960px panel boundary.
    let revision=1,reads=0;
    const pixels = new Uint8Array([255,0,0,255,0,255,0,128,0,0,255,0,255,255,255,255]);
    const commands = new Int32Array([
        0,0,0,1920,1080,20,40,60,255,0,0,0,
        0,10,10,30,30,255,100,50,128,0,0,0,
        1,7,0,0,2,2,50,50,2,2,0,0,
        1,7,0,0,2,2,100,60,16,16,0,0,
        1,7,0,0,1,1,-5,90,15,10,0,0,
        2,1,0,0,0,0,0,0,0,0,0,0,
        0,0,950,40,40,255,255,0,255,0,0,0,
        2,0,0,0,0,0,0,0,0,0,0,0,
        0,0,1000,40,40,0,255,255,255,0,0,0,
    ]);
    const fixture={hd_commands:()=>commands,hd_texture_revision:()=>revision,
        hd_texture_width:()=>2,hd_texture_height:()=>2,hd_texture:()=>{reads++;return pixels;}};
    const hd = new HdCanvas(fixture),ctx = cpu.getContext('2d',{willReadFrequently:true});
    hd.draw(ctx); renderer.draw(fixture,true,hd);
    let actual=await readGpu(renderer),expected=read(cpu);
    comparisons.push(compare(actual,expected,2,'synthetic blend/crop/clip'));
    assertFixture(actual,expected);
    const before=reads;renderer.draw(fixture,true,hd);assert(reads===before,'unchanged GPU texture uploaded again');
    revision++;pixels[0]=50;hd.draw(ctx);renderer.draw(fixture,true,hd);
    assert(reads===before+2,'changed revision did not refresh both hosts');
    actual=await readGpu(renderer);expected=read(cpu);
    comparisons.push(compare(actual,expected,2,'revision refresh'));
    assertFixture(actual,expected);
    renderer.invalidate();
    await init();
    const game = new WebGame('builder','reference');
    const pack=await(await fetch('./hd-assets.hdp')).arrayBuffer();game.load_hd(new Uint8Array(pack));
    const host=new HdCanvas(game);
    for(const [scene,skill,ticks] of [['builder','AssignDigger',180],['builder','AssignBomber',190],['win',null,240],['loss',null,300]]) {
        game.reset(scene,'reference');host.invalidate();renderer.invalidate();
        for(let tick=0;tick<ticks;tick++){if(tick===80&&skill)game.command(skill,0);game.step(1);}
        const state=game.state(),sound=game.sound_state();
        host.draw(ctx);renderer.draw(game,true,host);
        const pixels=await readGpu(renderer);
        const comparison=compare(pixels,read(cpu),3,`HD ${scene}/${skill}`);
        comparisons.push(comparison);
        // Different rasterizers can differ around filtered sprite/atlas edges.
        assert(comparison.mean<1&&comparison.over<cpu.width*cpu.height*.03,JSON.stringify(comparison));
        assert(game.state()===state&&game.sound_state()===sound,'renderer changed simulation/audio');
        if(skill==='AssignDigger') {
            const shot=createCanvas(1920,1080);
            shot.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(pixels),1920,1080),0,0);
            const blob=await new Promise(resolve=>shot.toBlob(resolve,'image/png'));shot.remove();
            await fetch('/__screenshot',{method:'POST',body:blob});
        }
    }
    c.width=320;c.height=200;renderer.resize(320,200);
    renderer.draw(game,false,null);
    const raw=game.frame(),got=await readGpu(renderer);
    for(let s=0,d=0;s<raw.length;s+=3,d+=4)assert(got[d]===raw[s]&&got[d+1]===raw[s+1]&&got[d+2]===raw[s+2]&&got[d+3]===255,'original RGB differs');
    const state=game.state();let changed=0;
    renderer.onChange=()=>{changed++;renderer.draw(game,false,null);};
    const lost=renderer.backend.device.lost;renderer.backend.device.destroy();await lost;
    await new Promise(resolve=>setTimeout(resolve,0));
    assert(renderer.kind==='canvas2d'&&changed===1,'device loss did not switch');
    assert(game.state()===state&&c.style.opacity==='','loss changed game or input visibility');
    const forced=await WebRenderer.create(createCanvas(320,200),{preference:'canvas2d'});
    const insecure=await WebRenderer.create(createCanvas(320,200),{secure:false});
    assert(forced.kind==='canvas2d'&&insecure.kind==='canvas2d','fallback selection failed');
    // Exercise the product's real DOM, rather than only its renderer adapter.
    const frame=document.createElement('iframe');
    const clock=`<script>
        window.requestAnimationFrame=()=>0;
        const original=GPUAdapter.prototype.requestDevice;
        GPUAdapter.prototype.requestDevice=async function(...args){
            const device=await original.apply(this,args);window.qaDevice=device;return device;
        };
    </script>`;
    const html=await(await fetch('./index.html')).text();
    frame.srcdoc=html.replace('<script type="module" src="./app.js">',clock+'<script type="module" src="./app.js">');
    const loaded=new Promise(resolve=>frame.onload=resolve);document.body.append(frame);await loaded;
    const win=frame.contentWindow,doc=win.document,app=await win.eval('import("./app.js")');
    let live;const detach=app.observe((type,g)=>{live=g;});
    assert(live&&doc.querySelector('#renderer-status').textContent==='WebGPU','product did not select WebGPU');
    const input=doc.querySelector('#game');
    const key=code=>input.dispatchEvent(new win.KeyboardEvent('keydown',{code,bubbles:true,cancelable:true}));
    key('Space');assert(live.paused(),'GPU input cannot pause');key('Space');assert(!live.paused(),'GPU input cannot resume');
    const presentation=doc.querySelector('#presentation'),initial=live.state();
    presentation.value='hd';presentation.dispatchEvent(new Event('change'));
    for(let n=0;presentation.disabled;n++){assert(n<600,'product HD timeout');await new Promise(resolve=>setTimeout(resolve,50));}
    assert(live.state()===initial&&input.width===1920,'HD switch changed game');
    const display=doc.querySelector('[data-game-display]'),a=input.getBoundingClientRect(),b=display.getBoundingClientRect();
    assert(display!==input&&Math.abs(a.x-b.x)<.1&&Math.abs(a.y-b.y)<.1&&Math.abs(a.width-b.width)<.1&&Math.abs(a.height-b.height)<.1,'GPU surface/input geometry differs');
    const levels=[];
    if(live.original()) for(const level of [2,22]) {
        doc.querySelector('#level').value=String(level);doc.querySelector('#open-level').click();
        for(let n=0;presentation.disabled;n++){assert(n<600,'product level timeout');await new Promise(resolve=>setTimeout(resolve,50));}
        assert(live.level()===level&&presentation.value==='hd'&&!doc.querySelector('#error').textContent,'original level transition failed');
        assert(doc.querySelector('#renderer-status').textContent==='WebGPU','level change dropped GPU');levels.push(level);
    }
    key('Space');assert(live.paused(),'HD input cannot pause');
    const held=live.state(),loss=win.qaDevice.lost;win.qaDevice.destroy();await loss;
    await new Promise(resolve=>setTimeout(resolve,10));
    assert(doc.querySelector('#renderer-status').textContent==='Canvas2D'&&live.state()===held,'product loss changed game');
    assert(doc.querySelector('#game')===input&&doc.querySelector('[data-game-display]')===input,'loss replaced input canvas');
    key('Space');assert(!live.paused(),'input broken after loss');
    presentation.value='original';presentation.dispatchEvent(new Event('change'));
    assert(input.width===320&&!doc.querySelector('#error').textContent,'original switch failed after loss');
    doc.querySelector('#restart').click();assert(JSON.parse(live.state()).session.control_step===0,'restart failed after loss');
    detach();frame.remove();
    game.free();renderer.dispose();
    result={status:'PASS',adapter:{vendor:info?.vendor,architecture:info?.architecture,device:info?.device,description:info?.description,isFallbackAdapter:info?.isFallbackAdapter},
        comparisons,original_pixel_exact:true,revision_cache:true,device_loss:true,state_audio_unchanged:true,
        product:{gpu_selected:true,hd_original_switch:true,input_preserved:true,surface_geometry:true,loss_while_paused:true,restart_after_loss:true,
            original_levels:levels.length?[1,...levels]:[]}};
} catch(error) {result={status:'FAIL',error:String(error),stack:error.stack};}
document.querySelector('#result').textContent=JSON.stringify(result,null,2);
await fetch('/__result',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(result)});
