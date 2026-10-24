#!/usr/bin/env python3
"""Validate source-preserving HD surface replacements and visible frame alignment."""
import argparse
import copy
import json
from pathlib import Path
try:
    from .hd_assets import read_pin, decode_png, sprite_bounds
except ImportError:
    from hd_assets import read_pin, decode_png, sprite_bounds

ROOT=Path(__file__).resolve().parents[1]


def reframe(frame, previous, bounds, canvas):
    """Crop new ink with a small gutter, keeping old visible HD bounds/phase."""
    for box,size in [(bounds,canvas)]:
        if len(box)!=4 or any(type(v) is not int for v in box) or not (0<=box[0]<box[2]<=size[0] and 0<=box[1]<box[3]<=size[1]):
            raise ValueError('invalid visible bounds')
    if previous[0]>=previous[2] or previous[1]>=previous[3]:raise ValueError('empty previous bounds')
    result=copy.deepcopy(frame)
    left=[max(0,bounds[0]-3),max(0,bounds[1]-3)]
    right=[min(canvas[0],bounds[2]+3),min(canvas[1],bounds[3]+3)]
    rect=left+[right[i]-left[i] for i in range(2)]
    display=[];anchor=[]
    for axis in range(2):
        old_scale=frame['display'][axis]/frame['rect'][axis+2]
        desired=(previous[axis+2]-previous[axis])*old_scale
        display.append(round(rect[axis+2]*desired/(bounds[axis+2]-bounds[axis])))
        scale=display[-1]/rect[axis+2]
        old_start=-frame['anchor'][axis]+(previous[axis]-frame['rect'][axis])*old_scale
        anchor.append(round((bounds[axis]-rect[axis])*scale-old_start))
        if not 1<=display[-1]<=4096 or not -32768<=anchor[-1]<=32767:raise ValueError('frame dimensions/range')
        for edge in [axis,axis+2]:
            before=-frame['anchor'][axis]+(previous[edge]-frame['rect'][axis])*old_scale
            after=-anchor[-1]+(bounds[edge]-rect[axis])*scale
            if abs(before-after)>1.5:raise ValueError('visible HD alignment changed')
    result.update(rect=rect,display=display,anchor=anchor)
    return result


def ink_bounds(pixels,width,cell,cutoff):
    x,y,w,h=cell;left=x+w;right=x;top=y+h;bottom=y
    for py in range(y,y+h):
        for px in range(x,x+w):
            if pixels[(py*width+px)*4+3]>cutoff:
                left=min(left,px);right=max(right,px+1);top=min(top,py);bottom=max(bottom,py+1)
    if left>=right or top>=bottom:raise ValueError('empty sprite cell')
    return [left,top,right,bottom]


def ink_bands(pixels,width,height,column,cutoff):
    """Locate pose rows by actual ink; exit arches can cross nominal grid rows."""
    occupied=[]
    for y in range(height):
        if any(pixels[(y*width+x)*4+3]>cutoff for x in range(column*384,min((column+1)*384,width))):occupied.append(y)
    bands=[]
    for y in occupied:
        if not bands or y-bands[-1][1]>32:bands.append([y,y+1])
        else:bands[-1][1]=y+1
    return bands


def check(root=ROOT):
    data=json.loads((root/'assets/manifests/hd_surface_quality.json').read_text())
    pack=json.loads((root/'assets/manifests/remaster_pack.json').read_text())
    if len(data['materials'])!=5 or len(data['facilities'])!=5:raise ValueError('incomplete theme profile')
    frames={json.dumps(f['key'],sort_keys=True):f for f in pack['frames']}
    count=0
    for entry in data['materials']+data['facilities']:
        raw=read_pin(entry['image'],root);w,h,pixels=decode_png(raw)
        if [w,h]!=entry['size']:raise ValueError('image dimensions')
        read_pin(entry['prompt'],root)
        active=pack['atlases'][entry['atlas']]
        if any(active[k]!=entry['image'][k] for k in ['path','bytes','sha256']):raise ValueError('inactive replacement')
        if entry in data['materials']:
            if any(a!=255 for a in pixels[3::4]):raise ValueError('material transparency')
            if frames[json.dumps({'material':entry['set']},sort_keys=True)]['display']!=entry['period']:raise ValueError('material footprint')
        else:
            previous=read_pin(entry['previous_image'],root);pw,ph,prior=decode_png(previous)
            for row in entry['frames']:
                old_bounds=ink_bounds(prior,pw,row['previous_cell'],entry['cutoff'])
                bounds=ink_bounds(pixels,w,row['cell'],entry['cutoff'])
                if bounds!=row['bounds'] or old_bounds!=row['previous_bounds']:raise ValueError('recorded ink bounds')
                expected=reframe(row['previous_frame'],old_bounds,bounds,[w,h])
                if frames[json.dumps(expected['key'],sort_keys=True)]!=expected:raise ValueError('runtime frame alignment')
                try:sprite_bounds(pixels,w,expected['rect'],entry['cutoff'])
                except ValueError as error:raise ValueError(f"unsafe sprite crop {expected['key']}: {error}") from error
                count+=1
    if count!=55:raise ValueError('facility frame coverage')
    return dict(status='PASS',materials=5,facility_frames=count,source_pixels_preserved=True,max_visible_alignment_error_hd_px=1.5)


if __name__=='__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    print(json.dumps(check()))
