import { present } from './host.js';
import { WebGpu } from './webgpu.js';

// Keep the input canvas and its event handlers alive when the GPU is lost.
// The sibling is displayed directly; there is no per-frame GPU -> 2D copy.
export class WebRenderer {
    static async create(canvas, {gpu = globalThis.navigator?.gpu, secure = globalThis.isSecureContext,
        preference = 'auto', timeoutMs = 3000, createGpu = WebGpu.create, onChange = () => {}} = {}) {
        const renderer = new WebRenderer(canvas, onChange);
        if (preference === 'canvas2d') renderer.reason = 'Canvas2D requested';
        else if (!secure) renderer.reason = 'secure context required';
        else if (!gpu) renderer.reason = 'WebGPU unavailable';
        else {
            const surface = canvas.ownerDocument.createElement('canvas');
            surface.width = canvas.width; surface.height = canvas.height;
            surface.className = canvas.className; surface.dataset.webgpu = '';
            surface.setAttribute('aria-hidden', 'true');
            let expired = false, timer, earlyFailure;
            const pending = Promise.resolve().then(() => createGpu(surface, gpu, reason => {
                earlyFailure = reason; renderer.fallback(reason);
            }));
            pending.then(value => {if (expired) value.dispose();}, () => {});
            try {
                const backend = await Promise.race([pending, new Promise((resolve,reject) => {
                    timer = setTimeout(() => {expired = true;reject(Error('WebGPU initialization timed out'));}, timeoutMs);
                })]);
                if (earlyFailure) {backend.dispose();throw Error(earlyFailure);}
                renderer.backend = backend; renderer.surface = surface; renderer.kind = 'webgpu';
                renderer.reason = ''; canvas.after(surface); canvas.style.opacity = '0';
                delete canvas.dataset.gameDisplay; surface.dataset.gameDisplay = '';
            } catch (error) { renderer.reason = String(error.message || error); }
            finally { clearTimeout(timer); }
        }
        return renderer;
    }
    constructor(canvas, onChange) {
        this.canvas = canvas; this.surface = canvas; this.kind = 'canvas2d'; this.reason = '';
        canvas.dataset.gameDisplay = '';
        this.context = canvas.getContext('2d', {alpha:false}); this.onChange = onChange;
        if (!this.context) throw Error('Canvas2D unavailable');
    }
    resize(width, height) {
        this.surface.width = width; this.surface.height = height;
        this.surface.className = this.canvas.className;
    }
    fallback(reason) {
        if (this.kind !== 'webgpu') return;
        this.kind = 'canvas2d'; this.reason = String(reason);
        try {this.backend.dispose();} finally {
            this.backend = null; this.surface.remove(); this.surface = this.canvas;
            this.canvas.dataset.gameDisplay = ''; this.canvas.style.opacity = ''; this.onChange();
        }
    }
    draw(game, hdMode, hd) {
        if (this.backend) {
            try {
                if (hdMode) this.backend.drawHd(game);
                else this.backend.drawOriginal(game.frame(), game.width(), game.height());
                return;
            } catch (error) {this.fallback(String(error.message || error));}
        }
        if (hdMode) hd.draw(this.context);
        else present(this.context, game.frame(), game.width(), game.height());
    }
    invalidate() {this.backend?.invalidate();}
    dispose() {this.backend?.dispose();}
}
