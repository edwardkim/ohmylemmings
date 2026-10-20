import init, { WebGame } from './pkg/lemmings_web.js';
import { FramePump, present } from './host.js';
import { GameInput, readFiles } from './controls.js';

const canvas = document.querySelector('#game');
const context = canvas.getContext('2d', { alpha: false });
const status = document.querySelector('#status');
const scene = document.querySelector('#scene');
const pause = document.querySelector('#pause');
const restart = document.querySelector('#restart');
const proceed = document.querySelector('#continue');
const population = document.querySelector('#population');
const files = document.querySelector('#files');
const level = document.querySelector('#level');
const openLevel = document.querySelector('#open-level');
const errorText = document.querySelector('#error');
const source = document.querySelector('#source');
let game;
let pump;
let input;
let request = null;
let loading = false;
let exited = false;
let observer = null;
function notify(type, detail = {}) { if (observer) observer(type, game, detail); }
const tickObserver = phase => notify(phase);
function newPump() { return new FramePump(game,64,observer ? tickObserver : null); }
// Optional developer observation. It has no UI and sends no data anywhere.
export function observe(listener) {
    if (listener !== null && typeof listener !== 'function') throw new Error('invalid observer');
    observer=listener;
    if (pump) pump.observer=listener ? tickObserver : null;
    if (listener && game) notify('attach');
    return () => { if (observer===listener) {observer=null;if(pump)pump.observer=null;} };
}
function invoke(op,args,fn) { notify('input',{op,args});return fn(); }
const inputGame={
    key:(...args)=>invoke('Key',[args[0],Number(args[1])],()=>game.key(...args)),
    pointer:(...args)=>invoke('Pointer',args,()=>game.pointer(...args)),
    release_input:()=>invoke('Release',[],()=>game.release_input()),
};
for (let n = 1; n <= 30; n++) level.add(new Option(`Fun ${String(n).padStart(2,'0')}`, n));

function stop() {
    if (request !== null) cancelAnimationFrame(request);
    request = null;
}
function recover(error) {
    errorText.textContent = `불러오기 실패: ${String(error)} 현재 게임에서 계속할 수 있습니다.`;
}
function fail(error) {
    stop();
    console.error(error);
    errorText.textContent = '게임을 실행하지 못했습니다. 새로고침해주세요.';
    for (const control of document.querySelectorAll('button,select,input')) control.disabled = true;
}
function draw() {
    present(context, game.frame(), game.width(), game.height());
    const state = JSON.parse(game.state());
    const finished = state.completion && state.completion.phase !== 'Playing';
    const paused = game.paused();
    pause.textContent = exited ? '게임으로 돌아가기' : paused ? '계속하기' : '멈추기';
    pause.disabled = loading || Boolean(finished);
    restart.disabled = loading;
    scene.disabled = files.disabled = loading;
    openLevel.disabled = loading || !game.original();
    level.disabled = loading;
    proceed.hidden = !finished;
    proceed.disabled = loading || !game.can_continue();
    proceed.textContent = state.completion?.phase === 'Won' ? game.can_continue() ? '다음 레벨' : '마지막 레벨 완료' : '다시 도전';
    status.textContent = loading ? '파일을 읽는 중…' : exited ? '게임 중단' : finished ? (state.completion.phase === 'Won' ? '구출 완료' : '다시 도전') : paused ? '일시 정지' : '진행 중';
    population.textContent = `활동 중 ${state.session.alive} · 구출 ${state.exit.saved}`;
    source.textContent = game.original() ? `원본 파일 · Fun ${String(game.level()).padStart(2,'0')}` : '직접 만든 자산으로 기술과 구출을 연습합니다.';
    notify('present',{canvas});
    if (finished || exited) stop();
    return !finished && !exited;
}
function schedule() {
    if (request === null && !document.hidden && !exited) request = requestAnimationFrame(frame);
}
function frame(timestamp) {
    request = null;
    try {
        if (document.hidden) { pump.suspend(); return; }
        pump.frame(timestamp);
        if (draw()) schedule();
    } catch (error) { fail(error); }
}
function replaced(reason) {
    notify('reset',{reason});
    input.release();
    exited = false;
    errorText.textContent = '';
    if (game.original()) { level.value = game.level(); scene.value = ''; }
    pump = newPump();
    stop(); draw(); schedule();
}
function action(name) {
    if (name === 'exit') { exited = true; input.release(); pump.suspend(); stop(); }
    else if ((name === 'restart' || name === 'continue') && game.transition(name)) replaced(name);
}
function attempt(fn) {
    try { fn(); draw(); } catch (error) { recover(error); draw(); }
}
pause.addEventListener('click', () => attempt(() => {
    if (exited) { exited = false; pump.suspend(); schedule(); }
    else {const command=game.paused() ? 'Resume' : 'Pause';invoke(command,[0],()=>game.command(command,0));}
}));
restart.addEventListener('click', () => attempt(() => { game.restart_level(); replaced('restart'); }));
proceed.addEventListener('click', () => attempt(() => action('continue')));
scene.addEventListener('change', () => attempt(() => { game.reset(scene.value, 'reference'); replaced('synthetic'); }));
openLevel.addEventListener('click', () => attempt(() => { game.start_level(Number(level.value)); replaced('level'); }));
files.addEventListener('change', async () => {
    if (!files.files.length || loading) return;
    loading = true;
    input.release();
    draw();
    let bank;
    try {
        bank = await readFiles(Array.from(files.files));
        game.load_original(bank, Number(level.value));
        replaced('load');
    } catch (error) { recover(error); }
    finally { bank?.free(); loading = false; files.value = ''; draw(); }
});
document.addEventListener('visibilitychange', () => {
    if (!game) return;
    input.release(); pump.suspend(); stop();
    if (!document.hidden && draw()) schedule();
});

try {
    await init();
    game = new WebGame(scene.value, 'reference');
    pump = newPump();
    input = new GameInput(canvas, () => inputGame, action, draw, recover, () => !loading && !exited);
    draw(); schedule();
} catch (error) { fail(error); }
