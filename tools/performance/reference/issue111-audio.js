// Bounded AudioBufferSource scheduling works on LAN HTTP without a worklet.
// Rust owns decoding/OPL/PCM; this adapter owns activation, nodes and device time.
export class AudioHost {
    constructor(getGame, WebAudio, changed = () => {}, active = () => true,
        createContext = () => new globalThis.AudioContext({latencyHint:'interactive'})) {
        this.getGame=getGame;this.WebAudio=WebAudio;this.changed=changed;this.active=active;this.createContext=createContext;
        this.context=null;this.pipe=null;this.sources=new Set();this.nextTime=0;
        this.muted=true;this.held=false;this.starting=false;this.volume=40;this.ticket=0;this.timer=null;
        this.submitted=0;this.underruns=0;this.error=null;
        this.notify('off');
    }
    notify(state) { this.changed({state,volume:this.volume,muted:this.muted,error:this.error}); }
    clear() {
        for(const source of this.sources) {source.onended=null;try{source.stop();}catch{}try{source.disconnect();}catch{}}
        this.sources.clear();this.nextTime=0;
    }
    stopTimer() {if(this.timer!==null)clearInterval(this.timer);this.timer=null;}
    async activate() {
        const ticket=++this.ticket;this.muted=false;this.error=null;
        if(this.held){this.notify('held');return;}
        this.starting=true;this.notify('starting');
        try {
            if(!this.context||this.context.state==='closed') {
                this.pipe?.free();this.pipe=null;
                this.context=this.createContext();
                this.context.onstatechange=()=>{
                    if(!this.muted&&!this.held&&!this.starting&&this.context.state!=='running')this.failure(new Error('audio context interrupted'));
                };
            }
            const context=this.context;
            // resume is invoked synchronously inside the user's click handler.
            // A blocked context may leave its promise pending; allow a retry.
            let timeout;
            try {await Promise.race([context.resume(),new Promise((_,reject)=>{timeout=setTimeout(()=>reject(new Error('activation timeout')),3000);})]);}
            finally {clearTimeout(timeout);}
            if(ticket!==this.ticket||this.muted||this.held||context!==this.context)return;
            if(context.state!=='running')throw new Error('audio context suspended');
            if(!this.pipe)this.pipe=new this.WebAudio(context.sampleRate);
            this.pipe.set_volume(this.volume);this.pipe.suspend();this.clear();
            this.starting=false;
            this.service();
            if(!this.muted) {
                this.stopTimer();this.timer=setInterval(()=>this.service(),20);this.notify('on');
            }
        } catch(error) {if(ticket===this.ticket)this.failure(error);}
    }
    mute() {
        ++this.ticket;this.muted=true;this.starting=false;this.stopTimer();this.clear();this.pipe?.suspend();
        this.context?.suspend().catch(()=>{});this.notify('off');
    }
    hold(value) {
        if(this.held===value)return;
        this.held=value;
        if(value) {++this.ticket;this.starting=false;this.stopTimer();this.clear();this.pipe?.suspend();this.context?.suspend().catch(()=>{});if(!this.muted)this.notify('held');}
        else if(!this.muted)void this.activate();
    }
    setVolume(value) {
        if(!Number.isInteger(value)||value<0||value>100)throw new Error('volume must be in 0..=100');
        this.pipe?.set_volume(value);this.volume=value;this.clear();
        this.notify(this.muted?'off':this.held?'held':this.starting?'starting':'on');
    }
    failure(error) {
        ++this.ticket;this.error=String(error);this.muted=true;this.starting=false;
        this.stopTimer();this.clear();this.pipe?.suspend();this.context?.suspend().catch(()=>{});this.notify('error');
    }
    service() {
        if(!this.pipe||!this.context||this.starting||this.muted||this.held)return;
        try {
            const audible=this.active()&&this.context.state==='running';
            if(this.nextTime&&this.nextTime<this.context.currentTime-0.15) {
                this.underruns++;this.clear();this.pipe.suspend();
            }
            if(this.pipe.prepare(this.getGame(),audible))this.clear();
            if(!audible){this.clear();return;}
            const now=this.context.currentTime;
            // Each call schedules at most eight small mono buffers. Neither
            // event-loop stalls nor device clock jumps accumulate playback debt.
            for(let n=0;n<8&&this.sources.size<8;n++) {
                const at=Math.max(this.nextTime,now+0.005);
                if(at>=now+0.06)break;
                const pcm=this.pipe.render(512);
                const buffer=this.context.createBuffer(1,pcm.length,this.context.sampleRate);
                const samples=buffer.getChannelData(0);
                for(let i=0;i<pcm.length;i++)samples[i]=pcm[i]/32768;
                const source=this.context.createBufferSource();source.buffer=buffer;
                source.connect(this.context.destination);source.onended=()=>{source.disconnect();this.sources.delete(source);};
                this.sources.add(source);source.start(at);
                this.nextTime=at+pcm.length/this.context.sampleRate;this.submitted+=pcm.length;
            }
        }catch(error){this.failure(error);}
    }
    async dispose() {
        this.mute();this.pipe?.free();this.pipe=null;
        const context=this.context;this.context=null;
        if(context){context.onstatechange=null;try{await context.close();}catch{}}
    }
}
