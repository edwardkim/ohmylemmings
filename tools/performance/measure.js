// Fixed recorder storage is reserved before the measured interval.
export class Series {
    constructor(capacity, width) {
        if (!Number.isInteger(capacity) || capacity < 1 || !Number.isInteger(width) || width < 1 || width > 10) throw Error('invalid recorder size');
        this.width=width;this.count=0;this.values=new Float64Array(capacity*width);
    }
    push(a,b,c,d=0,e=0,f=0,g=0,h=0,i=0,j=0) {
        if((this.count+1)*this.width>this.values.length)throw Error('recorder overflow');
        const offset=this.count*this.width;
        for(let n=0;n<this.width;n++) {
            const value=n===0?a:n===1?b:n===2?c:n===3?d:n===4?e:n===5?f:n===6?g:n===7?h:n===8?i:j;
            if(!Number.isFinite(value))throw Error('invalid recorder row');
            this.values[offset+n]=value;
        }
        this.count++;
    }
    rows() {return Array.from({length:this.count},(_,i)=>Array.from(this.values.subarray(i*this.width,(i+1)*this.width)));}
    get bytes(){return this.values.byteLength;}
}

export function timeMethod(proto, name, clock, observed) {
    const original=proto[name];
    proto[name]=function(){
        const begin=clock();
        try{return original.apply(this,arguments);}
        finally{observed(clock()-begin);}
    };
    return ()=>{proto[name]=original;};
}
