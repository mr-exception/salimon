#!/usr/bin/env python3
"""Verify every archived original and lossless hardlink by SHA-256, without extraction."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;results=[]
for index in sorted(root.glob('*.inventory.json')):
 summary=json.loads(index.read_text());full=root/summary['full_inventory'];assert hashlib.sha256(full.read_bytes()).hexdigest()==summary['full_inventory_sha256']
 inventory=json.loads(gzip.decompress(full.read_bytes()));archive=root/summary['archive'];assert hashlib.sha256(archive.read_bytes()).hexdigest()==summary['archive_sha256'];seen={}
 with tarfile.open(archive,'r|*') as stream:
  for member in stream:
   if member.islnk():sha=seen[member.linkname]
   else:
    h=hashlib.sha256();file=stream.extractfile(member)
    while chunk:=file.read(1024*1024):h.update(chunk)
    sha=h.hexdigest()
   assert sha==inventory['files'][member.name]['sha256'],member.name;seen[member.name]=sha
 assert set(seen)=={k for k,v in inventory['files'].items() if v['archived']}
 results.append({'archive':archive.name,'verified_original_files':len(seen),'executable_digests_only':sum(not v['archived'] for v in inventory['files'].values()),'status':'passed'});print(archive.name,len(seen),flush=True)
(root/'archive-file-verification.json').write_text(json.dumps(results,indent=2)+'\n')
