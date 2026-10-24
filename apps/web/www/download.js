// Counts response bytes, including streamed responses without Content-Length.
export async function readDownload(response,limit,onBytes=()=>{},signal=null) {
    if(!response.ok)throw Error('Resource download failed');
    const reader=response.body?.getReader();
    if(!reader){const bytes=new Uint8Array(await response.arrayBuffer());if(bytes.length>limit)throw Error('Resource exceeds limit');onBytes(bytes.length);return bytes;}
    const chunks=[];let size=0;
    const abort=()=>void reader.cancel().catch(()=>{});
    signal?.addEventListener('abort',abort,{once:true});
    try {
        for(;;){
            signal?.throwIfAborted();
            const {done,value}=await reader.read();signal?.throwIfAborted();if(done)break;
            size+=value.length;if(size>limit)throw Error('Resource exceeds limit');chunks.push(value);onBytes(size);
        }
        const bytes=new Uint8Array(size);let at=0;for(const chunk of chunks){bytes.set(chunk,at);at+=chunk.length;}return bytes;
    }catch(error){await reader.cancel().catch(()=>{});throw error;}
    finally{signal?.removeEventListener('abort',abort);reader.releaseLock();}
}

export class DownloadProgress {
    constructor(root){this.root=root;this.retry=null;this.last=0;this.pending=null;this.timer=null;
        root.querySelector('button').addEventListener('click',()=>this.retry?.());}
    update(event,retry=null){
        this.event=event;
        this.pending={event,retry};
        if(event.status!=='loading'||performance.now()-this.last>150||!this.timer)this.paint();
        if(event.status==='loading'&&!this.timer)this.timer=setTimeout(()=>{this.timer=null;if(this.pending)this.paint();},150);
    }
    paint(){
        if(!this.pending)return;const {event:e,retry}=this.pending;this.pending=null;this.last=performance.now();
        this.root.hidden=false;this.root.dataset.phase=e.phase;this.root.dataset.status=e.status;
        const title=this.root.querySelector('label'),bar=this.root.querySelector('progress'),text=this.root.querySelector('output'),hint=this.root.querySelector('p'),button=this.root.querySelector('button');
        title.textContent=e.label||(e.phase==='background'?'다음 레벨 준비':'플레이 준비');
        const mb=n=>(n/1000000).toFixed(1)+' MB';
        if(Number.isFinite(e.total)&&e.total>0){bar.max=e.total;bar.value=Math.min(e.loaded,e.total);text.textContent=`${Math.floor(Math.min(e.loaded,e.total)*100/e.total)}% · ${mb(e.loaded)} / ${mb(e.total)}`;}
        else if(e.status==='ready'){bar.max=1;bar.value=1;text.textContent='완료';}
        else {bar.removeAttribute('value');text.textContent=e.loaded?mb(e.loaded):'연결 중…';}
        hint.textContent=e.status==='error'?(e.phase==='background'?'다음 레벨 준비를 마치지 못했습니다. 현재 게임은 계속할 수 있습니다.':'파일을 받지 못했습니다. 연결 상태를 확인한 뒤 다시 시도해주세요.'):
            e.status==='ready'?(e.phase==='background'?'다음 레벨을 시작할 준비가 되었습니다.':'준비가 끝났습니다.'):
            e.phase==='background'?'플레이하면서 다음 레벨을 미리 준비합니다.':e.cached?`저장된 리소스 ${mb(e.cached)}를 재사용하고 있습니다.`:'필요한 리소스가 준비되면 게임을 진행합니다.';
        this.retry=retry;button.hidden=e.status!=='error'||!retry;button.disabled=false;
        this.root.setAttribute('aria-busy',String(e.status==='loading'));
    }
}
