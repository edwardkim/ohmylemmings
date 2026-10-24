import unittest
from tools.hd_surface_quality import reframe, ink_bands, ink_bounds


class SurfaceQualityTests(unittest.TestCase):
    def test_an_exit_crossing_the_nominal_grid_row_is_not_cropped(self):
        width=384;pixels=bytearray(width*1024*4)
        for start,end in [(100,150),(400,450),(630,920)]:
            for y in range(start,end):
                for x in range(50,300):pixels[(y*width+x)*4+3]=255
        bands=ink_bands(pixels,width,1024,0,8)
        self.assertEqual(bands,[[100,150],[400,450],[630,920]])
        start,end=bands[2]
        self.assertEqual(ink_bounds(pixels,width,[0,start,width,end-start],8),[50,630,300,920])

    def test_reframing_preserves_visible_world_bounds_and_pose_key(self):
        old=dict(key={'object':[2,1,3]},atlas=4,rect=[10,20,100,80],display=[200,160],anchor=[30,50])
        new=reframe(old,[15,25,105,95],[200,100,380,240],[512,512])
        self.assertEqual(new['key'],old['key'])
        self.assertEqual(new['atlas'],old['atlas'])
        for axis in range(2):
            prior=[-old['anchor'][axis]+(p-old['rect'][axis])*old['display'][axis]/old['rect'][axis+2] for p in [[15,25][axis],[105,95][axis]]]
            after=[-new['anchor'][axis]+(p-new['rect'][axis])*new['display'][axis]/new['rect'][axis+2] for p in [[200,100][axis],[380,240][axis]]]
            self.assertTrue(all(abs(a-b)<=1.5 for a,b in zip(prior,after)),(prior,after))
        self.assertEqual(old['rect'],[10,20,100,80])

    def test_empty_outside_or_degenerate_bounds_are_rejected(self):
        old=dict(key={'object':[0,0,0]},atlas=1,rect=[0,0,20,20],display=[40,40],anchor=[0,0])
        for bounds in [[0,0,0,2],[-1,0,2,2],[0,0,21,2],[0,5,5,4]]:
            with self.assertRaises(ValueError):reframe(old,[0,0,20,20],bounds,[20,20])
