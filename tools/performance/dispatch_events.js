import {Series} from './measure.js';

const kinds=['reset','input','present'];
const ops=['Release','Pointer','Key','Pause','Resume'];
const pointers=['move','down','up'];
const keys=['KeyB','KeyD','KeyM','KeyU','KeyF','KeyC','KeyE','KeyK','KeyN','KeyR','Space','Enter','Escape','ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End'];
const reasons=['restart','continue','synthetic','level','load'];

// Opt-in diagnosis only. Every argument of the app's input observer is retained.
export class DispatchEvents {
    constructor(capacity) {this.series=new Series(capacity,10);}
    record(event,detail,at,step,inCallback) {
        const kind=kinds.indexOf(event);
        if(kind<0)return;
        const args=detail.args||[];
        if(event==='input') {
            const op=ops.indexOf(detail.op);
            if(op<0)throw Error('unknown diagnostic input');
            const code=detail.op==='Pointer'?pointers.indexOf(args[0]):detail.op==='Key'?keys.indexOf(args[0]):-1;
            this.series.push(at,step,kind,op,code,Number(args[1])||0,Number(args[2])||0,Number(args[3])||0,Number(args[4])||0,Number(args[5])||0);
        } else this.series.push(at,step,kind,event==='reset'?reasons.indexOf(detail.reason):Number(inCallback));
    }
    get bytes(){return this.series.bytes;}
    result(){return {columns:['at_ms','control_step','kind','op_or_reason_or_in_raf','pointer_or_key','x_or_repeat','y','button','buttons','clicks'],kinds,ops,pointers,keys,reasons,events:this.series.rows(),reservedBytes:this.bytes};}
}
