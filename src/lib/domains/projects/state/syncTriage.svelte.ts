/**
 * Background AI review of repositories a sync could not finish. One at a time (a
 * local model serves one request well), outliving the page. Nothing is committed
 * or pushed until the user presses Apply.
 */
import { invokeClient } from '$lib/utils/invokeClient';
import { getErrorMessage } from '$lib/utils/errorMessage';
import { logger } from '$lib/domains/shared/services/logger';
import { backgroundJobs } from '$lib/domains/shared/jobs/backgroundJobs.svelte';
import { syncRepoFacts } from '../api/syncAllApi';
import {
  TRIAGE_SYSTEM,
  commitEverything,
  emptyPlan,
  hasWork,
  needsReview,
  parseTriage,
  triagePrompt,
  type TriageQuestion,
} from '../utils/syncTriage';
import type { SyncRepoFacts } from '../generated/SyncRepoFacts';
import type { SyncResolution } from '../generated/SyncResolution';
import { syncAll } from './syncAll.svelte';

export type TriageStatus =
  'queued' | 'reviewing' | 'ready' | 'input' | 'applying' | 'done' | 'error';

export interface Triage {
  status: TriageStatus;
  summary: string;
  /** Commit messages and the push box are edited in place before applying. */
  plan: SyncResolution;
  questions: TriageQuestion[];
  /** "question -> answer", sent back with the next review. */
  answered: string[];
  facts: SyncRepoFacts | null;
  /** Outcome of the last apply, or why something failed. */
  message: string;
}

const AUTO_KEY = 'portal.syncAll.autoReview';
const JOB_ID = 'sync-review';
const log = logger.createScoped('SyncTriage');

function readAuto(): boolean {
  try {
    return localStorage.getItem(AUTO_KEY) !== 'off';
  } catch {
    return true;
  }
}

const blank = (): Triage => ({
  status: 'queued',
  summary: '',
  plan: emptyPlan(),
  questions: [],
  answered: [],
  facts: null,
  message: '',
});

class SyncTriageState {
  items = $state<Record<number, Triage>>({});
  auto = $state(readAuto());
  applyingAll = $state(false);
  private queue: number[] = [];
  private working = false;

  /** Ids with a plan that only waits for Apply. */
  ready = $derived(
    Object.entries(this.items)
      .filter(([, t]) => t.status === 'ready' && hasWork(t.plan))
      .map(([id]) => Number(id))
  );

  setAuto(on: boolean) {
    this.auto = on;
    try {
      localStorage.setItem(AUTO_KEY, on ? 'on' : 'off');
    } catch {
      // Stays for this session only.
    }
  }

  /** A full sync, then (by default) a review of whatever is left over. */
  async run(snapshotPassword: string | null) {
    if (syncAll.running) return;
    this.reset();
    if ((await syncAll.start(snapshotPassword)) && this.auto) this.reviewAll();
  }

  reset() {
    this.queue = [];
    this.items = {};
    backgroundJobs.remove(JOB_ID);
  }

  stop() {
    for (const id of this.queue) delete this.items[id];
    this.queue = [];
    backgroundJobs.remove(JOB_ID);
  }

  reviewAll() {
    for (const p of syncAll.report?.projects ?? []) {
      if (needsReview(p) && !this.items[p.id]) this.review(p.id);
    }
  }

  review(id: number) {
    const item = (this.items[id] ??= blank());
    if (item.status === 'reviewing' || item.status === 'applying') return;
    item.status = 'queued';
    item.message = '';
    if (!this.queue.includes(id)) this.queue.push(id);
    void this.drain();
  }

  /** Send the picked or typed answers back for a concrete plan. */
  answer(id: number) {
    const item = this.items[id];
    if (!item || item.questions.some((q) => !q.answer.trim())) return;
    item.answered.push(...item.questions.map((q) => `${q.question} -> ${q.answer.trim()}`));
    item.questions = [];
    this.review(id);
  }

