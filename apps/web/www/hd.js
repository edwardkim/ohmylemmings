// Shared Rust draw-list v1 adapter. Texture pixels cross WASM only when their
// revision changes; terrain is rasterized by changed chunks, never per display.
export class HdCanvas {
    constructor(game) { this.game=game;this.cache=new Map(); }
    invalidate() { this.cache.clear(); }
    texture(id) {
        const revision=this.game.hd_texture_revision(id);
        if (!revision) throw new Error('HD texture is unavailable');
        const old=this.cache.get(id);if(old?.revision===revision)return old.canvas;
        const width=this.game.hd_texture_width(id),height=this.game.hd_texture_height(id);
        const bytes=this.game.hd_texture(id);
        if(!width||!height||bytes.length!==width*height*4)throw new Error('invalid HD texture size');
        const canvas=document.createElement('canvas');canvas.width=width;canvas.height=height;
        canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(bytes),width,height),0,0);
        this.cache.set(id,{revision,canvas});return canvas;
    }
    draw(context) {
        const commands=this.game.hd_commands();
        if(commands.length%12)throw new Error('invalid HD draw-list length');
        let clipped=false;context.save();
        try {
            context.setTransform(1,0,0,1,0,0);context.imageSmoothingEnabled=true;
            for(let i=0;i<commands.length;i+=12) {
                const c=commands.subarray(i,i+12);
                if(c[0]===0) {context.fillStyle=`rgba(${c[5]},${c[6]},${c[7]},${c[8]/255})`;context.fillRect(c[1],c[2],c[3],c[4]);}
                else if(c[0]===1)context.drawImage(this.texture(c[1]),c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);
                else if(c[0]===2) {
                    if(c[1]&&!clipped){context.save();context.beginPath();context.rect(0,0,1920,960);context.clip();clipped=true;}
                    else if(!c[1]&&clipped){context.restore();clipped=false;}
                } else throw new Error('unknown HD draw command');
            }
        } finally {if(clipped)context.restore();context.restore();}
    }
}
