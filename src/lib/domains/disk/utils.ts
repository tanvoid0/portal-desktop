// Formatting + proposal-tree helpers for the disk-cleanup UI. Ported from
// portal_disk_utility's App.tsx, with the recursive tree flattened to a linear
// row list (depth-tagged) so Svelte can render it without markup recursion.

import type { Proposal, ItemVerdict } from "./types";

export function fmtBytes(n: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

export function fmtDate(unixSecs: number): string {
  return new Date(unixSecs * 1000).toLocaleString();
}

export function fmtDuration(ms: number): string {
  if (ms < 1000) return "<1s";
  const s = Math.round(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return rem ? `${m}m ${rem}s` : `${m}m`;
}

export function fmtAgo(unixSecs: number): string {
  const s = Math.max(0, Math.floor(Date.now() / 1000) - unixSecs);
  if (s < 60) return "just now";
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

export interface TreeNode {
  name: string;
  fullPath: string;
  proposal?: Proposal;
  children: TreeNode[];
  sizeBytes: number;
  count: number;
}

function splitPath(p: string): string[] {
  return p.split(/[\\/]/).filter((s) => s.length > 0);
}

export function buildTree(proposals: Proposal[], root: string): TreeNode[] {
  const rootSegs = splitPath(root);
  const virtual: TreeNode = { name: root, fullPath: root, children: [], sizeBytes: 0, count: 0 };

  for (const p of proposals) {
    let segs = splitPath(p.path);
    if (segs.slice(0, rootSegs.length).join(" ") === rootSegs.join(" ")) {
      segs = segs.slice(rootSegs.length);
    }
    if (segs.length === 0) segs = [p.path];

    let node = virtual;
    segs.forEach((seg, i) => {
      const last = i === segs.length - 1;
      let child = node.children.find((c) => c.name === seg && (last ? !!c.proposal : !c.proposal));
      if (!child) {
        child = {
          name: seg,
          fullPath: node === virtual ? `${root}\\${seg}` : `${node.fullPath}\\${seg}`,
          children: [],
          sizeBytes: 0,
          count: 0,
        };
        node.children.push(child);
      }
      if (last) child.proposal = p;
      node = child;
    });
  }

  function collapse(n: TreeNode) {
    while (n.children.length === 1 && !n.children[0].proposal && !n.proposal) {
      const only = n.children[0];
      n.name = `${n.name}\\${only.name}`;
      n.fullPath = only.fullPath;
      n.children = only.children;
    }
    n.children.forEach(collapse);
  }

  function rollup(n: TreeNode): { bytes: number; count: number } {
    if (n.proposal && n.children.length === 0) {
      n.sizeBytes = n.proposal.sizeBytes;
      n.count = 1;
      return { bytes: n.sizeBytes, count: 1 };
    }
    let bytes = n.proposal ? n.proposal.sizeBytes : 0;
    let count = n.proposal ? 1 : 0;
    for (const c of n.children) {
      const r = rollup(c);
      bytes += r.bytes;
      count += r.count;
    }
    n.sizeBytes = bytes;
    n.count = count;
    return { bytes, count };
  }

  virtual.children.forEach(collapse);
  virtual.children.forEach(rollup);
  const sortRec = (n: TreeNode) => {
    n.children.sort((a, b) => {
      const af = a.children.length > 0 ? 0 : 1;
      const bf = b.children.length > 0 ? 0 : 1;
      return af - bf || b.sizeBytes - a.sizeBytes;
    });
    n.children.forEach(sortRec);
  };
  sortRec(virtual);
  return virtual.children;
}

export function leafIds(n: TreeNode): string[] {
  const out: string[] = [];
  if (n.proposal) out.push(n.proposal.id);
  for (const c of n.children) out.push(...leafIds(c));
  return out;
}

export function folderPaths(nodes: TreeNode[]): string[] {
  const out: string[] = [];
  for (const n of nodes) {
    if (n.children.length > 0) {
      out.push(n.fullPath);
      out.push(...folderPaths(n.children));
    }
  }
  return out;
}

/** One visible tree row: the node, its depth, and derived flags. */
export interface FlatRow {
  node: TreeNode;
  depth: number;
  isFolder: boolean;
}

/** Depth-first walk yielding only rows currently visible under `expanded`. */
export function flattenTree(nodes: TreeNode[], expanded: Set<string>): FlatRow[] {
  const out: FlatRow[] = [];
  const walk = (n: TreeNode, depth: number) => {
    const isFolder = n.children.length > 0;
    out.push({ node: n, depth, isFolder });
    if (isFolder && expanded.has(n.fullPath)) {
      for (const c of n.children) walk(c, depth + 1);
    }
  };
  for (const n of nodes) walk(n, 0);
  return out;
}

export const RISK_BADGE: Record<string, string> = {
  Safe: "bg-status-success-bg text-status-success border-status-success/30",
  Review: "bg-status-warning-bg text-status-warning border-status-warning/30",
  Danger: "bg-status-error-bg text-status-error border-status-error/30",
};

export const VERDICT_BADGE: Record<string, string> = {
  safe: "bg-status-success-bg text-status-success border-status-success/30",
  review: "bg-status-warning-bg text-status-warning border-status-warning/30",
  dangerous: "bg-status-error-bg text-status-error border-status-error/30",
};

export const KIND_BADGE: Record<string, string> = {
  node: "bg-status-success-bg text-status-success border-status-success/30",
  rust: "bg-status-warning-bg text-status-warning border-status-warning/30",
  maven: "bg-status-error-bg text-status-error border-status-error/30",
  gradle: "bg-status-info-bg text-status-info border-status-info/30",
  python: "bg-status-info-bg text-status-info border-status-info/30",
  dotnet: "bg-secondary text-secondary-foreground border-border",
  go: "bg-status-info-bg text-status-info border-status-info/30",
  php: "bg-secondary text-secondary-foreground border-border",
  "stopped-container": "bg-status-warning-bg text-status-warning border-status-warning/30",
  "created-container": "bg-status-warning-bg text-status-warning border-status-warning/30",
  "dead-container": "bg-status-error-bg text-status-error border-status-error/30",
  "dangling-image": "bg-status-success-bg text-status-success border-status-success/30",
  "unused-image": "bg-status-info-bg text-status-info border-status-info/30",
  "dangling-volume": "bg-status-warning-bg text-status-warning border-status-warning/30",
  "dangling-network": "bg-secondary text-secondary-foreground border-border",
  "build-cache": "bg-status-success-bg text-status-success border-status-success/30",
  projects: "bg-status-success-bg text-status-success border-status-success/30",
  docker: "bg-status-info-bg text-status-info border-status-info/30",
  podman: "bg-status-info-bg text-status-info border-status-info/30",
};

/** Summary stat cards (3-up) — extra columns on ultrawide so cards don't stretch. */
export const DISK_STAT_GRID =
  "grid gap-3 grid-cols-2 md:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-6";

/** Risk / scan stat cards (4-up) — one row across standard and ultrawide widths. */
export const DISK_STAT_GRID_FOUR =
  "grid gap-3 grid-cols-2 md:grid-cols-4 xl:grid-cols-4 3xl:grid-cols-4";

/** Drive cards — scale column count as the main area widens. */
export const DISK_DRIVE_GRID =
  "grid gap-3 grid-cols-1 md:grid-cols-2 xl:grid-cols-3 3xl:grid-cols-4";

/** Side-by-side dashboard panels (chart + list). */
export const DISK_PANEL_GRID = "grid gap-3 grid-cols-1 lg:grid-cols-2 3xl:grid-cols-2";

export function verdictMap(verdicts: ItemVerdict[]): Map<string, ItemVerdict> {
  const m = new Map<string, ItemVerdict>();
  for (const v of verdicts) m.set(v.path, v);
  return m;
}
