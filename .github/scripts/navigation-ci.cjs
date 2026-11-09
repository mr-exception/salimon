'use strict';

const REPOSITORY = 'mr-exception/salimon';
const GROUPS = ['world', 'navigation', 'perspective', 'visual', 'expedition'];
const JOBS = new Set(['Domain and build', ...GROUPS.map(group => `Full regression / ${group}`)]);
const FAILURES = new Set(['failure', 'timed_out', 'cancelled', 'startup_failure', 'action_required', 'stale']);

/** Called only by the scheduled/manual full workflow's isolated write-permission job. */
async function reportFailures({ github, context, core, runAttempt = Number(process.env.GITHUB_RUN_ATTEMPT ?? '1') }) {
  const { owner, repo } = context.repo;
  if (`${owner}/${repo}` !== REPOSITORY) throw new Error('CI reporting is restricted to mr-exception/salimon');
  if (!['schedule', 'workflow_dispatch'].includes(context.eventName)) throw new Error('Only scheduled/manual full runs may report issues');
  if (!Number.isSafeInteger(runAttempt) || runAttempt < 1) throw new Error('Invalid run attempt');
  const jobs = await github.paginate(github.rest.actions.listJobsForWorkflowRun, {
    owner, repo, run_id: context.runId, filter: 'latest', per_page: 100,
  });
  const failed = jobs.filter(job => JOBS.has(job.name) && FAILURES.has(job.conclusion));
  if (!failed.length) { core.info('No failed full-regression groups; no issue written.'); return; }
  const issues = await github.paginate(github.rest.issues.listForRepo, { owner, repo, state: 'open', per_page: 100 });
  for (const job of failed) {
    const marker = `<!-- salimon-full-ci:${job.name} -->`;
    const attempt = `<!-- salimon-full-ci-run:${context.runId}:${runAttempt} -->`;
    const runUrl = `${context.serverUrl}/${REPOSITORY}/actions/runs/${context.runId}`;
    const body = [marker, attempt, `The daily full browser regression failed in **${job.name}**.`, '',
      `- Commit: ${context.sha}`, `- Ref: ${context.ref}`, `- Conclusion: ${job.conclusion}`,
      `- Run / attempt: [${context.runId} / ${runAttempt}](${runUrl})`,
      `- Failed job: [logs](${job.html_url})`,
      `- Evidence: [run artifacts](${runUrl}#artifacts) (when produced; 14-day retention)`, '',
      `Reproduce the matching group from the [workflow at the tested commit](${context.serverUrl}/${REPOSITORY}/blob/${context.sha}/.github/workflows/navigation-client-full.yml). The full workflow can also be dispatched manually on a selected branch.`,
      'Inspect the failed assertions and retained originals, fix the cause, and validate the affected tests. Keep existing pixel thresholds and references intact.',
      'A passing PR smoke check does not establish that this full-regression failure is fixed.',
    ].join('\n');
    const existing = issues.find(issue => !issue.pull_request && issue.body?.includes(marker));
    if (!existing) {
      const result = await github.rest.issues.create({ owner, repo,
        title: `[CI] ${job.name} failed`, labels: ['bug'], body });
      issues.push(result.data);
      core.info(`Created bug #${result.data.number} for ${job.name}`);
    } else {
      // Preserve human edits, labels and assignees. Retry of the same report is idempotent.
      const comments = existing.body?.includes(attempt) ? [] : await github.paginate(
        github.rest.issues.listComments, { owner, repo, issue_number: existing.number, per_page: 100 });
      if (!existing.body?.includes(attempt) && !comments.some(comment => comment.body?.includes(attempt))) {
        await github.rest.issues.createComment({ owner, repo, issue_number: existing.number, body });
        core.info(`Updated bug #${existing.number} for ${job.name}`);
      }
    }
  }
}

module.exports = { GROUPS, reportFailures };
