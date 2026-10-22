#!/usr/bin/env node
// Functional qualification of a deployed Pages release using Node's built-in CDP client.
// Usage: node tools/check_pages.cjs HTTPS_URL STAGED_DIRECTORY EVIDENCE_DIRECTORY [CDP_PORT]
const fs = require('node:fs');
const path = require('node:path');
const {createHash} = require('node:crypto');
const assert = (ok, message) => {if (!ok) throw Error(message);};
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));

(async () => {
    const [url, directory, output, port = '19222'] = process.argv.slice(2);
    assert(url && directory && output, 'Expected HTTPS_URL STAGED_DIRECTORY EVIDENCE_DIRECTORY [CDP_PORT]');
    const base = new URL(url);
    assert(base.protocol === 'https:' && base.pathname.endsWith('/'), 'Expected an HTTPS directory URL');
    fs.mkdirSync(output, {recursive: false});
    const names = fs.readdirSync(directory, {recursive: true}).filter(name =>
        fs.statSync(path.join(directory, name)).isFile() && name !== '.nojekyll').sort();
    const http = [];
    let next = 0;
    await Promise.all(Array.from({length: 6}, async () => {
        while (next < names.length) {
            const name = names[next++];
            const response = await fetch(new URL(name, base), {signal: AbortSignal.timeout(30000)});
            assert(response.ok && response.url.startsWith(base.href), 'Public HTTP failed: ' + name);
            const bytes = Buffer.from(await response.arrayBuffer());
            const hash = createHash('sha256').update(bytes).digest('hex');
            assert(hash === createHash('sha256').update(fs.readFileSync(path.join(directory, name))).digest('hex'),
                'Public payload differs: ' + name);
            const mime = response.headers.get('content-type');
            if (name.endsWith('.wasm')) assert(mime?.split(';')[0] === 'application/wasm', 'Incorrect WASM MIME');
            http.push({path: name, bytes: bytes.length, sha256: hash, mime});
        }
    }));
    fs.writeFileSync(path.join(output, 'http.json'), JSON.stringify(http.sort((a,b) => a.path.localeCompare(b.path)), null, 2));
    console.log('PASS public HTTPS payload', http.length, 'files; WASM MIME');
    const endpoint = `http://127.0.0.1:${port}`;
    const target = await (await fetch(endpoint + '/json/new?about:blank', {method: 'PUT'})).json();
    fs.writeFileSync(path.join(output, 'tab.json'), JSON.stringify(target));
    const ws = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {ws.onopen = resolve; ws.onerror = reject;});
    let id = 0;
    let navigation;
    const pending = new Map();
    ws.onmessage = event => {
        const message = JSON.parse(event.data);
        if (message.method === 'Page.loadEventFired') navigation?.();
        if (!message.id) return;
        const request = pending.get(message.id); pending.delete(message.id);
        message.error ? request.reject(Error(JSON.stringify(message.error))) : request.resolve(message.result);
    };
    const command = (method, params = {}) => new Promise((resolve, reject) => {
        const number = ++id;
        const timer = setTimeout(() => {pending.delete(number); reject(Error('CDP timeout: ' + method));}, 60000);
        pending.set(number, {resolve: value => {clearTimeout(timer); resolve(value);},
                             reject: error => {clearTimeout(timer); reject(error);}});
        ws.send(JSON.stringify({id: number, method, params}));
    });
    const evaluate = async expression => {
        const result = await command('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true});
        assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
        return result.result.value;
    };
    async function navigate(method, params) {
        let timer;
        const loaded = new Promise((resolve, reject) => {
            timer = setTimeout(() => reject(Error('Page load timeout')), 60000);
            navigation = resolve;
        });
        try {await command(method, params); await loaded;}
        finally {clearTimeout(timer); navigation = null;}
    }
    async function ready() {
        for (let n = 0; n < 120; n++) {
            if (await evaluate(`(async()=>{if(!document.querySelector('#game'))return false;
                const app=await import('./app.js');let game;app.observe((_,g)=>game=g);
                return Boolean(game?.original()&&!document.querySelector('#error').textContent);})()`)) return;
            await pause(250);
        }
        throw Error('Published original game did not become ready');
    }
    try {
        await command('Page.enable'); await command('Network.enable');
        await command('Network.setCacheDisabled', {cacheDisabled: true});
        const clock = await command('Page.addScriptToEvaluateOnNewDocument', {
            source: 'window.requestAnimationFrame=()=>1;window.cancelAnimationFrame=()=>{};'});
        await navigate('Page.navigate', {url}); await command('Page.bringToFront'); await ready();
        const initial = await evaluate(`(async()=>{
            const assert=(ok,m)=>{if(!ok)throw Error(m);};const app=await import('./app.js');
            let game;const detach=app.observe((_,g)=>game=g);window.__pages={game,detach};
            assert(game.original()&&game.difficulty()==='fun'&&game.level()===1,'default original Fun01');
            const rgb=game.frame(),painted=document.querySelector('#game').getContext('2d').getImageData(0,0,320,200).data;
            assert(rgb.every((v,i)=>painted[Math.floor(i/3)*4+i%3]===v),'Canvas differs');
            const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',rgb))].map(v=>v.toString(16).padStart(2,'0')).join('');
            assert(hash==='1539ca2afdcc9cfbe4fa5de51036c45b881014ec14a0ab030c191ca41f988c4d','original graphics differ');
            return {first_frame_sha256:hash,canvas:true};})()`);
        const capture = await command('Page.captureScreenshot', {format: 'png', captureBeyondViewport: false});
        fs.writeFileSync(path.join(output, 'original-fun01.png'), Buffer.from(capture.data, 'base64'));
        const levels = await evaluate(`(async()=>{
            const assert=(ok,m)=>{if(!ok)throw Error(m);},p=window.__pages,g=p.game;
            p.select=(rank,n)=>{document.querySelector('#difficulty').value=rank;document.querySelector('#level').value=n;
                document.querySelector('#open-level').click();assert(!document.querySelector('#error').textContent&&g.original()&&g.difficulty()===rank&&g.level()===n,'level selection');};
            let loaded=0;for(const rank of ['fun','tricky','taxing','mayhem'])for(let n=1;n<=30;n++){p.select(rank,n);loaded++;}
            p.select('fun',1);
            const {AudioHost}=await import('./audio.js'),{WebAudio}=await import('./pkg/lemmings_web.js');
            p.activate=AudioHost.prototype.activate;p.render=WebAudio.prototype.render;p.nonzero=0;
            AudioHost.prototype.activate=function(){p.host=this;return p.activate.apply(this,arguments);};
            WebAudio.prototype.render=function(){const pcm=p.render.apply(this,arguments);p.nonzero+=pcm.filter(v=>v!==0).length;return pcm;};
            g.step(60);return loaded;})()`);
        // A real browser input event supplies the sound activation gesture; no autoplay override.
        const box = await evaluate(`(()=>{document.querySelector('#sound').scrollIntoView({block:'center'});
            const r=document.querySelector('#sound').getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()`);
        for (const type of ['mousePressed', 'mouseReleased']) await command('Input.dispatchMouseEvent', {type, ...box, button: 'left', clickCount: 1});
        const audio = await evaluate(`(async()=>{const p=window.__pages;
            for(let n=0;n<100&&(!p.host?.submitted||!p.nonzero);n++){await new Promise(r=>setTimeout(r,20));p.host?.service();}
            if(p.host?.context?.state!=='running'||!p.nonzero)throw Error('Gesture did not activate original PCM');
            const result={state:p.host.context.state,submitted:p.host.submitted,nonzero:p.nonzero,real_gesture:true};
            p.host.mute();const {AudioHost}=await import('./audio.js'),{WebAudio}=await import('./pkg/lemmings_web.js');
            AudioHost.prototype.activate=p.activate;WebAudio.prototype.render=p.render;return result;})()`);
        const transitions = await evaluate(`(async()=>{
            const p=window.__pages,g=p.game,{WebGame,WebAssets}=await import('./pkg/lemmings_web.js'),{loadBundled}=await import('./bundled.js');
            const bank=await loadBundled(WebAssets),baseline=new WebGame('builder','reference'),results=[];
            try{for(const [action,target] of [['restart',1],['continue',2]]){
                p.select('fun',1);g.step(60);g.command('AssignDigger',0);g.step(1000);
                if(JSON.parse(g.state()).completion.phase!=='Won')throw Error('Fun01 saved solution failed');
                document.querySelector('#pause').click();document.querySelector('#'+action).click();
                baseline.load_original_rank(bank,'fun',target);const frame=baseline.frame();
                if(g.level()!==target||g.state()!==baseline.state()||!g.frame().every((v,i)=>v===frame[i]))throw Error('Result transition differs');
                results.push({action,target,status:'PASS'});
            }}finally{baseline.free();bank.free();p.detach();await p.host.dispose();}return results;})()`);
        await navigate('Page.reload', {ignoreCache: true}); await ready();
        assert(await evaluate("window.__pages===undefined"), 'Refresh reused test state');
        const result = {status: 'PASS', url, public_files_verified: http.length, ...initial,
            original_levels_loaded: levels, audio, transitions, refresh: true,
            scope: 'Published HTTPS payload; actual WASM/DOM/Canvas, real sound gesture/nonzero PCM; no physical audibility or performance qualification'};
        fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify(result, null, 2));
        console.log(JSON.stringify(result));
        await command('Page.removeScriptToEvaluateOnNewDocument', {identifier: clock.identifier});
        await navigate('Page.reload', {ignoreCache: true}); await ready();
        // Leave one ordinary, uninstrumented live game tab visible for the user.
    } finally {ws.close();}
})().catch(error => {console.error(error.stack); process.exitCode = 1;});
