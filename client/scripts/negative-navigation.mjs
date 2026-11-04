// Run controlled source regressions in disposable copies. The working tree and
// approved baselines are never edited. A crash/timeout is not a detected defect.
import {cp,mkdtemp,mkdir,readFile,writeFile,symlink,rm} from 'node:fs/promises'
import {spawnSync} from 'node:child_process'
import {tmpdir} from 'node:os'
import {join,resolve,basename} from 'node:path'
import {createHash} from 'node:crypto'

const root=process.cwd(),output=resolve(process.env.SALIMON_NEGATIVE_DIR??'test-results/negative')
await mkdir(output,{recursive:true})
const mutations=[
 {id:'reversed-wasd',file:'src/CameraControls.tsx',before:'multiplyScalar(x*step)',after:'multiplyScalar(-x*step)',test:'NAV-1 seeded replay',assertion:'NAV-1 a moves the correct screen direction'},
 {id:'wrong-heading-color',file:'src/styles.css',before:'[data-vector=heading]{stroke:#ffe36a;',after:'[data-vector=heading]{stroke:#ff0000;',test:'NAV-3 heading and velocity',assertion:'NAV-3 heading is yellow'}
]
const hash=s=>createHash('sha256').update(s).digest('hex')
const records=[]
for(const mutation of mutations) {
 const scratch=await mkdtemp(join(tmpdir(),'salimon-negative-'))
 try {
  await cp(root,scratch,{recursive:true,filter:path=>!['node_modules','test-results','playwright-report','.git'].includes(basename(path))})
  await symlink(join(root,'node_modules'),join(scratch,'node_modules'),'dir')
  const target=join(scratch,mutation.file),original=await readFile(target,'utf8')
  if(original.split(mutation.before).length!==2)throw Error(`Mutation no longer matches exactly once: ${mutation.id}`)
  const changed=original.replace(mutation.before,mutation.after)
  await writeFile(target,changed)
  const evidence=join(output,mutation.id)
  const child=spawnSync(process.execPath,[join(root,'node_modules/@playwright/test/cli.js'),'test','--project=deterministic','--grep',mutation.test],{cwd:scratch,env:{...process.env,CI:'true',SALIMON_EVIDENCE_DIR:evidence},encoding:'utf8',timeout:180000,maxBuffer:2e6})
  await mkdir(evidence,{recursive:true})
  await writeFile(join(evidence,'command.log'),`${child.stdout??''}\n${child.stderr??''}`)
  const report=JSON.parse(await readFile(join(evidence,'results.json'),'utf8'))
  const specs=suite=>[...(suite.specs??[]),...(suite.suites??[]).flatMap(specs)]
  const results=report.suites.flatMap(specs).flatMap(s=>s.tests??[]).flatMap(t=>t.results??[])
  const errors=results.flatMap(r=>r.errors??[]).map(e=>e.message??'').join('\n')
  const detected=child.status===1&&report.stats.unexpected===1&&results.length===1&&results[0].status==='failed'&&errors.includes(mutation.assertion)&&!child.error
  records.push({id:mutation.id,file:mutation.file,original_sha256:hash(original),mutated_sha256:hash(changed),exit:child.status,assertion:mutation.assertion,result:detected?'EXPECTED_FAILURE':'INVALID_DEMONSTRATION',report:join(mutation.id,'results.json')})
  await writeFile(join(output,'summary.json'),JSON.stringify({schema_version:1,status:records.every(r=>r.result==='EXPECTED_FAILURE')?'PASS':'FAIL',mutations:records},null,2)+'\n')
  if(!detected)throw Error(`Expected assertion was not detected: ${mutation.id}. See ${evidence}`)
  console.log(`${mutation.id}: expected assertion failure retained`)
 } finally {await rm(scratch,{recursive:true,force:true})}
}
