import { mkdir, readFile, writeFile, rename } from "node:fs/promises";
import path from "node:path";
import { randomUUID } from "node:crypto";
import { config } from "./config";
export type Job = { id: string; issue: number; title: string; branch: string; status: "starting" | "running" | "ci" | "merged" | "failed"; container?: string; pr?: number; url?: string; error?: string; updatedAt: string };
export type State = { heartbeat: string | null; jobs: Job[]; events: {at:string;message:string}[]; queue: {number:number;title:string;reason?:string}[] };
export const initial = (): State => ({heartbeat:null,jobs:[],events:[],queue:[]});
export async function readJSON<T>(file: string, fallback: T): Promise<T> {
  try { return JSON.parse(await readFile(path.join(config.data,file),"utf8")); }
  catch (e) { if ((e as NodeJS.ErrnoException).code === "ENOENT") return fallback; throw e; }
}
export async function writeJSON(file: string, value: unknown) {
  await mkdir(config.data,{recursive:true});
  const target = path.join(config.data,file), temp = `${target}.${randomUUID()}.tmp`;
  await writeFile(temp,JSON.stringify(value,null,2),{mode:0o600});
  await rename(temp,target);
}
export function event(state:State,message:string) { state.events.unshift({at:new Date().toISOString(),message}); state.events = state.events.slice(0,200); }
