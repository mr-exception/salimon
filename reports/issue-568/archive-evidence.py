#!/usr/bin/env python3
"""Losslessly archive verification files with hash-deduplicated tar hardlinks."""
import argparse,hashlib,json,tarfile,gzip,lzma
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('root',type=Path);p.add_argument('output',type=Path);p.add_argument('--members',type=Path);a=p.parse_args()
files=sorted((a.root/x for x in json.loads(a.members.read_text())) if a.members else (x for x in a.root.rglob('*') if x.is_file()));seen={};inventory={};a.output.parent.mkdir(parents=True,exist_ok=True)
with lzma.open(a.output.with_suffix('.tar.xz'),'wb',format=lzma.FORMAT_XZ,filters=[{'id':lzma.FILTER_LZMA2,'dict_size':64*1024*1024,'preset':3}]) as compressed,tarfile.open(fileobj=compressed,mode='w|') as archive:
 for path in files:
  name=path.relative_to(a.root).as_posix();data=path.read_bytes();sha=hashlib.sha256(data).hexdigest()
  binary=data.startswith((b'\x7fELF',b'MZ',b'\xcf\xfa\xed\xfe',b'\xfe\xed\xfa\xcf'))
  entry={'sha256':sha,'bytes':len(data),'archived':not binary}
  if binary:entry['reason']='compiled executable retained locally; reproduce from frozen source/commands; digest retained'
  else:
   info=archive.gettarinfo(str(path),arcname=name);info.uid=info.gid=0;info.uname=info.gname=''
   if sha in seen:
    info.type=tarfile.LNKTYPE;info.linkname=seen[sha];info.size=0;archive.addfile(info);entry['hardlink_to']=seen[sha]
   else:
    with path.open('rb') as stream:archive.addfile(info,stream)
    seen[sha]=name
  inventory[name]=entry
result={'source_root':str(a.root),'files':inventory,'archive_sha256':hashlib.sha256(a.output.with_suffix('.tar.xz').read_bytes()).hexdigest(),'deduplicated_files':sum('hardlink_to' in x for x in inventory.values())}
full=a.output.with_suffix('.inventory.json.gz');full.write_bytes(gzip.compress((json.dumps(result,indent=2)+'\n').encode(),mtime=0))
summary={'archive':a.output.with_suffix('.tar.xz').name,'archive_sha256':result['archive_sha256'],'file_count':len(files),'archived_files':sum(v['archived'] for v in inventory.values()),'deduplicated_files':result['deduplicated_files'],'full_inventory':full.name,'full_inventory_sha256':hashlib.sha256(full.read_bytes()).hexdigest()}
a.output.with_suffix('.inventory.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'files':len(files),'unique_archived_contents':len(seen),'archive_bytes':a.output.with_suffix('.tar.xz').stat().st_size}))
