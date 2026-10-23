"""Reject misleading frame data before it can enter a comparison."""
import copy
import unittest
from frame_data import frames, validate_window

LINE = "[salimon_client::frame] frame={} interval_ms=16.667 present_wall_ms=1.0 cpu_update_ms=0.001 cpu_render_ms=0.2 gpu_time=Measured(547.583µs) ticks=0 backlog_ms=0.0 focused=true occluded=false"

def window():
    state = {k: {} for k in ['ship','player','survival','fabricator','world','journey','machinery','consumables','carrying']}
    state.update(visual_context={'backend':'Metal','adapter':'Apple M1','viewport_pixels':[1920,1080],'scale_factor':2},window={'minimized':False},native_input_isolated=True)
    state['ship']['power_available'] = False
    return dict(samples=frames([LINE.format(i) for i in range(100)]),before=state,after=copy.deepcopy(state),powered=False)

class Admission(unittest.TestCase):
    def test_valid_and_duration(self):
        row = window()
        validate_window(row)
        self.assertAlmostEqual(row['samples'][0]['gpu_latest_ms'], .547583)
    def test_unknown_gpu_is_rejected(self):
        with self.assertRaises(RuntimeError): frames([LINE.format(1).replace('Measured(547.583µs)','unexpected')])
    def test_unfocused_occluded_gap_and_short_are_rejected(self):
        for key,value in [('focused',False),('occluded',True),('ticks',1),('interval_ms',0),('frame',1000)]:
            row=window();row['samples'][50][key]=value
            with self.subTest(key=key), self.assertRaises(AssertionError): validate_window(row)
        row=window();row['samples'].pop()
        with self.assertRaises(AssertionError): validate_window(row)
    def test_changed_source_and_wrong_context_are_rejected(self):
        for change in ['source','adapter','viewport','power','minimized','isolation']:
            row=window();after=row['after']
            if change=='source':after['world']['changed']=True
            elif change=='adapter':after['visual_context']['adapter']='software'
            elif change=='viewport':after['visual_context']['viewport_pixels']=[1280,800]
            elif change=='power':after['ship']['power_available']=True
            elif change=='minimized':after['window']['minimized']=True
            else:after['native_input_isolated']=False
            with self.subTest(change=change), self.assertRaises(AssertionError):validate_window(row)

class CloudAdmission(unittest.TestCase):
    def cloud(self):
        row=window()
        for state in [row['before'],row['after']]:
            state['visual_context'].update(backend='Vulkan',adapter='llvmpipe (LLVM 19.1.7, 256 bits)',viewport_pixels=[1280,800],scale_factor=1)
        return row
    def test_cloud_requires_named_profile_and_retains_native_default(self):
        row=self.cloud()
        validate_window(row,'cloud-llvmpipe-1280')
        with self.assertRaises(AssertionError):validate_window(row)
        with self.assertRaises(AssertionError):validate_window(row,'unknown')
    def test_cloud_rejects_focus_geometry_adapter_and_context_changes(self):
        for change in ['focus','viewport','adapter','context','isolation','ticks']:
            row=self.cloud()
            if change=='focus':row['samples'][50]['focused']=False
            elif change=='viewport':row['before']['visual_context']['viewport_pixels']=[1920,1080]
            elif change=='adapter':row['before']['visual_context']['adapter']='unknown'
            elif change=='context':row['after']['visual_context']['driver']='changed'
            elif change=='isolation':row['after']['native_input_isolated']=False
            else:row['samples'][50]['ticks']=1
            with self.subTest(change=change),self.assertRaises(AssertionError):validate_window(row,'cloud-llvmpipe-1280')

if __name__ == '__main__':unittest.main()
