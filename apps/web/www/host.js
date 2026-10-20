// Browser time/Canvas adapter. All game updates and drawing stay in Rust.
export class FramePump {
    constructor(game, budget = 64, observer = null) {
        if (!Number.isInteger(budget) || budget < 1 || budget > 1000) {
            throw new Error('tick budget must be in 1..=1000');
        }
        if (observer !== null && typeof observer !== 'function') throw new Error('invalid tick observer');
        this.observer = observer;
        this.game = game;
        this.budget = budget;
        this.lastUs = null;
        this.pending = 0;
    }
    suspend() {
        // Hidden-tab wall time is not admitted; already admitted tick debt survives.
        this.lastUs = null;
    }
    frame(timestamp) {
        if (!Number.isFinite(timestamp) || timestamp < 0 || timestamp > Number.MAX_SAFE_INTEGER / 1000) {
            throw new Error('invalid animation timestamp');
        }
        const now = Math.floor(timestamp * 1000);
        const elapsed = this.lastUs === null ? 0 : now - this.lastUs;
        if (elapsed < 0 || elapsed > 3600000000 || this.pending > Number.MAX_SAFE_INTEGER - 61200) {
            throw new Error('animation time or accumulated debt out of range');
        }
        const due = this.game.due(elapsed);
        this.lastUs = now;
        this.pending += due;
        const requested = Math.min(this.pending, this.budget);
        let executed = 0;
        if (this.observer === null) executed = this.game.step(requested);
        else for (let i=0;i<requested;i++) {
            this.observer('before',this.game);
            const count=this.game.step(1);
            if (!count) break;
            executed+=count;
            this.observer('after',this.game);
        }
        this.pending = executed < requested ? 0 : this.pending - executed;
        return executed;
    }
}

export function present(context, rgb, width, height) {
    if (rgb.length !== width * height * 3) throw new Error('invalid RGB frame size');
    const image = context.createImageData(width, height);
    for (let src = 0, dest = 0; src < rgb.length; src += 3, dest += 4) {
        image.data[dest] = rgb[src];
        image.data[dest + 1] = rgb[src + 1];
        image.data[dest + 2] = rgb[src + 2];
        image.data[dest + 3] = 255;
    }
    context.putImageData(image, 0, 0);
}
