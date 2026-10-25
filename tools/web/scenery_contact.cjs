// Capture all audited source-art cards using a disposable loopback QA browser.
const fs=require('fs'),path=require('path');
const [base,cdp,output]=process.argv.slice(2),out=path.resolve(output);
if(!out.startsWith(path.resolve('assets/local')+path.sep)||fs.existsSync(out))throw Error('new ignored output required');
fs.mkdirSync(out,{recursive:true});
(async()=>{
    const tabs=await(await fetch(cdp+'/json/list')).json();
    const target=tabs.find(t=>t.type==='page'&&t.url==='about:blank');
    if(!target||tabs.filter(t=>t.type==='page').length!==1)throw Error('disposable blank browser required');
    const ws=new WebSocket(target.webSocketDebuggerUrl);
    await new Promise(resolve=>ws.addEventListener('open',resolve,{once:true}));
    let id=0;const pending=new Map();
    ws.onmessage=event=>{const m=JSON.parse(event.data);if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(Error(JSON.stringify(m.error))):p.resolve(m.result);}};
    const cmd=(method,params={})=>new Promise((resolve,reject)=>{pending.set(++id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
    try {
        await cmd('Page.enable');await cmd('Runtime.enable');
        await cmd('Emulation.setDeviceMetricsOverride',{width:1100,height:1000,deviceScaleFactor:1,mobile:false});
        await cmd('Page.navigate',{url:base});
        for(let n=0;;n++){
            const r=await cmd('Runtime.evaluate',{expression:'document.body?.dataset.ready === "true"',returnByValue:true});
            if(r.result.value)break;
            if(n>1200)throw Error('contact sheet timeout');
            await new Promise(resolve=>setTimeout(resolve,100));
        }
        const layout=await cmd('Page.getLayoutMetrics'),height=Math.ceil(layout.cssContentSize.height);
        for(let y=0;y<height;y+=1000){
            const shot=await cmd('Page.captureScreenshot',{format:'png',captureBeyondViewport:true,
                clip:{x:0,y,width:1100,height:Math.min(1000,height-y),scale:1}});
            fs.writeFileSync(path.join(out,`contact-${y/1000}.png`),Buffer.from(shot.data,'base64'));
        }
        console.log(JSON.stringify({status:'PASS',height,pages:Math.ceil(height/1000)}));
    } finally {await cmd('Browser.close').catch(()=>{});ws.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
