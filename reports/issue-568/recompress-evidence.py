#!/usr/bin/env python3
"""Change only the archive wrapper; verify identical original tar bytes."""
import gzip,lzma,json,hashlib
from pathlib import Path
root=Path(__file__).resolve().parent
records=[]
sources=sorted(root.glob('*.tar.gz'))
if not sources: raise SystemExit('No gzip archives to convert; existing verification is preserved.')
for source in sources:
 target=source.with_suffix('').with_suffix('.tar.xz');h=hashlib.sha256()
 with gzip.open(source,'rb') as a,lzma.open(target,'wb',format=lzma.FORMAT_XZ,filters=[{'id':lzma.FILTER_LZMA2,'dict_size':64*1024*1024,'preset':3}]) as b:
  while chunk:=a.read(1024*1024):h.update(chunk);b.write(chunk)
 check=hashlib.sha256()
 with lzma.open(target,'rb') as stream:
  while chunk:=stream.read(1024*1024):check.update(chunk)
 assert h.digest()==check.digest(),source
 base=source.name[:-7];full=root/(base+'.inventory.json.gz');inventory=json.loads(gzip.decompress(full.read_bytes()))
 original=inventory['archive_sha256'];inventory['archive_sha256']=hashlib.sha256(target.read_bytes()).hexdigest();inventory['compression']={'format':'xz','dictionary_bytes':64*1024*1024,'preset':3,'tar_sha256':h.hexdigest(),'recompressed_from_sha256':original}
 full.write_bytes(gzip.compress((json.dumps(inventory,indent=2)+'\n').encode(),mtime=0))
 summary_path=root/(base+'.inventory.json');summary=json.loads(summary_path.read_text());summary.update(archive=target.name,archive_sha256=inventory['archive_sha256'],full_inventory_sha256=hashlib.sha256(full.read_bytes()).hexdigest());summary_path.write_text(json.dumps(summary,indent=2)+'\n')
 records.append({'archive':target.name,'original_tar_bytes_verified':True,'tar_sha256':h.hexdigest(),'gzip_bytes':source.stat().st_size,'xz_bytes':target.stat().st_size});source.unlink();print(target.name,target.stat().st_size,flush=True)
index=root/'linux-archive-index.json';rows=json.loads(index.read_text())
for r in rows:r['archive']=r['archive'].replace('.tar.gz','.tar.xz')
index.write_text(json.dumps(rows,indent=2)+'\n');(root/'archive-wrapper-verification.json').write_text(json.dumps(records,indent=2)+'\n')
