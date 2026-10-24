import {readDownload} from './download.js';
const MAX=128*1024*1024,PREFIX='ohmylemmings-hd-v1-';
const table=Uint32Array.from({length:256},(_,i)=>{for(let n=0;n<8;n++)i=(i>>>1)^((i&1)?0xedb88320:0);return i>>>0;});
function crc(data){let v=0xffffffff;for(const b of data)v=(v>>>8)^table[(v^b)&255];return (v^0xffffffff)>>>0;}
function require(ok,message){if(!ok)throw Error(message);}
function validKey(k){return Array.isArray(k)&&k.length===3&&k.every(Number.isInteger)&&k[0]>=0&&k[0]<=4&&k[1]>=0&&k[1]<[28,5,80,13,4][k[0]]&&k[2]>=0&&k[2]<256;}
export function validateWebHd(info){
    require(info?.schema_version===1&&info.codec==='gzip'&&/^[a-f0-9]{64}$/.test(info.catalog_sha256)&&/^[a-f0-9]{64}$/.test(info.transport_sha256)&&Array.isArray(info.pages)&&info.pages.length>0&&info.pages.length<=128,'Invalid Web HD manifest');
    const keys=new Set(),paths=new Set();let raw=0,wire=0,actors=0;
    for(const row of info.pages){
        require(/^[a-f0-9]{64}$/.test(row.sha256)&&row.path===`web-hd/${row.sha256}.hdp.gz`&&!paths.has(row.path),'Invalid Web HD path');paths.add(row.path);
        require(Number.isInteger(row.bytes)&&row.bytes>=64&&row.bytes<=MAX&&Number.isInteger(row.download_bytes)&&row.download_bytes>0&&row.download_bytes<=16*1024*1024&&Number.isInteger(row.crc32)&&row.crc32>=0&&row.crc32<=0xffffffff&&Number.isInteger(row.wire_crc32)&&row.wire_crc32>=0&&row.wire_crc32<=0xffffffff,'Invalid Web HD bounds');
        require(Array.isArray(row.keys)&&row.keys.length>0&&row.keys.length<=4096,'Invalid Web HD keys');
        for(const k of row.keys){require(validKey(k)&&!keys.has(k.join(',')),'Invalid or duplicate Web HD key');keys.add(k.join(','));actors+=k[0]===0;}
        raw+=row.bytes;wire+=row.download_bytes;
    }
    require(raw<=512*1024*1024&&wire<=MAX&&keys.size===info.logical_keys&&actors===info.actor_frames&&info.glyphs===95,'Invalid Web HD inventory');return info;
}
export function selectWebHd(info,plan){
    require(Array.isArray(plan?.keys)&&plan.keys.length>0&&plan.keys.length<=128,'Invalid level download plan');
    const needed=new Set();for(const k of plan.keys){require(Array.isArray(k)&&k.length===2&&validKey([...k,0]),'Invalid level download key');needed.add(k.join(','));}
    const rows=info.pages.filter(row=>row.keys.some(k=>needed.has(k.slice(0,2).join(','))));
    const covered=new Set(rows.flatMap(row=>row.keys.map(k=>k.slice(0,2).join(','))));
    require([...needed].every(k=>covered.has(k))&&rows.length>0&&rows.length<=32&&12+rows.reduce((n,r)=>n+4+r.bytes,0)<=MAX,'Incomplete or excessive level HD assets');return rows;
}
export class ProgressiveHdLoader {
    constructor({fetcher=(...args)=>globalThis.fetch(...args),cacheStorage=globalThis.caches,onProgress=()=>{}}={}){
        this.fetcher=fetcher;this.storage=cacheStorage;this.onProgress=onProgress;this.info=null;this.cache=null;this.memory=new Map();this.memoryBytes=0;this.generation=0;this.foreground=null;this.background=null;
    }
    cancelBackground(){this.background?.abort();this.background=null;}
    cancel(){this.cancelBackground();this.foreground?.abort();this.foreground=null;this.generation++;}
    async initialize(signal){
        if(this.info)return;
        const bytes=await readDownload(await this.fetcher('./HD-WEB.json',{signal,cache:'no-cache'}),128*1024,()=>{},signal);
        const info=validateWebHd(JSON.parse(new TextDecoder().decode(bytes)));
        let cache=null;
        try{
            const name=PREFIX+info.transport_sha256;cache=await this.storage?.open(name);
            for(const old of await this.storage?.keys()||[])if(old.startsWith(PREFIX)&&old!==name)await this.storage.delete(old);
        }catch{/* Storage is an optional optimization. */}
        signal.throwIfAborted();this.info=info;this.cache=cache;
    }
    emit(token,event){if(token===this.generation)this.onProgress(event);}
    remember(path,bytes){
        if(this.memory.has(path))return;
        while(this.memoryBytes+bytes.length>64*1024*1024&&this.memory.size){const [key,b]=this.memory.entries().next().value;this.memory.delete(key);this.memoryBytes-=b.length;}
        this.memory.set(path,bytes);this.memoryBytes+=bytes.length;
    }
    async verify(row,bytes){
        require(bytes.length===row.download_bytes,'HD download checksum differs');
        if(globalThis.crypto?.subtle){const digest=await crypto.subtle.digest('SHA-256',bytes);require([...new Uint8Array(digest)].map(n=>n.toString(16).padStart(2,'0')).join('')===row.sha256,'HD download digest differs');}
        else require(crc(bytes)===row.wire_crc32,'HD download checksum differs');
    }
    async forget(path){const old=this.memory.get(path);if(old){this.memoryBytes-=old.length;this.memory.delete(path);}try{await this.cache?.delete('./'+path);}catch{}}
    async wire(row,signal,priority,progress){
        let bytes=this.memory.get(row.path);
        if(!bytes){try{const response=await this.cache?.match('./'+row.path);if(response)bytes=await readDownload(response,row.download_bytes,()=>{},signal);}catch(error){signal.throwIfAborted();await this.forget(row.path);}}
        if(bytes){try{await this.verify(row,bytes);signal.throwIfAborted();progress(bytes.length,true);return bytes;}catch{signal.throwIfAborted();await this.forget(row.path);}}
        const timer=setTimeout(()=>{const controller=this.foreground?.signal===signal?this.foreground:this.background?.signal===signal?this.background:null;controller?.abort(new DOMException('Download timed out','TimeoutError'));},120000);
        try{bytes=await readDownload(await this.fetcher('./'+row.path,{signal,priority}),row.download_bytes,n=>progress(n,false),signal);await this.verify(row,bytes);signal.throwIfAborted();}
        finally{clearTimeout(timer);}
        this.remember(row.path,bytes);
        try{await this.cache?.put('./'+row.path,new Response(bytes));}catch{/* Quota/private mode may disable persistent caching. */}
        return bytes;
    }
    async inflate(row,wire,signal){
        require(typeof DecompressionStream==='function','This browser cannot decode HD assets');
        const stream=new Blob([wire]).stream().pipeThrough(new DecompressionStream('gzip'));
        const raw=await readDownload(new Response(stream),row.bytes,()=>{},signal);
        require(raw.length===row.bytes&&crc(raw)===row.crc32,'HD decoded checksum differs');return raw;
    }
    async load(game){
        this.cancel();const controller=new AbortController(),signal=controller.signal,token=this.generation;this.foreground=controller;
        let total=null,loaded=[],cached=[];
        const event=status=>({phase:'foreground',status,loaded:loaded.reduce((a,b)=>a+b,0),cached:cached.reduce((a,b)=>a+b,0),total});
        this.emit(token,event('loading'));
        try{
            await this.initialize(signal);const rows=selectWebHd(this.info,JSON.parse(game.hd_download_plan(false)));
            total=rows.reduce((n,r)=>n+r.download_bytes,0);loaded=rows.map(()=>0);cached=rows.map(()=>0);this.emit(token,event('loading'));
            const parts=Array(rows.length);let cursor=0;
            const worker=async()=>{try{while(cursor<rows.length){const i=cursor++,row=rows[i];const wire=await this.wire(row,signal,'high',(n,hit)=>{loaded[i]=n;cached[i]=hit?n:0;this.emit(token,event('loading'));});try{parts[i]=await this.inflate(row,wire,signal);}catch(error){await this.forget(row.path);throw error;}}}catch(error){controller.abort(error);throw error;}};
            const results=await Promise.allSettled([worker(),worker()]);const failed=results.find(r=>r.status==='rejected');if(failed)throw failed.reason;
            signal.throwIfAborted();require(token===this.generation,'Stale HD request');
            const bytes=new Uint8Array(12+parts.reduce((n,b)=>n+4+b.length,0)),view=new DataView(bytes.buffer);bytes.set(new TextEncoder().encode('LHDSET01'));view.setUint16(8,parts.length,true);let at=12;
            for(const part of parts){view.setUint32(at,part.length,true);bytes.set(part,at+4);at+=4+part.length;}
            game.load_hd(bytes);this.emit(token,event('ready'));
        }catch(error){controller.abort();if(token===this.generation)this.emit(token,event('error'));throw error;}
        finally{if(this.foreground===controller)this.foreground=null;}
    }
    async prefetch(game){
        if(this.foreground||!this.info)return;
        this.cancelBackground();const controller=new AbortController(),signal=controller.signal,token=++this.generation;this.background=controller;
        let total=null,loaded=[],cached=[];
        const event=status=>({phase:'background',status,loaded:loaded.reduce((a,b)=>a+b,0),cached:cached.reduce((a,b)=>a+b,0),total});
        try{
            const plan=JSON.parse(game.hd_download_plan(true));if(!plan)return;
            const rows=selectWebHd(this.info,plan);total=rows.reduce((n,r)=>n+r.download_bytes,0);loaded=rows.map(()=>0);cached=rows.map(()=>0);this.emit(token,event('loading'));
            for(let i=0;i<rows.length;i++){await this.wire(rows[i],signal,'low',(n,hit)=>{loaded[i]=n;cached[i]=hit?n:0;this.emit(token,event('loading'));});}
            signal.throwIfAborted();this.emit(token,event('ready'));
        }catch(error){if(!signal.aborted||signal.reason?.name==='TimeoutError')this.emit(token,event('error'));}
        finally{if(this.background===controller)this.background=null;}
    }
}
