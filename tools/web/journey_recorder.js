// Optional local verification recorder. No network I/O or changes to game rules.
const names=['terrain','bricks','objects'];
function encoded(bytes) {
    let text='';
    for(let at=0;at<bytes.length;at+=16384)text+=String.fromCharCode(...bytes.subarray(at,at+16384));
    return btoa(text);
}
export class JourneyRecorder {
    constructor(app,{original=false,renderEvery=13}={}) {
        if(!Number.isInteger(renderEvery)||renderEvery<1)throw new Error('invalid render interval');
        this.data={version:1,original,attempts:[]};this.renderEvery=renderEvery;this.current=null;this.game=null;
        this.detach=app.observe((type,game,detail)=>this.event(type,game,detail));
    }
    read(){return {state:JSON.parse(this.game.state()),view:JSON.parse(this.game.view_state())};}
    maps(){return Object.fromEntries(names.map(name=>[name,this.game[name]()]));}
    event(type,game,detail) {
        this.game=game;
        if(type==='reset') {
            if(this.current&&!this.current.final)throw new Error('unfinished attempt replaced');
            const initial=this.read();this.previous=this.maps();initial.maps=Object.fromEntries(names.map(n=>[n,encoded(this.previous[n])]));
            this.current={level:initial.view.level,difficulty:initial.view.difficulty,reason:detail.reason,initial,rows:[],events:[],frames:[]};
            this.data.attempts.push(this.current);this.lastFrame=null;
        } else if(!this.current)return;
        if(type==='input')this.current.events.push({step:this.read().state.session.control_step,op:detail.op,args:detail.args});
        if(type==='before'&&!this.current.final)this.snapshot();
        if(type==='after'&&this.read().state.completion?.phase!=='Playing'&&!this.current.final){this.snapshot();this.finish();}
        if(type==='present')this.present(detail.canvas);
    }
    snapshot() {
        const {state}=this.read(),step=state.session.control_step;
        if(this.current.rows.at(-1)?.state.session.control_step===step)throw new Error('duplicate boundary');
        const next=this.maps(),changes={};
        for(const name of names) {
            const before=this.previous[name],after=next[name];
            if(before.length!==after.length)throw new Error('map dimensions changed within attempt');
            changes[name]=[];
            for(let i=0;i<after.length;i++)if(before[i]!==after[i])changes[name].push([i,before[i],after[i]]);
        }
        this.previous=next;this.current.rows.push({state,changes});
    }
    present(canvas) {
        const {state,view}=this.read(),step=state.session.control_step;
        if(step%this.renderEvery!==0&&state.completion?.phase==='Playing')return;
        const key=JSON.stringify([step,view]);if(key===this.lastFrame)return;this.lastFrame=key;
        const index=this.current.frames.length;
        const rgba=canvas.getContext('2d').getImageData(0,0,320,200).data;
        this.current.events.push({step,op:'Frame',args:[index]});
        this.current.frames.push({index,step,state,view,rgb:encoded(this.game.frame()),canvas:encoded(rgba)});
    }
    finish() {
        const info=this.read();info.maps=Object.fromEntries(names.map(n=>[n,encoded(this.previous[n])]));info.inputs=this.game.input_log();
        this.current.final=info;
    }
    stop() {
        this.detach();
        if(this.current&&!this.current.final) {
            if(this.current.rows.at(-1)?.state.session.control_step!==this.read().state.session.control_step)this.snapshot();
            this.finish();
        }
        return this.data;
    }
}
