import { writeJSON } from "../../../src/store";
export async function POST(request:Request) {
  const origin=request.headers.get("origin");
  if(!origin || origin!==new URL(request.url).origin) return Response.json({error:"Same-origin request required"},{status:403});
  const {action,id}=await request.json().catch(()=>({}));
  if(action==="start" || action==="pause") await writeJSON("control.json",{enabled:action==="start"});
  else if(action==="retry" && typeof id==="string" && /^[a-f0-9-]{36}$/.test(id)) await writeJSON(`retry-${id}.json`,{requested:true});
  else return Response.json({error:"Invalid action"},{status:400});
  return Response.json({ok:true});
}
