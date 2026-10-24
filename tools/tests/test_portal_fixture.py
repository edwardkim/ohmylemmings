import json,struct,unittest
from tools.portal_fixture import bank,HEIGHTS,LEFT,TOP
from tools.hd_assets import ROOT
from tools.hd_surface_quality import reframe
class PortalFixtureTests(unittest.TestCase):
    def test_themes_have_the_documented_portal_metadata_and_no_external_files(self):
        source=json.loads((ROOT/'assets/manifests/hd_portals.json').read_text())['source_objects']
        for theme in range(5):
            files=bank(theme);ground=files[f'GROUND{theme}O.DAT'];self.assertEqual(len(ground),1056)
            for id,count in [(0,1),(1,10)]:
                row=next(r for r in source if r['set']==theme and r['id']==id)
                self.assertEqual((ground[id*28+2],ground[id*28+3],ground[id*28+4],ground[id*28+5]),(row['first_frame'],count,row['width'],row['height']))
            self.assertEqual(struct.unpack_from('<HH',ground,14),(LEFT[theme],TOP[theme]));self.assertEqual(ground[5],HEIGHTS[theme])
            self.assertEqual(set(files),{*[f'LEVEL{i:03}.DAT' for i in range(10)],f'GROUND{theme}O.DAT',f'VGAGR{theme}.DAT','MAIN.DAT','ODDTABLE.DAT'})
        for invalid in [-1,5,True,'1']:
            with self.assertRaises(ValueError):bank(invalid)
    def test_default_portal_mapping_keeps_phase_and_subpixel_alignment(self):
        art=json.loads((ROOT/'assets/manifests/hd_portals.json').read_text());m=json.loads((ROOT/'assets/manifests/remaster_pack.json').read_text())
        self.assertTrue(m['complete_portals'])
        quality=json.loads((ROOT/'assets/manifests/hd_surface_quality.json').read_text())
        replacements={tuple(r['previous_frame']['key']['object']):(g,r) for g in quality['facilities'] for r in g['frames']}
        for group in art['layout']:
            self.assertEqual(len(group['rows']),11)
            for row in group['rows']:
                self.assertEqual(row['frame'],0 if row['id']==0 or row['cell']==9 else row['cell']+1)
                bound=next(f for f in m['frames'] if f['key'].get('object')==[group['set'],row['id'],row['frame']])
                group_quality,replacement=replacements[tuple(bound['key']['object'])]
                for key in ['rect','display','anchor']:self.assertEqual(replacement['previous_frame'][key],row[key])
                self.assertEqual(bound,reframe(replacement['previous_frame'],replacement['previous_bounds'],replacement['bounds'],group_quality['size']))
                for i in range(2):self.assertLessEqual(abs((row['alignment_source'][i]-row['rect'][i])*row['display'][i]/row['rect'][i+2]-row['anchor'][i]-row['alignment_target_hd'][i]),0.5)

    def test_exit_flames_reuse_only_effect_zero_six_frame_source_profiles(self):
        for theme in range(5):
            ground=bank(theme)[f'GROUND{theme}O.DAT']
            for id in [[7,6,6,6,8][theme]]:
                row=ground[id*28:(id+1)*28]
                self.assertEqual(struct.unpack_from('<H',row)[0],2)
                self.assertEqual(tuple(row[2:6]),(0,6,48,24 if theme==1 else 8))
                self.assertEqual(row[20],0)
                self.assertEqual(row[18:20],b'\0\0')
