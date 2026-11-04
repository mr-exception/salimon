// Read-only gate. Agents must never post the human approval this checks for.
import {readFile,mkdir,writeFile} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {sha256,verifyApproval} from './visual-comparison.ts'

const path='tests/navigation-baselines/manifest.json'
const manifestBytes=await readFile(path),manifest=JSON.parse(manifestBytes.toString()),digest=sha256(manifestBytes)
for(const [name,expected] of Object.entries(manifest.images)) {
 if(!/^[a-z-]+\.png$/.test(name))throw Error('Invalid baseline path')
 if(sha256(await readFile(`tests/navigation-baselines/${name}`))!==expected)throw Error(`Baseline hash mismatch: ${name}`)
}
const base=process.env.SALIMON_BASE_SHA
if(!base||!/^[a-f0-9]{40}$/.test(base))throw Error('Expected SALIMON_BASE_SHA (exact PR base)')
let inherited=false
try {inherited=execFileSync('git',['show',`${base}:client/${path}`],{stdio:['ignore','pipe','ignore']}).equals(manifestBytes)}catch{/* Initial baseline needs approval. */}
const comments:Parameters<typeof verifyApproval>[0]=[]
if(!inherited) {
 const pr=process.env.SALIMON_PR_NUMBER,token=process.env.GITHUB_TOKEN
 if(!pr||!/^\d+$/.test(pr)||!token)throw Error(`BASELINE_APPROVAL_REQUIRED: post human review "Approve navigation baselines: ${digest}" to this PR, then rerun the gate`)
 for(let page=1;;page++) {
  const response=await fetch(`https://api.github.com/repos/mr-exception/salimon/issues/${pr}/comments?per_page=100&page=${page}`,{headers:{Authorization:`Bearer ${token}`,Accept:'application/vnd.github+json'}})
  if(!response.ok)throw Error(`Cannot read baseline approvals: HTTP ${response.status}`)
  const batch=await response.json();if(!Array.isArray(batch))throw Error('Invalid approval response')
  comments.push(...batch);if(batch.length<100)break
 }
}
const approved=inherited||verifyApproval(comments,digest)
await mkdir('test-results',{recursive:true})
await writeFile('test-results/baseline-approval.json',JSON.stringify({digest,base,inherited,approved,scope:'Human baseline approval; independent implementation QA remains separate.'},null,2)+'\n')
if(!approved)throw Error(`BASELINE_APPROVAL_REQUIRED: human OWNER/MEMBER/COLLABORATOR must post exactly "Approve navigation baselines: ${digest}"; candidates are not approved references`)
console.log(`Baseline approval PASS (${inherited?'inherited from base':'explicit human review'}): ${digest}`)
