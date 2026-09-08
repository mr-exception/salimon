export type Issue = {number:number;title:string;body:string|null;labels:{name:string}[];pull_request?:unknown};
export function textualBlockers(body:string): number[] {
  const section = body.match(/(?:^|\n)#{1,6}\s*blocked by\s*\r?\n([\s\S]*?)(?=\n#{1,6}\s|$)/i)?.[1] || "";
  return [...new Set([...section.matchAll(/#(\d+)|https:\/\/github\.com\/[^/]+\/[^/]+\/issues\/(\d+)/g)].map(m=>Number(m[1]||m[2])))];
}
export function candidate(issue:Issue,label:string) {
  return !issue.pull_request && issue.labels.some(l=>l.name===label) && !issue.labels.some(l=>/^(blocked|human-verification|agent:hold)$/i.test(l.name));
}
export type Check = {status?:string;conclusion?:string|null};
export function checksPassed(checks:Check[], status:string) {
  return checks.length > 0 && checks.every(c=>c.status === "completed" && c.conclusion === "success") && status === "success";
}
