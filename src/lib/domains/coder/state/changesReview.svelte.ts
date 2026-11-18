import { tick } from 'svelte';
import { coderService } from '../services/coderService.js';
import type { FileChange, Hunk } from '../types.js';

/**
 * Hunk-by-hunk review state for ChangesPanel. Construct during component init
 * (registers $effects); pass getters so props stay reactive.
 */
export class ChangesReviewState {
  editing = $state<string | null>(null);
  draft = $state('');
  focusChangeId = $state<string | null>(null);
  focusHunkIndex = $state<number | null>(null);
  reviewedHunks = $state(new Set<string>());

  constructor(
    private getChanges: () => FileChange[],
    private onRefresh: () => void | Promise<void>
  ) {
    $effect(() => {
      this.getChanges();
      this.syncFocus();
    });

    $effect(() => {
      if (this.focusChangeId && this.focusHunkIndex !== null) {
        this.scrollToFocus();
      }
    });
  }

  get changes() {
    return this.getChanges();
  }

  pendingChanges = $derived(this.changes.filter((c) => c.status === 'pending'));

  focusChange = $derived(
    this.focusChangeId ? (this.changes.find((c) => c.id === this.focusChangeId) ?? null) : null
  );

  focusPosition = $derived.by(() => {
    if (!this.focusChangeId || this.focusHunkIndex === null) return null;
    const changeIdx = this.pendingChanges.findIndex((c) => c.id === this.focusChangeId);
    if (changeIdx < 0) return null;
    const change = this.pendingChanges[changeIdx];
    const hunkIdx = change.hunks.findIndex((h) => h.index === this.focusHunkIndex);
    if (hunkIdx < 0) return null;
    return {
      changeIdx,
      hunkIdx,
      totalChanges: this.pendingChanges.length,
      totalHunks: change.hunks.length,
    };
  });

  private hunkKey(changeId: string, hunkIndex: number): string {
    return `${changeId}:${hunkIndex}`;
  }

  isReviewed = (changeId: string, hunkIndex: number): boolean =>
    this.reviewedHunks.has(this.hunkKey(changeId, hunkIndex));

  isFocused = (changeId: string, hunkIndex: number): boolean =>
    this.focusChangeId === changeId && this.focusHunkIndex === hunkIndex;

  focus(changeId: string | null, hunkIndex: number | null) {
    this.focusChangeId = changeId;
    this.focusHunkIndex = hunkIndex;
  }

  private findNextUnreviewed(
    afterChangeId?: string,
    afterHunkIndex?: number
  ): { changeId: string; hunkIndex: number } | null {
    const list = this.pendingChanges;
    if (list.length === 0) return null;

    let startChangeIdx = 0;
    if (afterChangeId !== undefined) {
      const idx = list.findIndex((c) => c.id === afterChangeId);
      if (idx >= 0) startChangeIdx = idx;
    }

    for (let ci = startChangeIdx; ci < list.length; ci++) {
      const change = list[ci];
      let startHunk = 0;
      if (ci === startChangeIdx && afterHunkIndex !== undefined) {
        const currentIdx = change.hunks.findIndex((h) => h.index === afterHunkIndex);
        startHunk = currentIdx >= 0 ? currentIdx + 1 : 0;
      }

      for (let hi = startHunk; hi < change.hunks.length; hi++) {
        const hunk = change.hunks[hi];
        if (!this.isReviewed(change.id, hunk.index)) {
          return { changeId: change.id, hunkIndex: hunk.index };
        }
      }
    }

    return null;
  }

  private syncFocus() {
    if (this.focusChangeId && this.focusHunkIndex !== null) {
      const change = this.changes.find((c) => c.id === this.focusChangeId);
      const hunkExists = change?.hunks.some((h) => h.index === this.focusHunkIndex);
      if (change?.status === 'pending' && hunkExists) return;
    }

    const first = this.findNextUnreviewed();
    this.focus(first?.changeId ?? null, first?.hunkIndex ?? null);
  }

