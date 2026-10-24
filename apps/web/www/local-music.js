// User-selected files stay in this tab. No fetch, upload, persistent file copy,
// full-file ArrayBuffer or WASM decode is used for long recordings.
export class LocalMusic {
    constructor(getContext, changed=()=>{}, {
        createMedia=()=>new Audio(), createURL=file=>URL.createObjectURL(file),
        revokeURL=url=>URL.revokeObjectURL(url),
    }={}) {
        Object.assign(this,{getContext,changed,createMedia,createURL,revokeURL});
        this.media=null;this.url=null;this.source=null;this.gain=null;this.context=null;
        this.state='empty';this.name='';this.error='';this.volume=100;this.masterVolume=100;
        this.desired=false;this.generation=null;this.ticket=0;this.pending=false;
    }
    notify(state,error='') {
        this.state=state;this.error=error;
        this.changed({state,name:this.name,error});
    }
    release() {
        ++this.ticket;this.pending=false;
        this.media?.pause();
        if(this.media){this.media.removeAttribute('src');this.media.load();}
        this.source?.disconnect();this.gain?.disconnect();
        if(this.url)this.revokeURL(this.url);
        this.media=null;this.source=null;this.gain=null;this.context=null;this.url=null;
    }
    choose(file) {
        if(!file||!(/\.mp3$/i).test(file.name)||!Number.isSafeInteger(file.size)||file.size<=0||file.size>512*1024*1024)
            throw new Error('512MB 이하의 MP3 파일을 선택해주세요.');
        this.release();this.generation=null;
        this.install(this.createURL(file),file.name);
    }
    install(url,name) {
        const media=this.createMedia();this.media=media;this.name=name;
        const current=()=>this.media===media;
        media.loop=true;media.preload='metadata';
        media.addEventListener('canplay',()=>{if(current()){this.notify('ready');if(this.desired)this.start();}});
        media.addEventListener('waiting',()=>{if(current()&&this.desired)this.notify('loading');});
        media.addEventListener('playing',()=>{if(current()&&this.desired)this.notify('playing');});
        media.addEventListener('error',()=>{if(current()){++this.ticket;this.pending=false;media.pause();this.notify('error','이 MP3를 재생하지 못했습니다. 다른 파일을 선택하거나 다시 시도해주세요.');}});
        this.notify('loading');
        this.url=url;media.src=url;media.load();
    }
    setVolume(value) {
        if(!Number.isInteger(value)||value<0||value>100)throw new Error('volume must be in 0..=100');
        this.volume=value;this.applyGain();
    }
    setMasterVolume(value) {
        if(!Number.isInteger(value)||value<0||value>100)throw new Error('volume must be in 0..=100');
        this.masterVolume=value;this.applyGain();
    }
    applyGain(){if(this.gain)this.gain.gain.value=this.volume*this.masterVolume/10000;}
    stop() {
        ++this.ticket;this.pending=false;
        if(this.media){this.media.pause();try{this.media.currentTime=0;}catch{}}
        if(this.media&&this.state!=='error')this.notify('paused');
    }
    sync(active,generation,playing) {
        const changed=this.generation!==generation;
        this.generation=generation;
        const desired=Boolean(active&&playing);
        if(changed||(this.desired&&!desired))this.stop();
        const start=desired&&(!this.desired||changed);
        this.desired=desired;
        if(start)this.start();
    }
    start() {
        const media=this.media,context=this.getContext();
        if(!media||!this.desired||!context||this.pending||this.state==='error'||this.state==='blocked')return;
        if(!this.source) {
            try {
                this.source=context.createMediaElementSource(media);
                this.gain=context.createGain();this.context=context;
                this.applyGain();
                this.source.connect(this.gain);this.gain.connect(context.destination);
            } catch(error) {this.notify('error',String(error));return;}
        }
        if(media.readyState<2)this.notify('loading');
        const ticket=++this.ticket;this.pending=true;
        let attempt;
        try {attempt=media.play();}catch(error){attempt=Promise.reject(error);}
        Promise.resolve(attempt).then(()=>{
            if(this.media!==media)return;
            if(ticket!==this.ticket||!this.desired){if(!this.desired)media.pause();return;}
            this.pending=false;this.notify('playing');
        },()=>{
            if(this.media!==media||ticket!==this.ticket)return;
            this.pending=false;media.pause();this.notify('blocked','음악 재생 버튼을 눌러 다시 시도해주세요.');
        });
    }
    retry() {
        if(!this.media)return;
        const wasError=this.state==='error';this.notify('ready');
        if(wasError)this.media.load();
        this.start();
    }
    // A closed AudioContext cannot be reattached to the same media source.
    resetContext() {
        if(!this.media)return;
        const url=this.url,name=this.name,desired=this.desired;
        this.url=null;this.release();
        // Reuse the still-owned Blob URL; no file bytes are copied.
        this.install(url,name);this.desired=desired;
    }
    dispose(){this.desired=false;this.release();this.name='';this.notify('empty');}
}
