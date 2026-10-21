import {Series,timeMethod} from './measure.js';
export function checkMeasurement(assert,throws) {
    const series=new Series(2,3);series.push(1,2,3);series.push(4,5,6);
    assert(JSON.stringify(series.rows())==='[[1,2,3],[4,5,6]]'&&series.bytes===48,'fixed recorder rows/capacity');
    throws(()=>series.push(7,8,9),'recorder overflow accepted');
    const invalid=new Series(1,3);throws(()=>invalid.push(1,2,NaN),'invalid recorder row accepted');
    const object={value:7,call(){assert(arguments.length===3,'wrapper changed arity');return this.value;}};
    let now=0,elapsed=-1;const restore=timeMethod(object,'call',()=>++now,value=>{elapsed=value;});
    assert(object.call(1,2,3)===7&&elapsed===1,'wrapper changed receiver/result/time');restore();
    const context=document.createElement('canvas').getContext('2d');
    const undo=timeMethod(context,'putImageData',()=>0,()=>{});
    try{context.putImageData(context.createImageData(1,1),0,0);}finally{undo();}
    const failure={call(){throw Error('original failure');}};
    timeMethod(failure,'call',()=>0,()=>{});
    let message='';try{failure.call();}catch(e){message=e.message;}
    assert(message==='original failure','wrapper changed failure');
    return {status:'PASS',fixed_capacity:true,preserves_arguments_receiver_result_and_failure:true,canvas_three_argument_overload:true};
}
