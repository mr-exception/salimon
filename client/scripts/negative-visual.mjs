// Test a real CSS defect through the running App and screenshot comparator.
import {cp,mkdtemp,mkdir,readFile,writeFile,symlink,rm} from 'node:fs/promises'
import {spawnSync} from 'node:child_process'
import {tmpdir} from 'node:os'
import {join,resolve,basename} from 'node:path'
import {createHash} from 'node:crypto'
const root=process.cwd(),output=resolve(process.env.SALIMON_VISUAL_NEGATIVE_DIR??'test-results/visual-negative')
const scratch=await mkdtemp(join(tmpdir(),'salimon-visual-negative-'))
const hash=s=>createHash('sha256').update(s).digest('hex')
try {
 await cp(root,scratch,{recursive:true,filter:path=>!['node_modules','test-results','playwright-report','.git'].includes(basename(path))})
 await symlink(join(root,'node_modules'),join(scratch,'node_modules'),'dir')
 // Preserve revision metadata without making the copy a mutable repository.
 const git=await readFile(join(root,'.revision-for-negative'),'utf8').catch(()=>null)
 const source=git??spawnSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).stdout.trim()
 await writeFile(join(scratch,'.revision-for-negative'),source)
 const target=join(scratch,'src/styles.css'),original=await readFile(target,'utf8')
 const before='[data-vector=heading]{stroke:#ffe36a;',after='[data-vector=heading]{stroke:#ff0000;'
 if(original.split(before).length!==2)throw Error('Visual mutation must match exactly once')
 const changed=original.replace(before,after);await writeFile(target,changed)
 const child=spawnSync(process.execPath,[join(root,'node_modules/@playwright/test/cli.js'),'test','--project=navigation-captures'],{cwd:scratch,env:{...process.env,CI:'true',SALIMON_CAPTURE_CANDIDATES:'',SALIMON_EVIDENCE_DIR:output},encoding:'utf8',timeout:240000,maxBuffer:2e6})
 await mkdir(output,{recursive:true});await writeFile(join(output,'command.log'),`${child.stdout??''}\n${child.stderr??''}`)
 const report=JSON.parse(await readFile(join(output,'results.json'),'utf8'))
 const specs=suite=>[...(suite.specs??[]),...(suite.suites??[]).flatMap(specs)]
 const results=report.suites.flatMap(specs).flatMap(s=>s.tests??[]).flatMap(t=>t.results??[])
 const attachments=results[0]?.attachments??[]
 const summaryAttachment=attachments.find(a=>a.name==='comparison-summary.json')
 const summary=summaryAttachment?JSON.parse(summaryAttachment.body?Buffer.from(summaryAttachment.body,'base64').toString():await readFile(summaryAttachment.path,'utf8')):null
 const errors=results.flatMap(r=>r.errors??[]).map(e=>e.message??'').join('\n')
 const detected=child.status===1&&!child.error&&report.stats.unexpected===1&&results.length===1&&results[0].status==='failed'&&summary?.results.length===8&&summary.results.filter(r=>!r.pass).length===2&&summary.results.filter(r=>!r.pass).every(r=>['heading-velocity','opposite-vectors'].includes(r.id)&&r.changedPixels>100)&&errors.includes('VISUAL_PIXEL_MISMATCH heading-velocity')&&errors.includes('VISUAL_PIXEL_MISMATCH opposite-vectors')
 await writeFile(join(output,'mutation-summary.json'),JSON.stringify({status:detected?'PASS':'INVALID_DEMONSTRATION',source,file:'src/styles.css',original_sha256:hash(original),mutated_sha256:hash(changed),mutation:{before,after},exit:child.status,comparison:summary},null,2)+'\n')
 if(!detected)throw Error(`Visual defect did not fail only the intended comparisons: ${output}`)
 console.log('VIS-1 PASS: red heading fails two real browser screenshots; six unchanged screens pass; originals/diffs/trace retained')
} finally {await rm(scratch,{recursive:true,force:true})}
