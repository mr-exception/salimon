import {test} from "node:test";
import assert from "node:assert/strict";
import {candidate,textualBlockers,checksPassed} from "./policy";
test("blocker section ends at next heading and deduplicates",()=>{
  assert.deepEqual(textualBlockers("## Blocked by\n- #12\n- #12\n- https://github.com/a/b/issues/45\n## Acceptance\n#99"),[12,45]);
});
test("human and held issues are never candidates",()=>{
  const issue={number:1,title:"Example",body:"",labels:[{name:"auto-verification"}]};
  assert.equal(candidate(issue,"auto-verification"),true);
  assert.equal(candidate({...issue,labels:[...issue.labels,{name:"human-verification"}]},"auto-verification"),false);
  assert.equal(candidate({...issue,pull_request:{}},"auto-verification"),false);
});
test("merge requires actual successful completed checks and successful legacy status",()=>{
  assert.equal(checksPassed([],"success"),false);
  assert.equal(checksPassed([{status:"in_progress",conclusion:null}],"success"),false);
  assert.equal(checksPassed([{status:"completed",conclusion:"failure"}],"success"),false);
  assert.equal(checksPassed([{status:"completed",conclusion:"skipped"}],"success"),false);
  assert.equal(checksPassed([{status:"completed",conclusion:"success"}],"pending"),false);
  assert.equal(checksPassed([{status:"completed",conclusion:"success"}],"success"),true);
});
