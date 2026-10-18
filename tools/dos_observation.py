"""Strict framebuffer comparison; video evidence never certifies DOS internal state."""

import hashlib

WIDTH, HEIGHT = 320, 160
UNVERIFIED = ['internal_state', 'tick_rate', 'input_tick_alignment', 'skills',
              'triggers', 'rescue', 'result', 'audio', 'full_screen_hud']


def game_frame(width, height, rgb):
    """Verify 2x nearest duplication, crop HUD, then quantize BOTH sources to RGB565.

    Return three bytes per pixel (R5/G6/B5 component codes, not display RGB).
    DOSBox's surface capture on this host is RGB565; comparing 24-bit palette
    expansion would attribute capture quantization to the game's renderer.
    No spatial shifts, cursor masks, color tolerances, or resampling are used.
    """
    if (width, height) not in [(320, 200), (640, 400)] or len(rgb) != width*height*3:
        raise ValueError('expected complete 320x200 or exactly doubled 640x400 RGB24 frame')
    if width == 640:
        rows = []
        for y in range(HEIGHT):
            row = rgb[y*3840:y*3840+1920]
            if row != rgb[y*3840+1920:(y+1)*3840]:
                raise ValueError('DOS scanlines are not exact 2x duplicates')
            for c in range(3):
                if row[c::6] != row[c+3::6]:
                    raise ValueError('DOS pixels are not exact 2x duplicates')
            reduced = bytearray(960)
            for c in range(3):
                reduced[c::3] = row[c::6]
            rows.append(reduced)
        rgb = b''.join(rows)
    rgb = rgb[:WIDTH*HEIGHT*3]
    result = bytearray(len(rgb))
    for c, shift in enumerate([3, 2, 3]):
        result[c::3] = rgb[c::3].translate(bytes(v >> shift for v in range(256)))
    return bytes(result)


def pixel_difference(reference, candidate):
    if len(reference) != WIDTH*HEIGHT*3 or len(candidate) != len(reference):
        raise ValueError('incomplete normalized viewport')
    first, count = None, 0
    x0, y0, x1, y1 = WIDTH, HEIGHT, 0, 0
    for i in range(0, len(reference), 3):
        if reference[i:i+3] != candidate[i:i+3]:
            x, y = (i//3) % WIDTH, (i//3)//WIDTH
            first = first or [x, y]
            count += 1
            x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x+1), max(y1, y+1)
    return dict(pixels=count, first=first, bounds=[x0, y0, x1, y1] if count else None)


def scene_signatures(frame):
    if len(frame) != WIDTH*HEIGHT*3:
        raise ValueError('incomplete normalized viewport')
    return dict(full_320x160=hashlib.sha256(frame).hexdigest(),
                upper_320x100=hashlib.sha256(frame[:WIDTH*100*3]).hexdigest())


def match_anchors(native, observed):
    """Find exact, ordered image anchors; frame indexes remain video indexes.

    Native control steps are labels for candidate images, not inferred DOS ticks.
    Repeated still images remain ambiguous, with every matching index reported.
    """
    for rows in [native, observed]:
        if any(a[0] >= b[0] for a, b in zip(rows, rows[1:])):
            raise ValueError('frame/step indexes must increase strictly')
    previous, anchors, complete = -1, [], bool(native and observed)
    for step, signature in native:
        matches = [index for index, value in observed if value == signature and index > previous]
        anchors.append(dict(native_control_step=step, scene_sha256=signature, dos_frames=matches))
        if matches:
            previous = matches[0]
        else:
            complete = False
    return dict(status='MATCH_VISUAL_ANCHORS' if complete else 'UNVERIFIED_VISUAL_ANCHORS',
                anchors=anchors, unverified=UNVERIFIED.copy())


def repeat_verdict(runs):
    if len(runs) != 3 or any(r.get('status') != 'MATCH_VISUAL_ANCHORS' for r in runs):
        return 'UNVERIFIED'
    if not runs[0].get('settings_sha256') or not runs[0].get('scene_sha256'):
        return 'UNVERIFIED'
    if any((r.get('settings_sha256'), r.get('scene_sha256')) !=
           (runs[0]['settings_sha256'], runs[0]['scene_sha256']) for r in runs):
        return 'UNVERIFIED'
    return 'MATCH_THREE_RUNS'
