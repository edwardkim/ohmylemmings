"""Pure validation of read-only original DOS AdLib timing observations."""
from fractions import Fraction
try:
    from .dos_memory import decode_sample
except ImportError:
    from dos_memory import decode_sample


def decode_audio_sample(raw):
    result=decode_sample(raw);result['adlib_selected_u8']=raw[0x2010]
    # DS:001c increments once per IRQ, wrapping at 256. DS:1fcd is a
    # game-loop budget reset every five IRQs, so it cannot measure cadence.
    # The shared observer omits that legacy field from change detection.
    result['audio_irq_counter_u8']=raw[0x1c];return result

def cadence(samples):
    game=[s for s in samples if 1<=s['released_u8']<=10]
    if len(game)<20 or any(s['adlib_selected_u8']!=255 or s['pc_flags'][4]!=0 for s in game):raise ValueError('actual AdLib/VGA path not observed')
    elapsed=0;irqs=0;pairs=0
    for a,b in zip(game,game[1:]):
        delta=b['host_elapsed_ns']-a['host_elapsed_ns']
        if 0<delta<1_000_000_000:
            elapsed+=delta;irqs+=(b['audio_irq_counter_u8']-a['audio_irq_counter_u8'])&255;pairs+=1
    if elapsed<5_000_000_000:raise ValueError('at least five seconds of bounded-interval samples required')
    measured=Fraction(irqs*1_000_000_000,elapsed);expected=Fraction(2190197,31250)
    if abs(measured-expected)>2:raise ValueError('observed IRQ rate differs from VGA contract')
    return dict(samples=len(game),pairs=pairs,irqs=irqs,elapsed_ns=elapsed,measured_hz=float(measured),expected_hz=str(expected),tolerance_hz=2,adlib_selected_u8=255,unlocked_pit_u8=0,limitation='asynchronous read-only RAM sampling; no atomic instruction trace or OPL/PCM/device proof')
