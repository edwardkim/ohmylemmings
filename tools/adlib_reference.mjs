// Execute only pinned TS initialization/read method bodies; no TS/npm dependency.
// The Python caller verifies source size/SHA before this adapter runs.
import fs from 'node:fs';
import path from 'node:path';
const [folder,imageFile]=process.argv.slice(2);
const source=name=>fs.readFileSync(path.join(folder,name),'utf8');
function body(text,signature) {
    const at=text.indexOf(signature);if(at<0)throw new Error('missing pinned method '+signature);
    const begin=text.indexOf('{',at);let depth=1,end=begin+1;
    while(depth&&end<text.length){if(text[end]==='{')depth++;if(text[end]==='}')depth--;end++;}
    if(depth)throw new Error('unbalanced pinned method');
    return text.slice(begin+1,end-1).replace(/:\s*number\b/g,'').replace(/:\s*SoundImageChannels\b/g,'');
}
const readerSource=source('lemmings-ts__src__game__resources__file__binary-reader.ts');
const playerSource=source('lemmings-ts__src__game__resources__sound__sound-image-player.ts');
const channelSource=source('lemmings-ts__src__game__resources__sound__sound-image-channel.ts');
const readWord=new Function('offset',body(readerSource,'public readWordBE('));
const readByte=new Function('offset',body(readerSource,'public readByte('));
const setOffset=new Function('newPos',body(readerSource,'public setOffset('));
const initMusic=new Function('musicIndex',body(playerSource,'public initMusic('));
const initSound=new Function('soundIndex',body(playerSource,'public initSound('));
const enumBody=channelSource.match(/enum SoundImagChannelState\s*{([^}]+)}/)[1];
const states=Object.fromEntries(enumBody.split(',').map(s=>s.trim()).filter(Boolean).map((name,index)=>[name,index]));
const channelMusic=new Function('SoundImagChannelState','return function(){'+body(channelSource,'public initMusic(')+'}')(states);
const channelSound=new Function('SoundImagChannelState','return function(){'+body(channelSource,'public initSound(')+'}')(states);
const data=new Uint8Array(fs.readFileSync(imageFile));
const reader=()=>({data,pos:0,hiddenOffset:0,log:{log(){}},readWordBE(offset=-1){return readWord.call(this,offset)},readByte,setOffset});
const config=source('lemmings-ts__public__data__config.json');
const first=JSON.parse(config)[0];
const fileConfig=Object.fromEntries(Object.entries(first).filter(([k])=>k.startsWith('audio.')).map(([k,v])=>[k.slice(6),v]));
const configuredTracks=fileConfig.numberOfTracks;
fileConfig.numberOfTracks=21; // Pinned G00/Lemmix21-track contract, not TS's20-track bug.
function player() {
    return {reader:reader(),fileConfig,channels:[],debug(){},createChannel(){return {reader:reader(),fileConfig,initMusic:channelMusic,initSound:channelSound}}};
}
const music=[],effects=[];
for(let i=0;i<21;i++) {
    const p=player();initMusic.call(p,i);
    music.push({index:i,header_offset:p.songHeaderPosition,sample_rate_factor:p.sampleRateFactor,instrument_offset:p.instrumentPos,wait_cycles:p.waitCycles,channel_offsets:p.channels.map(c=>c.programPointer-2),first_bar_offsets:p.channels.map(c=>c.channelPosition)});
}
for(let i=0;i<18;i++) {const p=player();initSound.call(p,i);effects.push({index:i,data_offset:p.channels[0].channelPosition});}
console.log(JSON.stringify({music,effects,configured_tracks:configuredTracks,adopted_tracks:21,scope:'actual pinned initMusic/initSound/channel initMusic/readWordBE/readByte/setOffset method bodies; channel creation uses metadata-only stub; no command interpreter, OPL or PCM execution'}));
