// Offline host-loader checks; fake bytes never enter the original parsers.
export async function checkBundled(loadBundled, assert) {
    const names=['ADLIB.DAT','MAIN.DAT','ODDTABLE.DAT','VGALEMMI.EXE',
        ...Array.from({length:10},(_,i)=>`LEVEL00${i}.DAT`),
        ...Array.from({length:5},(_,i)=>`GROUND${i}O.DAT`),
        ...Array.from({length:5},(_,i)=>`VGAGR${i}.DAT`),
        ...Array.from({length:4},(_,i)=>`VGASPEC${i}.DAT`)];
    let freed=0,created=0,mode='valid';
    class Bank {constructor(){created++;this.files=[];}insert(name,bytes){if(mode==='insert')throw Error('insert');this.files.push([name,bytes]);}free(){freed++;}}
    const info=()=>({original_game_data:true,levels:120,files:names.map(name=>({name,size:1,sha256:'0'.repeat(64)}))});
    const fetcher=async path=>{
        if(path==='./GAME-DATA.json') {
            if(mode==='absent')return {status:404};
            if(mode==='network')throw Error('offline');
            const data=info();
            if(mode==='duplicate')data.files[1]=data.files[0];
            if(mode==='path')data.files[0].name='../MAIN.DAT';
            if(mode==='huge')data.files[0].size=1048577;
            return {ok:true,json:async()=>data};
        }
        return {ok:mode!=='missing',status:404,arrayBuffer:async()=>new Uint8Array(mode==='truncated'?0:1).buffer};
    };
    mode='absent';assert(await loadBundled(Bank,fetcher)===null&&created===0,'unbundled preview changed');
    let missingManifest=false;try{await loadBundled(Bank,fetcher,true);}catch{missingManifest=true;}
    assert(missingManifest,'packaged game silently fell back when manifest was missing');
    mode='valid';const bank=await loadBundled(Bank,fetcher);assert(bank.files.length===28,'bundle did not load all files');bank.free();
    for(mode of ['network','duplicate','path','huge','missing','truncated','insert']) {
        let failed=false;try{await loadBundled(Bank,fetcher);}catch{failed=true;}
        assert(failed,'invalid bundle accepted: '+mode);
    }
    assert(created===freed,'failed loader leaked an asset bank');
    return {status:'PASS',invalid_cases:8,all_files:28,unbundled_preview:true};
}