  dismiss(id: number) {
    this.queue = this.queue.filter((x) => x !== id);
    delete this.items[id];
  }

  /** Skip the model: one commit of everything safe, still behind Apply. */
  async planEverything(id: number) {
    const item = (this.items[id] ??= blank());
    let facts: SyncRepoFacts;
    try {
      // Fresh: the fingerprint has to match the tree as it is now.
      facts = item.facts = await syncRepoFacts(id);
    } catch (err) {
      item.status = 'error';
      item.message = getErrorMessage(err);
      return;
    }
    item.questions = [];
    item.summary = 'Everything changed, except files that look secret, in one commit.';
    item.plan = {
      ...emptyPlan(),
      commits: commitEverything(facts),
      push: facts.pushOk,
      fingerprint: facts.fingerprint,
    };
    item.status = 'ready';
  }

  private async drain() {
    if (this.working) return;
    this.working = true;
    try {
      let id: number | undefined;
      while ((id = this.queue.shift()) !== undefined) {
        backgroundJobs.upsert({
          id: JOB_ID,
          kind: 'sync',
          title: 'AI reviewing changes',
          detail: `${syncAll.project(id)?.name ?? id}, ${this.queue.length} waiting`,
          percent: null,
          href: '/projects/sync',
          cancel: () => this.stop(),
        });
        await this.reviewOne(id);
      }
    } finally {
      this.working = false;
      backgroundJobs.remove(JOB_ID);
    }
  }

  private async reviewOne(id: number) {
    const item = this.items[id];
    if (!item) return;
    item.status = 'reviewing';
    try {
      const facts = await syncRepoFacts(id);
      item.facts = facts;
      if (!facts.filesTotal) {
        item.status = 'done';
        item.message = 'Nothing left to commit.';
        return;
      }
      const res = await invokeClient.post<{ content: string }>('generate_ai_text_with_system', {
        systemMessage: TRIAGE_SYSTEM,
        userMessage: triagePrompt(syncAll.project(id)?.name ?? String(id), facts, item.answered),
        // Reasoning models spend most of the budget thinking (see AiTidy).
        options: { temperature: 0, max_tokens: 4000, timeout_ms: 180000 },
      });
      // Dismissed or reset while the model was thinking.
      if (this.items[id] !== item) return;
      const reply = parseTriage(res.content, facts);
      if (!reply) throw new Error('The AI reply had no usable plan.');
      item.summary = reply.summary;
      item.questions = reply.questions;
      item.plan = {
        ...emptyPlan(),
        commits: reply.commits,
        ignore: reply.ignore,
        push: facts.pushOk && reply.commits.length > 0,
        fingerprint: facts.fingerprint,
      };
      item.status = reply.questions.length ? 'input' : 'ready';
    } catch (err) {
      log.warn('review failed', err);
      if (this.items[id] !== item) return;
      item.status = 'error';
      item.message = getErrorMessage(err, 'The review failed');
      // No provider set up: every other item would fail the same way.
      if (item.message.includes('Configuration incomplete')) this.stop();
    }
  }

  async apply(id: number) {
    const item = this.items[id];
    if (item?.status !== 'ready') return;
    this.queue = this.queue.filter((x) => x !== id);
    item.status = 'applying';
    try {
      const result = await syncAll.runPlan(id, $state.snapshot(item.plan));
      item.message = [...result.done, result.error].filter(Boolean).join(' · ');
      item.status = result.error ? 'error' : 'done';
      item.plan = emptyPlan();
    } catch (err) {
      item.message = getErrorMessage(err, 'Could not apply');
      item.status = 'error';
    }
  }

  /** One after another; the backend refuses overlapping writes. */
  async applyReady() {
    if (this.applyingAll) return;
    this.applyingAll = true;
    try {
      for (const id of [...this.ready]) await this.apply(id);
    } finally {
      this.applyingAll = false;
    }
  }
}

export const syncTriage = new SyncTriageState();