  private async scrollToFocus() {
    await tick();
    if (!this.focusChangeId || this.focusHunkIndex === null) return;
    const el = document.querySelector(
      `[data-hunk-id="${this.focusChangeId}:${this.focusHunkIndex}"]`
    );
    el?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  private async refresh() {
    await this.onRefresh();
    this.syncFocus();
    await this.scrollToFocus();
  }

  private async finalizeChangeIfReviewed(change: FileChange) {
    if (change.hunks.some((h) => !this.isReviewed(change.id, h.index))) return;

    const allAccepted = change.hunks.every((h) => h.accepted);
    const allRejected = change.hunks.every((h) => !h.accepted);

    if (allAccepted && change.status !== 'accepted') {
      await coderService.acceptChange(change.id);
    } else if (allRejected && change.status !== 'rejected') {
      await coderService.rejectChange(change.id);
    }
  }

  private async advanceAfterReview(changeId: string, hunkIndex: number) {
    const change = this.changes.find((c) => c.id === changeId);
    if (change) {
      await this.finalizeChangeIfReviewed(change);
    }

    await this.refresh();

    const next = this.findNextUnreviewed(changeId, hunkIndex);
    this.focus(next?.changeId ?? null, next?.hunkIndex ?? null);
    await this.scrollToFocus();
  }

  private markReviewed(changeId: string, hunkIndex: number) {
    this.reviewedHunks = new Set(this.reviewedHunks).add(this.hunkKey(changeId, hunkIndex));
  }

  private markAllReviewed(change: FileChange) {
    const next = new Set(this.reviewedHunks);
    for (const h of change.hunks) {
      next.add(this.hunkKey(change.id, h.index));
    }
    this.reviewedHunks = next;
  }

  acceptHunk = async (c: FileChange, h: Hunk) => {
    if (!h.accepted) {
      await coderService.setHunk(c.id, h.index, true);
    }
    this.markReviewed(c.id, h.index);
    await this.advanceAfterReview(c.id, h.index);
  };

  rejectHunk = async (c: FileChange, h: Hunk) => {
    if (h.accepted) {
      await coderService.setHunk(c.id, h.index, false);
    }
    this.markReviewed(c.id, h.index);
    await this.advanceAfterReview(c.id, h.index);
  };

  acceptAllFiles = async () => {
    for (const c of [...this.pendingChanges]) {
      await coderService.acceptChange(c.id);
      this.markAllReviewed(c);
    }
    await this.refresh();
  };

  rejectAllFiles = async () => {
    for (const c of [...this.pendingChanges]) {
      await coderService.rejectChange(c.id);
      this.markAllReviewed(c);
    }
    await this.refresh();
  };

  acceptAll = async (c: FileChange) => {
    await coderService.acceptChange(c.id);
    this.markAllReviewed(c);
    await this.advanceAfterReview(c.id, c.hunks.at(-1)?.index ?? 0);
  };

  rejectAll = async (c: FileChange) => {
    await coderService.rejectChange(c.id);
    this.markAllReviewed(c);
    await this.advanceAfterReview(c.id, c.hunks.at(-1)?.index ?? 0);
  };

  startEdit = (c: FileChange) => {
    this.editing = c.id;
    this.draft = c.original_after;
  };

  saveEdit = async (c: FileChange) => {
    await coderService.modifyChange(c.id, this.draft);
    this.editing = null;
    this.reviewedHunks = new Set();
    await this.refresh();
  };

  goToPrevHunk = () => {
    if (!this.focusChangeId || this.focusHunkIndex === null || !this.focusPosition) return;
    const { changeIdx, hunkIdx } = this.focusPosition;
    if (hunkIdx > 0) {
      this.focusHunkIndex = this.pendingChanges[changeIdx].hunks[hunkIdx - 1].index;
    } else if (changeIdx > 0) {
      const prev = this.pendingChanges[changeIdx - 1];
      this.focus(prev.id, prev.hunks.at(-1)?.index ?? null);
    }
    this.scrollToFocus();
  };

  goToNextHunk = () => {
    if (!this.focusChangeId || this.focusHunkIndex === null || !this.focusPosition) return;
    const { changeIdx, hunkIdx, totalHunks } = this.focusPosition;
    if (hunkIdx < totalHunks - 1) {
      this.focusHunkIndex = this.pendingChanges[changeIdx].hunks[hunkIdx + 1].index;
    } else if (changeIdx < this.pendingChanges.length - 1) {
      const next = this.pendingChanges[changeIdx + 1];
      this.focus(next.id, next.hunks[0]?.index ?? null);
    }
    this.scrollToFocus();
  };
}
