import init, { WebGame, WebAudio, WebAssets } from './pkg/lemmings_web.js';
import { FramePump, present } from './host.js';
import { AudioHost } from './audio.js';
import { GameInput, readFiles } from './controls.js';
import { loadBundled } from './bundled.js';
import { HdCanvas } from './hd.js';
import { HdLoader } from './hd-loader.js';

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
const difficulty = document.querySelector('#difficulty');
const openLevel = document.querySelector('#open-level');
const errorText = document.querySelector('#error');
const source = document.querySelector('#source');
const presentation=document.querySelector('#presentation');
let hd;const hdLoader=new HdLoader();let hdScope=null;let hdPending=false;let hdMode=false;
function screenSize(){canvas.classList.toggle('hd',hdMode);canvas.width=hdMode?1920:320;canvas.height=hdMode?1080:200;canvas.style.setProperty('--aspect',String(canvas.width/canvas.height));}
let audio;
let audioHeld=false;
let game;
let pump;
let input;
let request = null;
let loading = false;
let exited = false;
let observer = null;
let presentations = 0;
let needsDraw = false;
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
function invoke(op,args,fn) { needsDraw=true;notify('input',{op,args});return fn(); }
const inputGame={
    key:(...args)=>invoke('Key',[args[0],Number(args[1])],()=>game.key(...args)),
    pointer:(...args)=>invoke(hdMode?'HdPointer':'Pointer',args,()=>hdMode?game.hd_pointer(...args):game.pointer(...args)),
    release_input:()=>invoke('Release',[],()=>game.release_input()),
};
for (let n = 1; n <= 30; n++) level.add(new Option(String(n).padStart(2,'0'), n));

function stop() {
    if (request !== null) cancelAnimationFrame(request);
    request = null;
}
function recover(error) {
    errorText.textContent = `불러오기 실패: ${String(error)} 현재 게임에서 계속할 수 있습니다.`;
}
function fail(error) {
    stop();
    audio?.mute();
    console.error(error);
    status.textContent = '시작 실패';
    errorText.textContent = `게임을 실행하지 못했습니다: ${String(error)} 파일을 모두 압축 해제했는지 확인하고 새로고침해주세요.`;
    for (const control of document.querySelectorAll('button,select,input')) control.disabled = true;
}
function draw() {
    presentations++;
    needsDraw=false;
    if(!hdPending){if(hdMode)hd.draw(context);else present(context, game.frame(), game.width(), game.height());}
    const state = JSON.parse(game.state());
    const finished = state.completion && state.completion.phase !== 'Playing';
    const paused = game.paused();
    audioHeld=loading||Boolean(finished)||exited;
    audio?.service();
    audio?.hold(document.hidden||audioHeld);
    pause.textContent = exited ? '게임으로 돌아가기' : paused ? '계속하기' : '멈추기';
    pause.disabled = loading || Boolean(finished);
    restart.disabled = loading;
    presentation.disabled=loading;
    scene.disabled = files.disabled = loading;
    openLevel.disabled = loading || !game.original();
    level.disabled = difficulty.disabled = loading;
    proceed.hidden = !finished;
    proceed.disabled = loading || !game.can_continue();
    proceed.textContent = state.completion?.phase === 'Won' ? game.can_continue() ? '다음 레벨' : '마지막 레벨 완료' : '다시 도전';
    status.textContent = loading ? '파일을 읽는 중…' : exited ? '게임 중단' : finished ? (state.completion.phase === 'Won' ? '구출 완료' : '다시 도전') : paused ? '일시 정지' : '진행 중';
    population.textContent = `활동 중 ${state.session.alive} · 구출 ${state.exit.saved}`;
    const rank = game.difficulty();
    source.textContent = game.original() ? `원본 파일 · ${rank[0].toUpperCase()+rank.slice(1)} ${String(game.level()).padStart(2,'0')}` : '직접 만든 자산으로 기술과 구출을 연습합니다.';
    if(!hdPending)notify('present',{canvas});
    if (finished || exited) stop();
    return !finished && !exited;
}
function schedule() {
    if (request === null && !document.hidden && !exited) request = requestAnimationFrame(frame);
}
function frame(timestamp) {
    request = null;
    try {
        if (document.hidden || loading) { pump.suspend(); return; }
        const ticks = pump.frame(timestamp);
        // Inputs and replacements already draw synchronously. Between admitted
        // ticks the pixels and UI are unchanged; still service real-time audio.
        if (ticks || needsDraw) { if (draw()) schedule(); }
        else { audio?.service(); schedule(); }
    } catch (error) { fail(error); }
}
function replaced(reason) {
    hd?.invalidate();
    notify('reset',{reason});
    input.release();
    exited = false;
    errorText.textContent = '';
    if (game.original()) { difficulty.value = game.difficulty(); level.value = game.level(); scene.value = ''; }
    pump = newPump();
    stop();
    if(hdMode && hdScope!==game.hd_scope()) {
        loading=true;hdPending=true;draw();
        void hdLoader.load(game).then(()=>{hdScope=game.hd_scope();hd.invalidate();})
            .catch(error=>{hdMode=false;presentation.value='original';screenSize();recover(error);})
            .finally(()=>{hdPending=false;loading=false;pump.suspend();draw();schedule();});
    } else {draw();schedule();}
}
function action(name) {
    if(name==='restart-level'){game.restart_level();replaced('restart');return;}
    if (name === 'exit') { exited = true; input.release(); pump.suspend(); stop(); }
    else if ((name === 'restart' || name === 'continue') && game.transition(name)) replaced(name);
}
function attempt(fn) {
    const before = presentations;
    try { fn(); if (presentations === before) draw(); } catch (error) { recover(error); draw(); }
}
pause.addEventListener('click', () => attempt(() => {
    if (exited) { exited = false; pump.suspend(); schedule(); }
    else {const command=game.paused() ? 'Resume' : 'Pause';invoke(command,[0],()=>game.command(command,0));}
}));
restart.addEventListener('click', () => attempt(() => { game.restart_level(); replaced('restart'); }));
proceed.addEventListener('click', () => attempt(() => action('continue')));
scene.addEventListener('change', () => attempt(() => { game.reset(scene.value, 'reference'); replaced('synthetic'); }));
openLevel.addEventListener('click', () => attempt(() => { game.start_level_rank(difficulty.value, Number(level.value)); replaced('level'); }));
files.addEventListener('change', async () => {
    if (!files.files.length || loading) return;
    loading = true;
    input.release();
    draw();
    let bank;
    try {
        bank = await readFiles(Array.from(files.files));
        game.load_original_rank(bank, difficulty.value, Number(level.value));
        replaced('load');
    } catch (error) { recover(error); }
    finally { bank?.free(); if(!hdPending)loading = false; files.value = ''; draw(); }
});
document.addEventListener('visibilitychange', () => {
    if (!input) return;
    input.release(); pump.suspend(); stop();
    audio?.hold(document.hidden||audioHeld);
    if (!document.hidden && draw()) schedule();
});

