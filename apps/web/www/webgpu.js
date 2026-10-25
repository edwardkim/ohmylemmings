// Browser-only adapter for the shared Rust draw-list. No simulation or I/O
// policy belongs here. Textures use premultiplied RGBA, matching Canvas2D.
export function premultiply(bytes) {
    const pixels = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; i += 4) {
        const alpha = bytes[i + 3];
        for (let c = 0; c < 3; c++) pixels[i + c] = Math.round(bytes[i + c] * alpha / 255);
        pixels[i + 3] = alpha;
    }
    return pixels;
}

export function compileCommands(commands, width, height, texture) {
    if (commands.length % 12 || commands.length > 12 * 32768) throw Error('invalid HD draw-list length');
    const vertices = new Float32Array(commands.length / 12 * 6 * 8);
    const draws = [];
    let offset = 0, clip = height;
    function quad(id, x, y, w, h, u, v, uw, vh, color) {
        if (w <= 0 || h <= 0) return;
        const first = offset / 8;
        for (const [dx, dy] of [[0,0],[1,0],[0,1],[0,1],[1,0],[1,1]]) {
            vertices.set([(x + dx * w) * 2 / width - 1, 1 - (y + dy * h) * 2 / height,
                u + dx * uw, v + dy * vh, ...color], offset);
            offset += 8;
        }
        draws.push({id, first, clip});
    }
    for (let i = 0; i < commands.length; i += 12) {
        const c = commands.subarray(i, i + 12);
        if (c[0] === 0) {
            const a = c[8] / 255;
            quad(null, c[1], c[2], c[3], c[4], 0, 0, 1, 1,
                [c[5] / 255 * a, c[6] / 255 * a, c[7] / 255 * a, a]);
        } else if (c[0] === 1) {
            const t = texture(c[1]);
            quad(c[1], c[6], c[7], c[8], c[9], c[2] / t.width, c[3] / t.height,
                c[4] / t.width, c[5] / t.height, [1,1,1,1]);
        } else if (c[0] === 2) clip = c[1] ? Math.min(960, height) : height;
        else throw Error('unknown HD draw command');
    }
    return {vertices:vertices.subarray(0, offset), draws};
}

const SHADER = `
struct Vertex {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec4f,
}
@vertex fn vertex(@location(0) position: vec2f, @location(1) uv: vec2f,
                  @location(2) color: vec4f) -> Vertex {
    var out: Vertex;
    out.position = vec4f(position, 0.0, 1.0);
    out.uv = uv; out.color = color;
    return out;
}
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var filtering: sampler;
@fragment fn fragment(in: Vertex) -> @location(0) vec4f {
    return textureSample(image, filtering, in.uv) * in.color;
}`;

