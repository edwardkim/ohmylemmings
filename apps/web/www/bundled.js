// HTTP loading belongs to the host. WASM continues to accept flat asset bytes.
const names = [
    'ADLIB.DAT','MAIN.DAT','ODDTABLE.DAT','VGALEMMI.EXE',
    ...Array.from({length:10},(_,n)=>`LEVEL00${n}.DAT`),
    ...Array.from({length:5},(_,n)=>`GROUND${n}O.DAT`),
    ...Array.from({length:5},(_,n)=>`VGAGR${n}.DAT`),
    ...Array.from({length:4},(_,n)=>`VGASPEC${n}.DAT`),
];
const require=(condition,message)=>{if(!condition)throw Error(message);};

export async function loadBundled(Bank, fetcher=fetch, required=false) {
    const response=await fetcher('./GAME-DATA.json',{cache:'no-cache'});
    if(response.status===404&&!required)return null; // Unbundled source/development preview.
    require(response.ok,`GAME-DATA.json: HTTP ${response.status}`);
    const info=await response.json();
    require(info.original_game_data===true&&info.levels===120&&Array.isArray(info.files)&&info.files.length===names.length,
        'GAME-DATA.json: 게임 파일 목록이 올바르지 않습니다.');
    const found=new Set();
    for(const pin of info.files) {
        require(names.includes(pin.name)&&!found.has(pin.name)&&Number.isInteger(pin.size)&&pin.size>0&&pin.size<=1048576&&/^[a-f0-9]{64}$/.test(pin.sha256),
            'GAME-DATA.json: 게임 파일 정보가 올바르지 않습니다.');
        found.add(pin.name);
    }
    // Archive SHA-256 checks are performed by the packager/verifier. Size checks
    // and WASM parsers also work on LAN HTTP without secure-context crypto APIs.
    const contents=await Promise.all(info.files.map(async pin=>{
        const response=await fetcher('./data/'+pin.name,{cache:'no-cache'});
        require(response.ok,`${pin.name}: HTTP ${response.status}`);
        const bytes=new Uint8Array(await response.arrayBuffer());
        require(bytes.length===pin.size,`${pin.name}: 파일 크기가 다릅니다.`);
        return [pin.name,bytes];
    }));
    const bank=new Bank();
    try {for(const [name,bytes] of contents)bank.insert(name,bytes);return bank;}
    catch(error){bank.free();throw error;}
}
