import { describe, expect, it, vi } from 'vitest';
import { applyDelta, applyEvent, emptyView, fromSnapshot, type ThreadView } from './reducer';
import type { ThreadDeltaFrame, ThreadEventFrame, ThreadSnapshot } from '../generated';
import claudeRun from './fixtures/claude_run.json';
import portalTurn from './fixtures/portal_turn.json';

// Written by `fixtures_match_checked_in` in portal-core/.../threads/tests.rs.
interface Fixture {
  events: ThreadEventFrame[];
  snapshot: ThreadSnapshot;
}

function fold(events: ThreadEventFrame[], from: ThreadView = emptyView()): ThreadView {
  return events.reduce((v, f) => {
    const n = applyEvent(v, f);
    if (!n) throw new Error(`gap at seq ${f.seq}`);
    return n;
  }, from);
}

describe.each([
  ['claude run', claudeRun as unknown as Fixture],
  ['portal turn', portalTurn as unknown as Fixture],
])('%s fixture', (_name, { events, snapshot }) => {
  it('folding every event from empty equals the snapshot', () => {
    expect(fold(events)).toEqual(fromSnapshot(snapshot));
  });

  it('seqs are contiguous and end at the snapshot seq', () => {
    expect(events.map((e) => e.seq)).toEqual(events.map((_, i) => i + 1));
    expect(fold(events).seq).toBe(snapshot.seq);
  });

  it('a duplicate frame changes nothing', () => {
    const done = fold(events);
    expect(applyEvent(done, events[3])).toBe(done);
    expect(applyEvent(done, events[events.length - 1])).toBe(done);
  });

  it('an out-of-order frame is a gap, not applied', () => {
    expect(applyEvent(emptyView(), events[2])).toBeNull();
    const v = fold(events.slice(0, 2));
    expect(applyEvent(v, events[3])).toBeNull();
    // catching up in order recovers
    expect(fold(events.slice(2), v)).toEqual(fold(events));
  });
});

it('portal fixture covers stop_requested and sub_agent_updated', () => {
  const { events } = portalTurn as unknown as Fixture;
  const kinds = events.map((e) => e.event.type);
  expect(kinds).toContain('stop_requested');
  expect(kinds).toContain('sub_agent_updated');
  expect(fold(events.slice(0, kinds.indexOf('stop_requested') + 1)).stopping).toBe(true);
  expect(fold(events).subAgents).toHaveLength(1);
});

it('an unknown event type is a no-op that advances the cursor', () => {
  const frame = {
    thread_id: 't',
    provider: 'portal',
    seq: 1,
    at: '',
    actor: 'system',
    event: { type: 'from_the_future' },
  } as unknown as ThreadEventFrame;
  const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
  const v = applyEvent(emptyView(), frame);
  expect(v).toEqual({ ...emptyView(), seq: 1 });
  warn.mockRestore();
});

const base = (over: Partial<ThreadView> = {}): ThreadView => ({
  ...emptyView(),
  seq: 5,
  status: 'running',
  streamingText: 'hello',
  ...over,
});
const delta = (d: Partial<ThreadDeltaFrame>): ThreadDeltaFrame => ({
  thread_id: 't',
  seq: 5,
  offset: 0,
  text: '',
  thinking: '',
  ...d,
});

describe('applyDelta', () => {
  it('appends at offset == partial length', () => {
    expect(applyDelta(base(), delta({ offset: 5, text: ' you' }))?.streamingText).toBe('hello you');
  });

  it('drops a chunk the snapshot partial already holds', () => {
    const v = base();
    expect(applyDelta(v, delta({ offset: 2, text: 'llo' }))).toBe(v);
    expect(applyDelta(v, delta({ offset: 0, text: 'hello' }))).toBe(v);
  });

  it('keeps only the new tail of an overlapping chunk', () => {
    expect(applyDelta(base(), delta({ offset: 3, text: 'lo!!' }))?.streamingText).toBe('hello!!');
  });

  it('asks for a re-snapshot when the chunk starts past the partial', () => {
    expect(applyDelta(base(), delta({ offset: 6, text: 'x' }))).toBeNull();
  });

  it('drops a chunk behind the cursor, re-snapshots one ahead of it', () => {
    const v = base();
    expect(applyDelta(v, delta({ seq: 4, offset: 5, text: 'x' }))).toBe(v);
    expect(applyDelta(v, delta({ seq: 6, offset: 5, text: 'x' }))).toBeNull();
  });

  it('counts offsets in UTF-16 units', () => {
    const v = base({ streamingText: 'a😀' }); // length 3
    expect(applyDelta(v, delta({ offset: 3, text: 'b' }))?.streamingText).toBe('a😀b');
  });

  it('keeps thinking apart from text', () => {
    const n = applyDelta(base(), delta({ thinking: 'hmm' }))!;
    expect(n.streamingThinking).toBe('hmm');
    expect(n.streamingText).toBe('hello');
  });
});
