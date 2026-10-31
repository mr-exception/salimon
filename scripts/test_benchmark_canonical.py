"""Benchmark collectors must retain missing counters and reject interrupted samples."""
import unittest
from unittest.mock import patch

import benchmark_canonical as benchmark


class CanonicalBenchmark(unittest.TestCase):
    def test_macos_rss_does_not_invent_process_high_water_or_gpu_memory(self):
        with patch.object(benchmark, 'command', return_value='123456') as command:
            self.assertEqual(benchmark.memory_sample(42, 'darwin'),
                             {'rss_kib': 123456, 'process_high_water_kib': None})
        command.assert_called_once_with(['ps', '-o', 'rss=', '-p', '42'])

    def test_linux_memory_keeps_kernel_high_water(self):
        with patch.object(benchmark.Path, 'read_text', return_value='Name:\tgame\nVmRSS:\t42 kB\nVmHWM:\t84 kB\n'):
            self.assertEqual(benchmark.memory_sample(42, 'linux'),
                             {'rss_kib': 42, 'process_high_water_kib': 84})

    def test_gpu_units_unavailable_counters_and_foreground_admission(self):
        rows = []
        for n in range(100):
            gpu = ('Measured(2ms)', 'Measured(3000µs)', 'Measured(4000000ns)',
                   'Measured(0.005s)', 'Unavailable')[n % 5]
            rows.append(f'frame={n} interval_ms=16.7 present_wall_ms=2.1 cpu_update_ms=0.1 '
                        f'cpu_render_ms=0.2 gpu_time={gpu} ticks=1 backlog_ms=0.0 focused=true occluded=false')
        parsed = benchmark.frame_samples('\n'.join(rows))
        self.assertEqual([r['gpu_ms'] for r in parsed[:5]], [2, 3, 4, 5, None])
        benchmark.validate_frames(parsed)
        with self.assertRaises(ValueError):
            benchmark.validate_frames(parsed[:99])
        with self.assertRaises(ValueError):
            benchmark.validate_frames(parsed[:50] + parsed[51:] + [parsed[-1]])
        parsed[50]['focused'] = False
        with self.assertRaises(ValueError):
            benchmark.validate_frames(parsed)

    def test_macos_window_context_does_not_call_x11_or_confuse_points_with_pixels(self):
        with patch.object(benchmark, 'command', return_value='{"os_bounds_unit":"logical points; includes decorations"}') as command:
            context = benchmark.window_context(42, 'darwin')
        self.assertIn('logical points', context['os_bounds_unit'])
        self.assertEqual(command.call_args.args[0][0], 'swift')


if __name__ == '__main__':
    unittest.main()
