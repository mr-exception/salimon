import { readJSON, initial } from "../../../src/store";
import { config } from "../../../src/config";
import { command } from "../../../src/process";
export const dynamic="force-dynamic";
export async function GET() {
  const [state,control,user,docker]=await Promise.all([
    readJSON("state.json",initial()),readJSON("control.json",{enabled:false}),
    command("gh",["api","user","--jq",".login"]).then(login=>({login,error:null})).catch((e:Error)=>({login:null,error:e.message})),
    command("docker",["info","--format","{{.ServerVersion}}"],10000).then(()=>({ok:true,error:null})).catch((e:Error)=>({ok:false,error:e.message})),
  ]);
  return Response.json({...state,...control,github:user,docker,config:{repo:config.repo,concurrency:config.concurrency}},{headers:{"Cache-Control":"no-store"}});
}
