// Fetch only the current level's immutable atlas pages. One current set is cached.
const MAX=128*1024*1024;
const table=Uint32Array.from({length:256},(_,i)=>{for(let n=0;n<8;n++)i=(i>>>1)^((i&1)?0xedb88320:0);return i>>>0;});
function crc(data){let v=0xffffffff;for(const b of data)v=(v>>>8)^table[(v^b)&255];return (v^0xffffffff)>>>0;}
export function catalogEntries(bytes,scope){
    const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
    if(bytes.length<12||new TextDecoder().decode(bytes.subarray(0,8))!=='LHDIDX01')throw Error('Invalid HD catalog');
    const count=v.getUint16(8,true);
    if(count<1||count>128||v.getUint16(10,true)||bytes.length!==12+count*12)throw Error('Invalid HD catalog bounds');
    if(!Number.isInteger(scope)||scope<0||scope>1023||!(scope&1)||!(scope&62)||((scope&62)&((scope&62)-1))||((scope&960)&((scope&960)-1)))throw Error('Invalid HD level scope');
    const rows=[];let total=0,selected=12;
    for(let index=0;index<count;index++){
        const at=12+index*12,mask=v.getUint16(at,true),size=v.getUint32(at+4,true),checksum=v.getUint32(at+8,true);
        if(!mask||mask>1023||v.getUint16(at+2,true)||size<64||size>MAX)throw Error('Invalid HD catalog entry');
        total+=size;if(mask&scope){rows.push({index,size,checksum});selected+=4+size;}
    }
    if(total>512*1024*1024||!rows.length||rows.length>32||selected>MAX)throw Error('HD selection exceeds limits');
    return rows;
}
async function bounded(response,limit){
    if(!response.ok)throw Error('HD asset download failed');
    const declared=Number(response.headers.get('content-length'));
    if(Number.isFinite(declared)&&declared>limit)throw Error('HD download exceeds limit');
    const reader=response.body?.getReader();
    if(!reader){const b=new Uint8Array(await response.arrayBuffer());if(b.length>limit)throw Error('HD download exceeds limit');return b;}
    const chunks=[];let size=0;
    try {while(true){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>limit)throw Error('HD download exceeds limit');chunks.push(value);}}
    catch(error){await reader.cancel().catch(()=>{});throw error;}
    const bytes=new Uint8Array(size);let at=0;for(const chunk of chunks){bytes.set(chunk,at);at+=chunk.length;}return bytes;
}
export class HdLoader {
    constructor(fetcher=(...args)=>globalThis.fetch(...args)){this.fetcher=fetcher;this.catalog=null;this.cache=new Map();this.legacy=null;}
    async load(game){
        if(!this.catalog&&!this.legacy){
            const response=await this.fetcher('./hd-assets/catalog.hdi');
            if(response.status===404){this.legacy=await bounded(await this.fetcher('./hd-assets.hdp'),MAX);}
            else this.catalog=await bounded(response,12+128*12);
        }
        if(this.legacy){game.load_hd(this.legacy);return;}
        const rows=catalogEntries(this.catalog,game.hd_scope()),next=new Map();let size=12;
        for(const row of rows){
            let part=this.cache.get(row.index);
            if(!part){part=await bounded(await this.fetcher(`./hd-assets/atlas-${String(row.index).padStart(3,'0')}.hdp`),row.size);if(part.length!==row.size||crc(part)!==row.checksum)throw Error('HD atlas checksum differs');}
            next.set(row.index,part);size+=4+part.length;
        }
        const bytes=new Uint8Array(size),view=new DataView(bytes.buffer);bytes.set(new TextEncoder().encode('LHDSET01'));view.setUint16(8,rows.length,true);let at=12;
        for(const part of next.values()){view.setUint32(at,part.length,true);at+=4;bytes.set(part,at);at+=part.length;}
        game.load_hd(bytes); // Rust validates the complete set before committing.
        this.cache=next;
    }
}
