import { command } from "./process";
import { config } from "./config";
import { candidate, textualBlockers, type Issue } from "./policy";
export async function gh<T>(endpoint:string,method="GET",fields:Record<string,string>={}) : Promise<T> {
  const args=["api",endpoint,"--method",method];
  for(const [k,v] of Object.entries(fields)) args.push("-f",`${k}=${v}`);
  return JSON.parse(await command("gh",args));
}
export async function pages<T>(endpoint:string):Promise<T[]> {
  const all:T[]=[];
  for(let page=1;page<=100;page++) {
    const batch=await gh<T[]>(`${endpoint}${endpoint.includes("?")?"&":"?"}per_page=100&page=${page}`);
    all.push(...batch); if(batch.length<100) return all;
  }
  throw Error("GitHub pagination exceeded 100 pages; narrow the issue label");
}
export async function eligible(issue:Issue) {
  if(!candidate(issue,config.label)) return false;
  const native=await pages<{state:string}>(`repos/${config.repo}/issues/${issue.number}/dependencies/blocked_by`);
  if(native.some(i=>i.state!=="closed")) return false;
  // Cross-repository textual blockers need their own repository lookup.
  const body=issue.body||"";
  const section=body.match(/(?:^|\n)#{1,6}\s*blocked by\s*\r?\n([\s\S]*?)(?=\n#{1,6}\s|$)/i)?.[1]||"";
  for(const url of section.matchAll(/https:\/\/github\.com\/([\w.-]+\/[\w.-]+)\/issues\/(\d+)/g)) {
    if((await gh<{state:string}>(`repos/${url[1]}/issues/${url[2]}`)).state!=="closed") return false;
  }
  const localSection=section.replace(/https:\/\/github\.com\/[^\s)]+/g,"");
  for(const number of textualBlockers(`## Blocked by\n${localSection}`)) {
    if((await gh<{state:string}>(`repos/${config.repo}/issues/${number}`)).state!=="closed") return false;
  }
  return true;
}
