import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_desktop import verify_scaled_frame

class DesktopTests(unittest.TestCase):
    def test_integer_scaled_frame_in_a_resized_window_checks_all_letterbox_pixels(self):
        logical=b''.join(bytes([x%251,y%251,(x+y)%251]) for y in range(200) for x in range(320))
        width,height=941,508;scale=2;left,top=150,54
        rows=[]
        for y in range(height):
            if top<=y<top+400:
                row=logical[((y-top)//scale)*960:((y-top)//scale+1)*960]
                rows.append(bytes(left*3)+b''.join(row[x:x+3]*scale for x in range(0,960,3))+bytes((width-left-640)*3))
            else:rows.append(bytes(width*3))
        pixels=b''.join(rows)
        verify_scaled_frame(width,height,pixels,logical)
        for index in [0,(top*width+left)*3,len(pixels)-1]:
            damaged=bytearray(pixels);damaged[index]^=1
            with self.assertRaises(AssertionError):verify_scaled_frame(width,height,bytes(damaged),logical)
    def test_truncated_or_smaller_than_logical_capture_cannot_pass(self):
        for width,height,pixels,logical in [(319,200,bytes(319*200*3),bytes(320*200*3)),(320,200,b'',bytes(320*200*3)),(320,200,bytes(320*200*3),b'')]:
            with self.assertRaises(AssertionError):verify_scaled_frame(width,height,pixels,logical)
if __name__=='__main__':unittest.main()
