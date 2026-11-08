import { test } from 'node:test'
import assert from 'node:assert/strict'
import { announcement, orderedFeedback, type FeedbackMessage } from './ui/feedback.ts'

test('persistent critical state outranks rejection, work and confirmation; duplicate owner text is announced once', () => {
  const messages: FeedbackMessage[] = [
    { id: 'duplicate-status', text: 'Overweight · nothing moved.', priority: 'status' },
    { id: 'saved', text: 'Saved: deposited 333 g.', priority: 'confirmation' },
    { id: 'work', text: 'Review · nothing moved.', priority: 'pending' },
    { id: 'capacity', text: 'Overweight · nothing moved.', priority: 'error' },
    { id: 'health', text: 'Run ended · zero health.', priority: 'critical' },
    { id: 'copy', text: 'Run ended · zero health.', priority: 'critical' },
    { id: 'empty', text: '', priority: 'status' },
  ]
  assert.deepEqual(orderedFeedback(messages).map(m => m.id), ['health', 'capacity', 'work', 'saved'])
  assert.equal(announcement(messages), announcement(messages.map(m => ({ ...m }))))
  assert.equal(orderedFeedback(messages).filter(m => m.priority === 'critical').length, 1)
  assert.match(announcement(messages), /^Warning: Run ended/)
  assert.match(announcement(messages), /Error: Overweight/)
  assert.equal(announcement([]), '')
})
