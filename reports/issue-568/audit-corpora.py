#!/usr/bin/env python3
"""Read-only approval/context admission audit; never compare incompatible pixels."""
import argparse,json
from pathlib import Path
FIELDS=('backend','adapter','driver','platform','viewport_pixels','scale_factor')
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('captures',type=Path);p.add_argument('baselines',type=Path);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();capture=json.loads((a.captures/'corpus.json').read_text());rows=[]
for path in sorted(a.baselines.glob('*/corpus.json')):
 b=json.loads(path.read_text());review=b.get('review',{})
 reviewed=b.get('status')=='approved' and all(review.get(k) for k in ('reviewer','reviewed_at','evidence')) and review.get('masks_and_tolerances_reviewed') is True
 differences={k:{'baseline':b.get('context',{}).get(k),'capture':capture['context'].get(k)} for k in FIELDS if b.get('context',{}).get(k)!=capture['context'].get(k)}
 fixture_matches=b.get('fixture_contract_sha256')==capture['fixture_contract_sha256']
 eligible=bool(reviewed and not differences and fixture_matches)
 rows.append({'baseline':str(path),'approval':b.get('status'),'review_record_valid':bool(reviewed),'fixture_contract_matches':fixture_matches,'context_differences':differences,'eligible_for_pixel_comparison':eligible,'status':'eligible' if eligible else 'incompatible' if reviewed else 'pending-approval','pixel_comparison_performed':False})
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps({'capture_revision':capture['revision'],'capture_status':capture['status'],'context':capture['context'],'baselines':rows,'policy':'only an explicitly approved and fully context/fixture-compatible corpus may proceed to pixel comparison; no promotion/mask/tolerance changes'},indent=2)+'\n')
