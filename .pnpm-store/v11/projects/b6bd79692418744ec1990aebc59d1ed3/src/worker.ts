import "@next/env";
import { loadEnvConfig } from "@next/env";
loadEnvConfig(process.cwd());
// Configuration is loaded after .env.local.
const {config}=await import("./config");
const {command}=await import("./process");
const {gh,pages,eligible}=await import("./github");
const {readJSON,writeJSON,initial,event}=await import("./store");
const {checksPassed}=await import("./policy");
import type {Job} from "./store";
import type {Issue} from "./policy";
import {mkdir,open,unlink,readFile} from "node:fs/promises";
import path from "node:path";
import {randomUUID} from "node:crypto";
await mkdir(config.data,{recursive:true});
// An exclusive lock prevents a second scheduler from claiming the same issues.
const lock=await open(path.join(config.data,"worker.lock"),"wx").catch(()=>{throw Error("Worker lock exists. Stop any running worker before removing .data/worker.lock.");});
const state=await readJSON("state.json",initial());
let stopping=false;
process.on("SIGINT",()=>{stopping=true;});
process.on("SIGTERM",()=>{stopping=true;});
const now=()=>new Date().toISOString();
async function save(){state.heartbeat=now();await writeJSON("state.json",state);}
async function fail(job:Job,error:unknown){job.status="failed";job.error=error instanceof Error?error.message:String(error);job.updatedAt=now();event(state,`Issue #${job.issue}: ${job.error}`);}
async function launch(issue:Issue) {
  const id=randomUUID();
  const job:Job={id,issue:issue.number,title:issue.title,branch:`agents/issue-${issue.number}-${id.slice(0,8)}`,status:"starting",container:`agent-control-${id}`,updatedAt:now()};
  state.jobs.unshift(job);await save();
  try {
    const auth=await readFile(path.join(config.codexHome,"auth.json"),"utf8");
    if(!JSON.parse(auth).tokens) throw Error("A file-backed ChatGPT Codex login is required");
    const prompt=`Work on ${config.repo} issue #${issue.number}. Read the issue and comments, AGENTS.md and affected guides. Resolve blockers before editing. Implement acceptance criteria, execute meaningful automated checks, and create reports/issue-${issue.number}/README.md recording real checks and limitations. Commit all intended changes on the existing branch. Do not create or merge a PR; the host does that. Do not change GitHub configuration or secrets. If blocked or tests fail, exit without a commit and describe the failure.`;
    const dir=path.join(config.data,id);await mkdir(dir,{recursive:true});
    // Copy only credentials, never host config, plugins, or Docker socket.
    await writeJSON(`${id}/auth.json`,JSON.parse(auth));
    await command("docker",["create","--name",job.container!,"--label","agent-control=true","--memory","6g","--cpus","2","--pids-limit","512","--cap-drop","ALL","--security-opt","no-new-privileges",config.image,config.repo,job.branch,prompt]);
    await command("docker",["cp",path.join(dir,"auth.json"),`${job.container}:/home/agent/.codex/auth.json`]);
    await unlink(path.join(dir,"auth.json"));
    const token=await command("gh",["auth","token"]);
    // docker cp via stdin avoids putting a token in argv or Docker inspection env.
    const {spawn}=await import("node:child_process");
    await new Promise<void>((resolve,reject)=>{
      const child=spawn("docker",["cp","-",`${job.container}:/home/agent/`],{windowsHide:true,stdio:["pipe","ignore","ignore"]});
      // A small USTAR archive containing the credential consumed by entrypoint.
      const contents=Buffer.from(token+"\n");const header=Buffer.alloc(512);
      header.write("github-token");header.write("0000600\0",100);header.write("0001750\0",108);header.write("0001750\0",116);
      header.write(contents.length.toString(8).padStart(11,"0")+"\0",124);header.write("00000000000\0",136);header.fill(32,148,156);header.write("0",156);header.write("ustar\0",257);
      const sum=header.reduce((a,b)=>a+b,0);header.write(sum.toString(8).padStart(6,"0")+"\0 ",148);
      child.once("error",reject);child.stdin.on("error",reject);child.once("close",code=>code===0?resolve():reject(Error("Credential transfer failed")));
      child.stdin.end(Buffer.concat([header,contents,Buffer.alloc((512-contents.length%512)%512+1024)]));
    });
    await command("docker",["start",job.container!]);job.status="running";event(state,`Worker started for #${job.issue}`);
  }catch(e){await fail(job,e);}
}
async function reconcile(job:Job){
  if(job.status==="starting") {await fail(job,Error("Startup interrupted; inspect the container before retrying"));return;}
  if(job.status==="running") {
    const info=JSON.parse(await command("docker",["inspect","--format","{{json .State}}",job.container!]));
    if(info.Running) return;
    if(info.ExitCode!==0) throw Error(`Worker exited ${info.ExitCode}; inspect docker logs ${job.container}`);
    const existing=await gh<{number:number;html_url:string}[]>(`repos/${config.repo}/pulls?state=open&head=${config.repo.split("/")[0]}:${job.branch}`);
    const pr=existing[0]||await gh<{number:number;html_url:string}>(`repos/${config.repo}/pulls`,"POST",{title:`Resolve #${job.issue}: ${job.title}`,head:job.branch,base:(await gh<{default_branch:string}>(`repos/${config.repo}`)).default_branch,body:`Closes #${job.issue}\n\nAutomated implementation. Evidence: reports/issue-${job.issue}/README.md. Review CI and repository policy before merge.`});
    job.pr=pr.number;job.url=pr.html_url;job.status="ci";event(state,`PR #${job.pr} opened for issue #${job.issue}`);
  }
  if(job.status==="ci") {
    const endpoint=`repos/${config.repo}/pulls/${job.pr}`;
    const pr=await gh<{merged:boolean;state:string;draft:boolean;head:{sha:string};mergeable_state:string}>(endpoint);
    if(pr.merged){job.status="merged";event(state,`PR #${job.pr} merged`);return;}
    if(pr.state!=="open") throw Error("PR was closed without merging");
    const checks=await gh<{check_runs:{status:string;conclusion:string|null}[]}>(`repos/${config.repo}/commits/${pr.head.sha}/check-runs?per_page=100`);
    const statuses=await gh<{state:string;total_count:number}>(`repos/${config.repo}/commits/${pr.head.sha}/status`);
    // Check-only repos have pending combined status with total_count=0.
    const status=statuses.total_count===0?"success":statuses.state;
    if(checks.check_runs.some(c=>c.status==="completed" && ["failure","cancelled","timed_out","action_required"].includes(c.conclusion||"")) || status==="failure") throw Error("CI failed; manual remediation or retry required");
    if(checks.check_runs.length===100) throw Error("More than one check page; merge withheld");
    if(pr.draft || pr.mergeable_state!=="clean" || !checksPassed(checks.check_runs,status)) return;
    const issue=await gh<Issue>(`repos/${config.repo}/issues/${job.issue}`);
    if(!await eligible(issue)) throw Error("Issue is no longer eligible; merge withheld");
    const merged=await gh<{merged:boolean}>(`${endpoint}/merge`,"PUT",{sha:pr.head.sha,merge_method:"squash"});
    if(!merged.merged) throw Error("GitHub did not confirm merge");
    job.status="merged";event(state,`PR #${job.pr} merged; moving to the next issue`);
  }
  job.updatedAt=now();
}
try {
  event(state,"Scheduler started");
  while(!stopping){
    try {
      const control=await readJSON("control.json",{enabled:false});
      if(control.enabled){
        await command("gh",["auth","status"]);await command("docker",["info","--format","{{.ServerVersion}}"]);
        for(const job of state.jobs.filter(j=>["starting","running","ci"].includes(j.status))){try{await reconcile(job);}catch(e){await fail(job,e);}await save();}
        for(const job of state.jobs.filter(j=>j.status==="failed")){
          const file=`retry-${job.id}.json`;
          if((await readJSON(file,{requested:false})).requested){
            await unlink(path.join(config.data,file));
            if(job.pr) {job.status="ci";job.error=undefined;}
            else { job.error="Retry requested; issue may be selected again";job.status="failed";state.jobs=state.jobs.filter(j=>j.id!==job.id); }
          }
        }
        const issues=await pages<Issue>(`repos/${config.repo}/issues?state=open&labels=${encodeURIComponent(config.label)}&sort=created&direction=asc`);
        state.queue=[];
        for(const issue of issues){
          if(state.jobs.some(j=>j.issue===issue.number))continue;
          const ready=await eligible(issue);state.queue.push({number:issue.number,title:issue.title,...(!ready?{reason:"Blocked or excluded by policy"}:{})});
          const active=state.jobs.filter(j=>["starting","running","ci"].includes(j.status)).length;
          if(ready && active<config.concurrency) {await launch(issue);}
        }
      }
    } catch(e){event(state,e instanceof Error?e.message:String(e));}
    await save();
    for(let waited=0;waited<config.pollMs && !stopping;waited+=250) await new Promise(r=>setTimeout(r,250));
  }
}finally{await save();await lock.close();await unlink(path.join(config.data,"worker.lock"));}
