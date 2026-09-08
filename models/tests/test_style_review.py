"""Deterministic style signals do not weaken hard contracts."""
import copy
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from style_review import report, compare
from validate_asset import read_document, validate_document, ValidationError

REPO=Path(__file__).resolve().parents[2]


def fixture(normals=True):
    values=[0,0,0,1,0,0,0,1,0]+[0,0,1]*3
    data=struct.pack('<18f',*values)
    doc={'asset':{'version':'2.0'},'nodes':[{'name':'Visual','mesh':0}],
         'meshes':[{'primitives':[{'attributes':{'POSITION':0,**({'NORMAL':1} if normals else {})}}]}],
         'materials':[], 'bufferViews':[{'buffer':0,'byteOffset':0,'byteLength':36},
                                       {'buffer':0,'byteOffset':36,'byteLength':36}],
         'accessors':[{'bufferView':0,'componentType':5126,'count':3,'type':'VEC3'},
                      {'bufferView':1,'componentType':5126,'count':3,'type':'VEC3'}]}
    manifest={'assetId':'resource.fixture','category':'resources',
              'styleProfile':{'name':'option-a-v1'}}
    return doc,data,manifest


class StyleReviewTests(unittest.TestCase):
    def test_opt_in_and_flat_normals(self):
        d,b,m=fixture()
        r=report(d,b,72,m)
        self.assertEqual(r['measurements']['flatTriangles'],1)
        self.assertEqual(r,report(d,b,72,m))
        del m['styleProfile']
        self.assertEqual(report(d,b,72,m)['measurements'],{})

    def test_smooth_missing_normals_and_reasoned_exception(self):
        d,b,m=fixture()
        b=struct.pack('<18f',0,0,0,1,0,0,0,1,0,*([0,.6,.8]*3))
        self.assertIn('smooth-geology',[f['code'] for f in report(d,b,72,m)['warnings']])
        m['styleProfile']['exceptions']=[{'code':'smooth-geology','reason':'Functional spherical fragment'}]
        r=report(d,b,72,m)
        self.assertFalse(r['warnings'])
        self.assertEqual(r['exceptions'][0]['reason'],'Functional spherical fragment')
        d,b,m=fixture(False)
        self.assertEqual(report(d,b,72,m)['warnings'][0]['code'],'missing-normals')

    def test_reuse_lod_and_micro_geometry(self):
        d,b,m=fixture()
        d['meshes'].append(copy.deepcopy(d['meshes'][0]))
        d['nodes']=[{'name':'LOD0','mesh':0},{'name':'LOD1','mesh':1},{'name':'Instance','mesh':0}]
        m['lods']=[{'node':'LOD0'},{'node':'LOD1'}]
        r=report(d,b,72,m)
        self.assertEqual(r['measurements']['reusedMeshReferences'],1)
        self.assertEqual({f['code'] for f in r['warnings']},{'mesh-reuse','lod-cost'})
        d,b,m=fixture()
        b=struct.pack('<18f',0,0,0,.0001,0,0,0,.0001,0,*([0,0,1]*3))
        self.assertIn('micro-geometry',[f['code'] for f in report(d,b,72,m)['warnings']])

    def test_comparison_and_categories(self):
        d,b,m=fixture()
        a=report(d,b,72,m);c=report(d,b,68,m)
        self.assertEqual(compare(a,c)['delta']['glbBytes'],-4)
        m['category']='ships'
        self.assertEqual(report(d,b,72,m)['measurements']['microAreaSquareMeters'],.0001)
        c['assetId']='other'
        with self.assertRaises(ValueError):compare(a,c)

    def test_existing_pipeline_includes_style_only_when_opted_in(self):
        import process_glb
        m=json.loads((REPO/'models/assets/items/mining-tool/manifest.json').read_text())
        m.pop('styleProfile', None)  # Exercise opt-out even after production adopts it.
        with tempfile.TemporaryDirectory() as directory:
            destination=Path(directory)/'candidate.glb'
            with patch.object(process_glb,'compliance',return_value={
                    'issues':{'numErrors':0,'numWarnings':0}}):
                plain=process_glb.process(m,REPO/m['runtime'],destination)
                self.assertNotIn('styleReview',plain)
                m['styleProfile']={'name':'option-a-v1'}
                reviewed=process_glb.process(m,REPO/m['runtime'],destination)
                self.assertEqual(reviewed['styleReview']['before'],reviewed['styleReview']['after'])
                self.assertEqual(destination.read_bytes(),(REPO/m['runtime']).read_bytes())

    def test_schema_opt_in_and_hard_budgets_remain(self):
        import jsonschema
        schema=json.loads((REPO/'models/asset-manifest.schema.json').read_text())
        jsonschema.Draft202012Validator.check_schema(schema)
        m=json.loads((REPO/'models/assets/items/mining-tool/manifest.json').read_text())
        jsonschema.validate(m,schema)
        m['styleProfile']={'name':'option-a-v1','exceptions':[{'code':'triangle-cost','reason':'Readable close-up tool'}]}
        jsonschema.validate(m,schema)
        bad=copy.deepcopy(m);bad['styleProfile']['exceptions'][0]['reason']=''
        with self.assertRaises(jsonschema.ValidationError):jsonschema.validate(bad,schema)
        bad=copy.deepcopy(m);bad['styleProfile']['name']='unknown'
        with self.assertRaises(jsonschema.ValidationError):jsonschema.validate(bad,schema)
        m['budgets']['maxTriangles']=1
        with self.assertRaisesRegex(ValidationError,'maxTriangles'):
            validate_document(REPO/m['runtime'],m)


if __name__=='__main__':unittest.main()
