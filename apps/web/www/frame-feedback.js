// Opt-in, bounded observations of callback cadence, separate from game ticks.
export class FrameSamples {
    constructor() {
        this.gaps=new Float64Array(240);
        this.draws=new Float64Array(240);
        this.reset();
    }
    reset() {
        this.last=null;this.count=0;this.cursor=0;this.elapsed=0;
        this.intervals=0;this.slow=0;
    }
    hold() {this.last=null;}
    frame(now,draws=0) {
        if(!Number.isFinite(now)||now<0||(this.last!==null&&now<this.last)||!Number.isInteger(draws)||draws<0)throw Error('invalid frame observation');
        const previous=this.last;this.last=now;
        if(previous===null||now===previous)return;
        const gap=now-previous;
        this.gaps[this.cursor]=gap;this.draws[this.cursor]=draws;
        this.cursor=(this.cursor+1)%this.gaps.length;
        this.count=Math.min(this.count+1,this.gaps.length);
        this.elapsed+=gap;this.intervals++;if(gap>50)this.slow++;
    }
    summary() {
        const gaps=Array.from(this.gaps.subarray(0,this.count)).sort((a,b)=>a-b);
        const elapsed=gaps.reduce((a,b)=>a+b,0);
        const draws=this.draws.subarray(0,this.count).reduce((a,b)=>a+b,0);
        return {fps:elapsed?this.count*1000/elapsed:null,draws_per_second:elapsed?draws*1000/elapsed:null,
            p95_ms:gaps.length?gaps[Math.ceil(gaps.length*.95)-1]:null,max_ms:gaps.length?gaps[gaps.length-1]:null,
            recent_intervals:this.count,intervals:this.intervals,active_seconds:this.elapsed/1000,slow_intervals:this.slow};
    }
}

export function feedbackText(context,summary) {
    // Explicit allowlist: no selected file names, URLs or arbitrary game state.
    const {version,platform,level,presentation,viewport,runtime,measurement}=context;
    return JSON.stringify({feedback_version:1,version,platform,level,presentation,viewport,runtime,measurement,
        cadence:'rAF callback entry; not physical display FPS',window:'last 240 active intervals',
        slow_threshold_ms:50,summary,체감:'부드러움 / 끊김이 느껴진 상황을 적어주세요.'},null,2);
}

const number=value=>value===null?'—':value.toFixed(1);
export class FrameFeedback {
    constructor(doc,context) {
        this.doc=doc;this.context=context;this.samples=new FrameSamples();
        this.enabled=false;this.active=false;this.pendingDraws=0;this.updated=-Infinity;
        this.toggle=doc.querySelector('#fps-toggle');this.copy=doc.querySelector('#fps-copy');
        this.root=doc.querySelector('#fps-hud');this.output=doc.querySelector('#fps-values');
        this.report=doc.querySelector('#fps-report');this.message=doc.querySelector('#fps-message');
        this.toggle.addEventListener('click',()=>this.setEnabled(!this.enabled));
        this.copy.addEventListener('click',()=>void this.copyFeedback());
        doc.addEventListener('keydown',event=>{
            if(event.isComposing||event.repeat||event.ctrlKey||event.altKey||event.metaKey||event.shiftKey)return;
            if(event.target!==doc.querySelector('#game')&&event.target!==this.toggle&&!doc.fullscreenElement)return;
            if(event.target.matches('input,textarea,select'))return;
            if(event.code==='F3'){event.preventDefault();event.stopImmediatePropagation();this.setEnabled(!this.enabled);}
            if(event.code==='F2'&&this.enabled){event.preventDefault();event.stopImmediatePropagation();void this.copyFeedback();}
        },true);
    }
    reset() {
        this.samples.reset();this.pendingDraws=0;this.updated=-Infinity;
        this.report.hidden=true;this.message.textContent='';
        if(this.enabled)this.paint();
    }
    setEnabled(enabled) {
        this.enabled=enabled;this.reset();this.root.hidden=!enabled;
        this.toggle.setAttribute('aria-pressed',String(enabled));
        this.toggle.textContent=enabled?'FPS 끄기 (F3)':'FPS 보기 (F3)';this.copy.disabled=!enabled;
    }
    setActive(active) {
        if(this.active===active)return;
        this.active=active;this.samples.hold();this.pendingDraws=0;
        if(this.enabled)this.paint();
    }
    present() {if(this.enabled&&this.active)this.pendingDraws++;}
    frame(now) {
        if(!this.enabled||!this.active)return;
        this.samples.frame(now,this.pendingDraws);this.pendingDraws=0;
        if(now-this.updated>=500){this.updated=now;this.paint();}
    }
    paint() {
        const s=this.samples.summary();
        this.output.textContent=`rAF ${number(s.fps)} FPS · 갱신 ${number(s.draws_per_second)}/s\np95 ${number(s.p95_ms)} ms · 최대 ${number(s.max_ms)} ms\n50 ms 초과 ${s.slow_intervals}회 · ${this.active?'측정 중':'측정 대기'} · F2 복사`;
    }
    async copyFeedback() {
        if(!this.enabled)return;
        const text=feedbackText({...this.context(),measurement:this.active?'active':'held'},this.samples.summary());
        // Always retain a manual option, including LAN HTTP / denied clipboard.
        this.report.value=text;this.report.hidden=false;
        try {
            if(!this.doc.defaultView.navigator.clipboard?.writeText)throw Error('manual copy');
            await this.doc.defaultView.navigator.clipboard.writeText(text);
            this.report.hidden=true;
            this.message.textContent='피드백을 복사했습니다. 체감 의견을 덧붙여 전달해주세요.';
        } catch {
            this.message.textContent='아래 내용을 선택해 복사한 뒤 체감 의견을 덧붙여주세요.';
            this.report.focus();this.report.select();
        }
    }
}
