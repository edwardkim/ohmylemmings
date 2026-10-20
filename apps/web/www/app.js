import init, { WebGame } from './pkg/lemmings_web.js';
import { FramePump, present } from './host.js';

const canvas = document.querySelector('#game');
const context = canvas.getContext('2d', { alpha: false });
const status = document.querySelector('#status');
const scene = document.querySelector('#scene');
const pause = document.querySelector('#pause');
const restart = document.querySelector('#restart');
const population = document.querySelector('#population');
let game;
let pump;
let request = null;

function stop() {
    if (request !== null) cancelAnimationFrame(request);
    request = null;
}
function fail(error) {
    stop();
    console.error(error);
    status.textContent = '게임을 불러오지 못했습니다. 새로고침해주세요.';
    scene.disabled = pause.disabled = restart.disabled = true;
}
function draw() {
    present(context, game.frame(), game.width(), game.height());
    const state = JSON.parse(game.state());
    const finished = state.completion && state.completion.phase !== 'Playing';
    const paused = game.paused();
    pause.textContent = paused ? '계속하기' : '멈추기';
    pause.disabled = Boolean(finished);
    status.textContent = finished ? (state.completion.phase === 'Won' ? '구출 완료' : '다시 도전') : paused ? '일시 정지' : '진행 중';
    population.textContent = `활동 중 ${state.session.alive} · 구출 ${state.exit.saved}`;
    if (finished) stop();
    return !finished;
}
function schedule() {
    if (request === null && !document.hidden) request = requestAnimationFrame(frame);
}
function frame(timestamp) {
    request = null;
    try {
        if (document.hidden) { pump.suspend(); return; }
        pump.frame(timestamp);
        if (draw()) schedule();
    } catch (error) { fail(error); }
}
function reset() {
    try {
        game.reset(scene.value, 'reference');
        pump = new FramePump(game);
        stop(); draw(); schedule();
    } catch (error) { fail(error); }
}
pause.addEventListener('click', () => {
    try { game.command(game.paused() ? 'Resume' : 'Pause', 0); draw(); }
    catch (error) { fail(error); }
});
restart.addEventListener('click', reset);
scene.addEventListener('change', reset);
document.addEventListener('visibilitychange', () => {
    if (!game) return;
    pump.suspend(); stop();
    if (!document.hidden && draw()) schedule();
});

try {
    await init();
    game = new WebGame(scene.value, 'reference');
    pump = new FramePump(game);
    scene.disabled = pause.disabled = restart.disabled = false;
    draw(); schedule();
} catch (error) { fail(error); }
