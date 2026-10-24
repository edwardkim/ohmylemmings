import {DownloadProgress,readDownload} from './download.js';
export async function checkDownload(assert){
    const html=await(await fetch('./index.html')).text();
    const root=new DOMParser().parseFromString(html,'text/html').querySelector('#download');
    assert(root,'download UI is missing');document.body.append(root);
    try{
        const ui=new DownloadProgress(root),counts=[];
        const response=new Response(new ReadableStream({async start(c){c.enqueue(new Uint8Array(2));await new Promise(r=>setTimeout(r,20));c.enqueue(new Uint8Array(3));c.close();}}));
        const bytes=await readDownload(response,5,n=>{counts.push(n);ui.update({phase:'foreground',status:'loading',loaded:n,total:5,cached:0});});
        assert(bytes.length===5&&counts.join(',')==='2,5','byte progress lost streamed chunks');
        ui.update({phase:'foreground',status:'ready',loaded:5,total:5,cached:0});
        assert(!root.hidden&&root.querySelector('progress').value===5&&root.querySelector('output').textContent.includes('100%'),'completion progress UI');
        assert(root.querySelector('label').htmlFor===root.querySelector('progress').id&&root.getAttribute('aria-busy')==='false','download accessibility');
        let retries=0;ui.update({phase:'foreground',status:'error',loaded:2,total:5,cached:0},()=>retries++);
        assert(root.querySelector('progress').value===2&&!root.querySelector('button').hidden,'failed progress was reset or retry missing');root.querySelector('button').click();assert(retries===1,'retry did not invoke exactly once');
        ui.update({phase:'background',status:'ready',loaded:5,total:5,cached:5});assert(root.dataset.phase==='background'&&root.querySelector('p').textContent.includes('다음 레벨'),'background UX missing');
        let bounded=false;try{await readDownload(new Response(new Uint8Array(6)),5);}catch{bounded=true;}assert(bounded,'oversized resource accepted');
        return {status:'PASS',streamed_bytes:counts,percent:true,accessible_progress:true,retry:true,background_status:true,bounded_download:true};
    }finally{root.remove();}
}
