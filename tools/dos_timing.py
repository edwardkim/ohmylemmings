"""Separate the pinned executable's VGA-synchronised and free-running PIT paths."""
from fractions import Fraction


def control_rate(unlocked_pit, divisor, video_rate):
    if divisor <= 0 or video_rate <= 0:
        raise ValueError('positive PIT divisor and video rate required')
    # Original offset 0x2a6 waits until IRQ counter > 4. DOSBox timer.h pins PIT.
    return Fraction(1193182, divisor*5) if unlocked_pit else video_rate/5


def check_cadence(anchors, video_rate, logical_rate):
    if len(anchors) < 2:
        return dict(status='UNVERIFIED', maximum_residual_video_frames=None)
    first_step, first_frame = anchors[0]
    residuals = [abs(Fraction(frame-first_frame) - (step-first_step)*video_rate/logical_rate)
                 for step, frame in anchors[1:]]
    maximum = max(residuals)
    return dict(status='MATCH_VIDEO_QUANTIZATION' if maximum < 1 else 'DIFFERENT',
                maximum_residual_video_frames=str(maximum),
                expected_calls_per_second=str(logical_rate),
                video_frames_per_call=str(video_rate/logical_rate))