presentation.addEventListener('change',async()=>{
    if(!game||loading)return;
    const requested=presentation.value==='hd';input.release();loading=true;draw();
    try {
        if(requested&&(!hd||hdScope!==game.hd_scope())){
            await hdLoader.load(game);hd=new HdCanvas(game);hdScope=game.hd_scope();
        }
        hdMode=requested;screenSize();errorText.textContent='';
    } catch(error){presentation.value=hdMode?'hd':'original';recover(error);}
    finally {loading=false;pump.suspend();draw();schedule();canvas.focus({preventScroll:true});}
});
document.querySelector('#fullscreen').addEventListener('click',()=>{
    const screen=canvas.parentElement;
    const action=document.fullscreenElement?document.exitFullscreen():screen.requestFullscreen();
    action.catch(recover);
});
try {
    await init();
    game = new WebGame(scene.value, 'reference');
    const bundled = await loadBundled(WebAssets, fetch, Boolean(document.querySelector('meta[name="game-data"]')));
    if (bundled) {
        try { game.load_original_rank(bundled, 'fun', 1); } finally { bundled.free(); }
        difficulty.value = 'fun'; level.value = '1'; scene.value = '';
        document.querySelector('#asset-help').textContent = '원본 그래픽·120개 레벨·음원이 준비되어 있습니다. 난이도와 레벨을 골라 플레이하세요. 다른 파일을 직접 선택할 수도 있습니다.';
    }
    pump = newPump();
    const sound=document.querySelector('#sound'),volume=document.querySelector('#volume'),audioStatus=document.querySelector('#audio-status');
    const audioSupported=typeof globalThis.AudioContext==='function';
    audio=new AudioHost(()=>game,WebAudio,({state,volume,muted,error})=>{
        if(!audioSupported){sound.disabled=true;audioStatus.textContent='이 브라우저에서는 소리를 사용할 수 없습니다. 게임은 계속할 수 있습니다.';return;}
        sound.disabled=state==='starting';sound.textContent=muted?'소리 켜기':'음소거';
        audioStatus.textContent=state==='on'?`소리 켜짐 · ${volume}%`:state==='held'?'소리 일시 중지':state==='starting'?'소리를 준비하는 중…':state==='error'?(error.includes('ADLIB.DAT')?'ADLIB.DAT 파일을 함께 선택한 후 소리를 켜주세요.':'소리를 재생하지 못했습니다. 소리 켜기로 다시 시도해주세요.'):'소리는 버튼을 눌러 켤 수 있습니다.';
    },()=>!audioHeld&&!document.hidden);
    const toggleSound=()=>{if(!audioSupported)return;if(audio.muted)void audio.activate();else audio.mute();};
    sound.addEventListener('click',toggleSound);
    canvas.addEventListener('keydown',event=>{
        if(event.isComposing||event.ctrlKey||event.altKey||event.metaKey||!['KeyS','PageUp','PageDown'].includes(event.code))return;
        event.preventDefault();if(event.repeat)return;
        if(event.code==='KeyS')toggleSound();else {volume.value=Math.max(0,Math.min(100,Number(volume.value)+(event.code==='PageUp'?10:-10)));audio.setVolume(Number(volume.value));}
    });
    volume.addEventListener('input',()=>audio.setVolume(Number(volume.value)));
    audio.setVolume(Number(volume.value));
    window.addEventListener('pagehide',()=>void audio.dispose());
    input = new GameInput(canvas, () => inputGame, action, draw, recover, () => !loading && !exited);
    draw(); schedule();
} catch (error) { fail(error); }
