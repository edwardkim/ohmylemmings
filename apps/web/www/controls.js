import { WebAssets } from './pkg/lemmings_web.js';

// Convert CSS pixels to logical pixels; out-of-screen values remain outside.
export function logicalPoint(canvas, event) {
    const rect = canvas.getBoundingClientRect();
    if (!rect.width || !rect.height) return null;
    return [Math.floor((event.clientX - rect.left) * 320 / rect.width),
        Math.floor((event.clientY - rect.top) * 200 / rect.height)];
}

export async function readFiles(files) {
    const bank = new WebAssets();
    try {
        for (const file of files) {
            if (!WebAssets.accepts(file.name)) continue;
            if (file.size > 1048576) throw new Error(`${file.name}: 파일은 1 MiB 이하여야 합니다.`);
            bank.insert(file.name, new Uint8Array(await file.arrayBuffer()));
        }
        if (!bank.count()) throw new Error('게임 파일을 찾지 못했습니다. DAT 파일을 선택해주세요.');
        return bank;
    } catch (error) { bank.free(); throw error; }
}

const keys = new Set(['KeyB','KeyD','KeyM','KeyU','KeyF','KeyC','KeyE','KeyK','KeyN','KeyR','Space','Enter','Escape','ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End']);

export class GameInput {
    constructor(canvas, getGame, action, changed, failed, active = () => true) {
        this.canvas = canvas;
        this.getGame = getGame;
        this.previousClick = null;
        this.buttons = 0;
        const run = fn => { try { const result = fn(); action(result); changed(); } catch (error) { failed(error); } };
        canvas.addEventListener('keydown', event => {
            if (!active()) return;
            if (event.isComposing || event.ctrlKey || event.altKey || event.metaKey || !keys.has(event.code)) return;
            event.preventDefault();
            run(() => getGame().key(event.code, event.repeat));
        });
        canvas.addEventListener('contextmenu', event => event.preventDefault());
        for (const kind of ['down','up','move']) canvas.addEventListener(`pointer${kind}`, event => {
            if (!active()) return;
            if (!event.isPrimary || (kind !== 'move' && ![0,2].includes(event.button))) return;
            const point = logicalPoint(canvas, event);
            if (!point) return;
            event.preventDefault();
            this.point = point;
            if (kind === 'down') {
                canvas.focus({ preventScroll: true });
                try { canvas.setPointerCapture(event.pointerId); } catch { /* Synthetic events have no active pointer. */ }
            }
            // A second mouse button changes pointermove.buttons without another
            // pointerdown. Translate those edges too, including right-held clicks.
            const edges = kind !== 'move' || event.button !== -1;
            const pressed = edges ? event.buttons & ~this.buttons : 0;
            const released = edges ? this.buttons & ~event.buttons : 0;
            this.buttons = event.buttons;
            run(() => {
                const game = getGame();
                let action = game.pointer('move', point[0], point[1], 0, event.buttons, 0);
                for (const [bit,button] of [[2,2],[1,0]]) {
                    if (released & bit) game.pointer('up',point[0],point[1],button,event.buttons,0);
                    if (!(pressed & bit)) continue;
                    let clicks = 1;
                    if (button === 0) {
                        const old = this.previousClick;
                        clicks = old && event.timeStamp - old.time <= 500 && Math.abs(point[0]-old.x) <= 4 && Math.abs(point[1]-old.y) <= 4 ? 2 : 1;
                        this.previousClick = clicks === 2 ? null : { time:event.timeStamp,x:point[0],y:point[1] };
                    }
                    const result=game.pointer('down',point[0],point[1],button,event.buttons,clicks);
                    if (result !== 'none') action=result;
                }
                return action;
            });
        });
        canvas.addEventListener('lostpointercapture', () => {
            this.buttons=0;
            const game=getGame();
            if (!game) return;
            const [x,y]=this.point || [-1,-1];
            game.pointer('up',x,y,0,0,0);
            game.pointer('move',x,y,0,0,0);
        });
        for (const name of ['blur','pointercancel']) canvas.addEventListener(name, () => this.release());
        canvas.addEventListener('pointerleave', event => { if (!event.buttons) this.release(); });
        window.addEventListener('blur', () => this.release());
    }
    release() {
        this.buttons = 0;
        this.point = null;
        this.previousClick = null;
        const game = this.getGame();
        if (game) game.release_input();
    }
}