export class WebGpu {
    static async create(canvas, gpu, onLost) {
        const adapter = await gpu.requestAdapter();
        if (!adapter) throw Error('no WebGPU adapter');
        const device = await adapter.requestDevice();
        let renderer;
        try {
            const context = canvas.getContext('webgpu');
            if (!context) throw Error('WebGPU canvas unavailable');
            const format = gpu.getPreferredCanvasFormat();
            device.pushErrorScope('validation');
            context.configure({device, format, alphaMode:'opaque'});
            renderer = new WebGpu(canvas, context, device);
            const module = device.createShaderModule({code:SHADER});
            renderer.pipeline = await device.createRenderPipelineAsync({layout:'auto',
                vertex:{module, entryPoint:'vertex', buffers:[{arrayStride:32, attributes:[
                    {shaderLocation:0, offset:0, format:'float32x2'},
                    {shaderLocation:1, offset:8, format:'float32x2'},
                    {shaderLocation:2, offset:16, format:'float32x4'}]}]},
                fragment:{module, entryPoint:'fragment', targets:[{format, blend:{
                    color:{srcFactor:'one', dstFactor:'one-minus-src-alpha'},
                    alpha:{srcFactor:'one', dstFactor:'one-minus-src-alpha'}}}]},
                primitive:{topology:'triangle-list'}});
            renderer.white = renderer.upload(new Uint8Array([255,255,255,255]), 1, 1);
            const error = await device.popErrorScope();
            if (error) throw Error(error.message);
            device.lost.then(info => {if (!renderer.disposed) onLost(`device lost: ${info.reason}`);});
            renderer.onError = event => {event.preventDefault();onLost(`GPU error: ${event.error.message}`);};
            device.addEventListener('uncapturederror', renderer.onError);
            return renderer;
        } catch (error) {
            if (renderer) renderer.dispose(); else device.destroy();
            throw error;
        }
    }
    constructor(canvas, context, device) {
        this.canvas = canvas; this.context = context; this.device = device;
        this.cache = new Map(); this.disposed = false; this.capacity = 0;
        this.linear = device.createSampler({magFilter:'linear', minFilter:'linear'});
        this.nearest = device.createSampler({magFilter:'nearest', minFilter:'nearest'});
    }
    upload(bytes, width, height, nearest = false) {
        if (!width || !height || width > this.device.limits.maxTextureDimension2D ||
            height > this.device.limits.maxTextureDimension2D || bytes.length !== width * height * 4) {
            throw Error('invalid GPU texture size');
        }
        const texture = this.device.createTexture({size:[width,height], format:'rgba8unorm',
            usage:GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST});
        this.device.queue.writeTexture({texture}, bytes, {bytesPerRow:width * 4}, [width,height]);
        const bind = this.device.createBindGroup({layout:this.pipeline.getBindGroupLayout(0),entries:[
            {binding:0, resource:texture.createView()},
            {binding:1, resource:nearest ? this.nearest : this.linear}]});
        return {texture, bind, width, height};
    }
    texture(game, id) {
        const revision = game.hd_texture_revision(id);
        if (!revision) throw Error('HD texture is unavailable');
        const old = this.cache.get(id);
        if (old?.revision === revision) return old;
        const row = this.upload(premultiply(game.hd_texture(id)),
            game.hd_texture_width(id), game.hd_texture_height(id));
        old?.texture.destroy(); row.revision = revision; this.cache.set(id, row);
        return row;
    }
    drawHd(game) {
        const compiled = compileCommands(game.hd_commands(), this.canvas.width, this.canvas.height,
            id => this.texture(game, id));
        this.render(compiled, id => id === null ? this.white : this.cache.get(id));
    }
    drawOriginal(rgb, width, height) {
        if (rgb.length !== width * height * 3) throw Error('invalid RGB frame size');
        const pixels = new Uint8Array(width * height * 4);
        for (let s = 0, d = 0; s < rgb.length; s += 3, d += 4) {
            pixels[d] = rgb[s]; pixels[d + 1] = rgb[s + 1]; pixels[d + 2] = rgb[s + 2]; pixels[d + 3] = 255;
        }
        if (this.original?.width !== width || this.original?.height !== height) {
            this.original?.texture.destroy(); this.original = this.upload(pixels, width, height, true);
        } else this.device.queue.writeTexture({texture:this.original.texture}, pixels, {bytesPerRow:width * 4}, [width,height]);
        const commands = new Int32Array([1,0,0,0,width,height,0,0,width,height,0,0]);
        this.render(compileCommands(commands,width,height,()=>this.original),()=>this.original);
    }
    render({vertices, draws}, texture) {
        if (this.disposed) throw Error('GPU renderer disposed');
        if (vertices.byteLength > this.capacity) {
            this.buffer?.destroy(); this.capacity = Math.max(4096, 2 ** Math.ceil(Math.log2(vertices.byteLength)));
            this.buffer = this.device.createBuffer({size:this.capacity,usage:GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST});
        }
        if (vertices.byteLength) this.device.queue.writeBuffer(this.buffer, 0, vertices);
        const encoder = this.device.createCommandEncoder();
        const pass = encoder.beginRenderPass({colorAttachments:[{view:this.context.getCurrentTexture().createView(),
            clearValue:{r:0,g:0,b:0,a:1},loadOp:'clear',storeOp:'store'}]});
        pass.setPipeline(this.pipeline);
        if (vertices.byteLength) pass.setVertexBuffer(0, this.buffer);
        for (const draw of draws) {
            pass.setScissorRect(0, 0, this.canvas.width, draw.clip);
            pass.setBindGroup(0, texture(draw.id).bind); pass.draw(6, 1, draw.first);
        }
        pass.end(); this.device.queue.submit([encoder.finish()]);
    }
    invalidate() {
        for (const row of this.cache.values()) row.texture.destroy();
        this.cache.clear(); this.original?.texture.destroy(); this.original = null;
    }
    dispose() {
        if (this.disposed) return;
        this.disposed = true; this.invalidate(); this.white?.texture.destroy(); this.buffer?.destroy();
        if (this.onError) this.device.removeEventListener('uncapturederror', this.onError);
        this.context.unconfigure(); this.device.destroy();
    }
}
