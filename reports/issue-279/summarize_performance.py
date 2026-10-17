import json,statistics,sys
from pathlib import Path
sys.path.insert(0,str(Path('scripts').resolve()))
from benchmark_fragments import compare
root=Path('reports/issue-279');src=Path('artifacts/native-279')
before=json.loads((src/'before-onboard/summary.json').read_text())
after=json.loads(Path('reports/issue-255/settled/summary.json').read_text())
bs=json.loads((src/'before-spheres/summary.json').read_text());as_=json.loads((src/'after-spheres/summary.json').read_text())
regressions=compare(after,before,.2)+compare(as_,bs,.2)
(root/'performance-comparison.json').write_text(json.dumps(dict(regressions=regressions,tolerance=.2,before_onboard=before,after_onboard=after,before_spheres=bs,after_spheres=as_),indent=2)+'\n')
lines=['# Native performance comparison','',
'Medians of three per-window p95 values, same physical M1 and foreground admission. All 500-piece cases remain naturally asleep with original IDs/mass. Raw p99/max/spikes are preserved in the linked summaries and compressed intervals. Native windows ran sequentially, after compilation/capture tests finished.','',
'| Workload / build | Intervals total | Median intervals/s | Frame p95 ms | Physics p95 ms | CPU render p95 ms | GPU latest p95 ms | Max interval ms | Draw calls |',
'| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
for label,r in [('onboard before',before),('onboard after (#255)',after),('spheres before',bs),('spheres after',as_)]:
 for layout in sorted({c['config']['layout'] for c in r['cases']}):
  cases=[c for c in r['cases'] if c['config']['layout']==layout]
  med=lambda k:statistics.median(c['metrics'][k]['p95'] for c in cases)
  lines.append(f"| {label} / {layout} | {sum(c['frame_samples_retained'] for c in cases)} | {statistics.median(c['median_interval_fps'] for c in cases):.2f} | {med('frame_ms'):.3f} | {med('physics_total_ms'):.3f} | {med('cpu_render_ms'):.3f} | {med('gpu_latest_ms'):.3f} | {max(c['metrics']['frame_ms']['max'] for c in cases):.3f} | {med('draw_calls'):.0f} |")
lines += ['',f'Existing 20% same-device relative gates on each corresponding repeat: **{len(regressions)} regressions**. '+('Pass.' if not regressions else '**FAIL; inspect JSON before signoff.**'),'',
'Onboard windows: 2 s warmup + 30 s collection, preparation limit 2,400 steps. Sphere-deck/pile: 2 s + 8 s, same preparation limit. Seed 155, facing camera, minimum 100 completed intervals per window. Prepared sleep is verified throughout, not inferred from the requested state. 500 onboard pieces total 16 kg; 500 production mining spheres total 5,000 kg. The workloads are not interchangeable.','',
'Before binary: #236 digest `7dd87ac17e3545e5365a27adb562176bf35974666684d9ff4b5c06e462ad5953`; after: `a7e2bdbec8d32dff611e870daebf2b42465c507c7c1277c737203164f83dae4b`. Current onboard windows/raw files are reused from [#255](../issue-255/settled/), avoiding duplicate collection. Before/current sphere intervals and focus records live in [benchmarks](benchmarks/). Machine/build metadata and their limits are in the report.','',
'Raw spikes and p99 remain reviewable; passing p95 gates does not establish zero stalls or a physical display FPS floor. GPU latest-pass values can repeat asynchronously. Metal allocator residency is unavailable; upload/scratch/texture budgets are not total GPU resident memory. Active performance fails separately under #288.','',
'## Offscreen Metal terrain pass costs','',
'512×384 synthetic fixtures, 30 GPU samples per scene, debug test harness, identical geometry. These are pass timings, not native gameplay FPS.','',
'| Scene | Before median / p95 ms | After median / p95 ms |','| --- | ---: | ---: |']
terrain=json.loads((src/'terrain-comparison/comparison.json').read_text())
for s in terrain['scenes']:
 if s['scene'].endswith(('near','far','orbit')):
  a=s['before'];b=s['after'];lines.append(f"| {s['scene']} | {float(a['median_gpu_ms']):.3f} / {float(a['p95_gpu_ms']):.3f} | {float(b['median_gpu_ms']):.3f} / {float(b['p95_gpu_ms']):.3f} |")
(root/'performance.md').write_text('\n'.join(lines)+'\n')
print('Relative regressions:',regressions)
