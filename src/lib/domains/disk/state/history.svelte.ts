// The last minute of readings for the System page traces. Module-level so it survives navigation.

export interface Sample {
  at: number;
  /** load: CPU %. net: download bytes/s. */
  a: number;
  /** load: memory %. net: upload bytes/s. */
  b: number;
}

export const WINDOW_MS = 60_000;

/** A longer silence than this starts the trace over rather than drawing a line across it. */
const GAP_MS = 10_000;

export const history = $state<{ load: Sample[]; net: Sample[] }>({ load: [], net: [] });

export function record(series: keyof typeof history, a: number, b: number) {
  const at = Date.now();
  const last = history[series].at(-1);
  const kept =
    last && at - last.at <= GAP_MS ? history[series].filter((s) => at - s.at <= WINDOW_MS) : [];
  history[series] = [...kept, { at, a, b }];
}
