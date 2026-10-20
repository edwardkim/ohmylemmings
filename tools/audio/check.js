import init, {audio_pcm} from './pkg/audio_probe.js';
const sha = async bytes => [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(x=>x.toString(16).padStart(2,'0')).join('');
try {
    await init();
    const expected=await (await fetch('./expected.json')).json();
    const images=new Map();const cases=[];
    for (const test of expected.cases) {
        if (!images.has(test.image)) images.set(test.image,new Uint8Array(await (await fetch(test.image)).arrayBuffer()));
        let info;
        for (const partitions of [[8192],[1,31,1024,7,8192]]) {
            const samples=audio_pcm(images.get(test.image),test.rate,test.num,test.den,test.actions,new Uint32Array(partitions));
            const bytes=new Uint8Array(samples.length*2);const view=new DataView(bytes.buffer);
            for(let i=0;i<samples.length;i++)view.setInt16(i*2,samples[i],true);
            info={name:test.name,frames:samples.length,sha256:await sha(bytes)};
            if(info.frames!==test.expected.frames || info.sha256!==test.expected.sha256)throw new Error(`PCM mismatch: ${test.name}`);
        }
        cases.push(info);
    }
    await fetch('/__result',{method:'POST',body:JSON.stringify({status:'PASS',cases})});
} catch(error) {
    await fetch('/__result',{method:'POST',body:JSON.stringify({status:'FAIL',error:String(error)})});
}
