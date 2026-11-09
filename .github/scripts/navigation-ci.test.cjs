'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { resolve } = require('node:path');
const { GROUPS, reportFailures } = require('./navigation-ci.cjs');
const root = resolve(__dirname, '../..');
const read = path => readFileSync(resolve(root, path), 'utf8');

function fixture({ jobs = [], issues = [], comments = [], eventName = 'schedule', repo = 'salimon', runAttempt = 1 } = {}) {
  const calls = [];
  const endpoints = { actions: { listJobsForWorkflowRun: 'jobs' }, issues: {
    listForRepo: 'issues', listComments: 'comments',
    create: async args => { calls.push({ action: 'create', ...args }); return { data: { number: 10, ...args } }; },
    createComment: async args => { calls.push({ action: 'comment', ...args }); },
  } };
  const input = { github: { rest: endpoints, paginate: async (endpoint, args) => {
    assert.equal(args.owner, 'mr-exception'); assert.equal(args.repo, 'salimon');
    assert.equal(args.per_page, 100);
    return ({ jobs, issues, comments })[endpoint];
  } }, context: { repo: { owner: 'mr-exception', repo }, eventName,
    runId: 123, sha: 'abc123', ref: 'refs/heads/main', serverUrl: 'https://github.com' },
  core: { info() {} }, runAttempt };
  return { calls, run: () => reportFailures(input) };
}
const failed = (name = 'Full regression / world', conclusion = 'failure') => ({ name, conclusion,
  html_url: 'https://github.com/mr-exception/salimon/actions/runs/123/job/456' });

test('daily full workflow covers every Playwright project exactly once and preserves all negative/repeat checks', () => {
  const full = read('.github/workflows/navigation-client-full.yml');
  const config = read('client/playwright.config.ts').match(/projects:\s*\[([\s\S]*?)\],\s*webServer:/)[1];
  const expected = [...config.matchAll(/\bname:\s*['"]([^'"]+)['"]/g)].map(m => m[1]).sort();
  const actual = [...full.matchAll(/projects:\s*'([^']+)'/g)].flatMap(m => [...m[1].matchAll(/--project=([\w-]+)/g)].map(p => p[1])).sort();
  assert.deepEqual(actual, expected);
  assert.deepEqual([...full.matchAll(/- group: (\w+)/g)].map(m => m[1]), GROUPS);
  assert.match(full, /cron: '17 3 \* \* \*'/);
  assert.match(full, /workflow_dispatch:/); assert.doesNotMatch(full, /pull_request:/);
  assert.match(full, /fail-fast: false/); assert.match(full, /cancel-in-progress: false/);
  for (const command of ['npm test', 'npm run build', 'test:browser:negative', 'test:visual', 'test:visual:negative', 'test:ui', 'test:ui:negative']) {
    assert.ok(full.includes(`run: ${command.startsWith('npm ') ? command : 'npm run ' + command}\n`), command);
  }
  assert.equal((full.match(/issues: write/g) ?? []).length, 1);
  assert.match(full.split('  report:')[1], /issues: write/);
  assert.match(full, /if: \$\{\{ always\(\) &&/);
  // Re-running only a failed browser job must still find the original build.
  assert.equal((full.match(/name: navigation-full-build-\$\{\{ github.run_id \}\}/g) ?? []).length, 2);
  assert.doesNotMatch(full, /navigation-full-build-.*github.run_attempt/);
});

test('PR build check keeps domain/build coverage and selects only bounded browser smoke', () => {
  const pr = read('.github/workflows/navigation-client.yml');
  assert.match(pr, /  build:\n/); assert.match(pr, /timeout-minutes: 15/);
  assert.ok(pr.includes('run: npm test\n')); assert.ok(pr.includes('run: npm run build\n'));
  assert.ok(pr.includes('run: npm run test:browser:smoke\n'));
  assert.doesNotMatch(pr, /run: npm run test:(?:ui|visual|browser:negative)\b/);
  assert.doesNotMatch(pr, /issues: write/);
  const smoke = JSON.parse(read('client/package.json')).scripts['test:browser:smoke'];
  for (const selector of ['production.spec.ts', 'deterministic.spec.ts', 'flight.spec.ts', '--project=production', '--project=deterministic', '--project=cockpit', 'built production|NAV-2|FLIGHT-838']) assert.ok(smoke.includes(selector), selector);
  assert.match(pr, /name: navigation-evidence-.*github.run_attempt/);
});

test('successful, skipped and unrelated jobs produce no issue', async () => {
  const f = fixture({ jobs: [failed(undefined, 'success'), failed('Report full regression failures')] });
  await f.run(); assert.deepEqual(f.calls, []);
});

test('each failed/timed-out group creates a scoped bug with revision and evidence links', async () => {
  const f = fixture({ jobs: [failed(), failed('Domain and build', 'timed_out')] });
  await f.run(); assert.equal(f.calls.length, 2);
  for (const call of f.calls) {
    assert.equal(call.action, 'create'); assert.equal(call.owner, 'mr-exception'); assert.equal(call.repo, 'salimon');
    assert.deepEqual(call.labels, ['bug']); assert.match(call.body, /abc123/);
    assert.match(call.body, /actions\/runs\/123/); assert.match(call.body, /14-day retention/);
  }
});

test('recurring failure comments on the existing bug without replacing human metadata', async () => {
  const issue = { number: 9, body: '<!-- salimon-full-ci:Full regression / world -->\nHuman diagnosis', labels: ['bug'], assignees: ['owner'] };
  const before = structuredClone(issue), f = fixture({ jobs: [failed()], issues: [issue] });
  await f.run(); assert.equal(f.calls.length, 1); assert.equal(f.calls[0].action, 'comment');
  assert.equal(f.calls[0].issue_number, 9); assert.deepEqual(issue, before);
});

test('same-attempt retries are idempotent for initial issue and subsequent comment reports', async () => {
  const marker = '<!-- salimon-full-ci:Full regression / world -->';
  const attempt = '<!-- salimon-full-ci-run:123:1 -->';
  for (const [body, comments] of [[`${marker}\n${attempt}`, []], [marker, [{ body: attempt }]]]) {
    const f = fixture({ jobs: [failed()], issues: [{ number: 9, body }], comments });
    await f.run(); assert.deepEqual(f.calls, []);
  }
  const f = fixture({ jobs: [failed()], issues: [{ number: 9, body: `${marker}\n${attempt}` }], runAttempt: 2 });
  await f.run(); assert.equal(f.calls.length, 1);
});

test('a PR carrying the marker is not mistaken for a bug issue', async () => {
  const f = fixture({ jobs: [failed()], issues: [{ body: '<!-- salimon-full-ci:Full regression / world -->', pull_request: {} }] });
  await f.run(); assert.equal(f.calls[0].action, 'create');
});

test('reporting refuses other repositories, PR events and invalid attempts before any API write', async () => {
  for (const options of [{ repo: 'other' }, { eventName: 'pull_request' }, { runAttempt: 0 }]) {
    const f = fixture({ jobs: [failed()], ...options });
    await assert.rejects(f.run); assert.deepEqual(f.calls, []);
  }
});
