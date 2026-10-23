#!/usr/bin/env python3
"""Audit two v1 packs: exact protected RGBA and unchanged frame geometry."""
import argparse,hashlib,json,struct,zlib
from pathlib import Path


def unpack(path):
    data=path.read_bytes();assert data[:8]==b'LHDPAK01';count,frames=struct.unpack_from('<HH',data,8);at=12;atlases=[]
    for _ in range(count):
        w,h,size,crc,cutoff=struct.unpack_from('<IIIII',data,at);at+=20;pixels=memoryview(data)[at:at+size];at+=size
        assert len(pixels)==w*h*4 and zlib.crc32(pixels)==crc
        atlases.append((w,h,pixels,cutoff))
    rows=[struct.unpack_from('<BBHHHIIIIHHhh',data,at+i*32) for i in range(frames)];assert at+frames*32==len(data)
    return data,atlases,{row[:3]:row for row in rows}


def compare(old_path,new_path):
    old,oa,of=unpack(old_path);new,na,nf=unpack(new_path);assert of.keys()==nf.keys()
    protected=visible=changed_invisible=0
    for key,before in of.items():
        after=nf[key];sw,sh,source,cutoff=oa[before[3]];tw,th,target,new_cutoff=na[after[3]];assert cutoff==new_cutoff
        assert before[7:]==after[7:],('frame geometry changed',key)
        sx,sy,w,h=before[5:9];tx,ty=after[5:7];box=[w,h,-1,-1]
        for row in range(h):
            start=((sy+row)*sw+sx)*4;line=source[start:start+w*4]
            for col,a in enumerate(line[3::4]):
                if a>cutoff:box=[min(box[0],col),min(box[1],row),max(box[2],col),max(box[3],row)];visible+=1
        assert box[2]>=0
        x0=max(0,box[0]-1);y0=max(0,box[1]-1);x1=min(w,box[2]+2);y1=min(h,box[3]+2)
        for row in range(h):
            a=((sy+row)*sw+sx)*4;b=((ty+row)*tw+tx)*4;old_line=source[a:a+w*4];new_line=target[b:b+w*4]
            if y0<=row<y1:
                assert old_line[x0*4:x1*4]==new_line[x0*4:x1*4],('protected RGBA differs',key,row)
                protected+=x1-x0;ranges=[(0,x0),(x1,w)]
            else:ranges=[(0,w)]
            for left,right in ranges:
                u=old_line[left*4:right*4];v=new_line[left*4:right*4]
                assert all(a<=cutoff for a in u[3::4]) and all(a<=cutoff for a in v[3::4]),('visible border changed',key,row)
                if u!=v:changed_invisible+=sum(u[i:i+4]!=v[i:i+4] for i in range(0,len(u),4))
    return dict(status='PASS',frames=len(of),visible_samples=visible,protected_rgba_samples=protected,changed_invisible_filler_samples=changed_invisible,all_visible_and_filter_boundary_rgba_exact=True,all_frame_dimensions_display_anchors_equal=True,old_bytes=len(old),bytes=len(new),saved_bytes=len(old)-len(new),old_atlases=len(oa),atlases=len(na),old_sha256=hashlib.sha256(old).hexdigest(),sha256=hashlib.sha256(new).hexdigest(),scope='Full frame geometry unchanged; exact visible RGBA plus one-source-pixel filtering boundary. Only cutoff-invisible filler may share storage. Actual SDL/Canvas comparison is separate.')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('baseline',type=Path);p.add_argument('candidate',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();r=compare(a.baseline,a.candidate)
    if a.output:
        root=Path(__file__).resolve().parents[1]/'assets/local';out=a.output.resolve();assert out.is_relative_to(root) and not out.exists();out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(r,indent=2)+'\n')
    print(json.dumps(r))


if __name__=='__main__':main()
