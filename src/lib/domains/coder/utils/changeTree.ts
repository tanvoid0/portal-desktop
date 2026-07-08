import type { GitFileChange } from "../types.js";

export interface ChangeTreeNode {
  name: string;
  path: string;
  isFile: boolean;
  change?: GitFileChange;
  children: ChangeTreeNode[];
  additions: number;
  deletions: number;
}

function splitPath(p: string): string[] {
  return p.split(/[/\\]/).filter((s) => s.length > 0);
}

export function buildChangeTree(changes: GitFileChange[]): ChangeTreeNode[] {
  const root: ChangeTreeNode = {
    name: "",
    path: "",
    isFile: false,
    children: [],
    additions: 0,
    deletions: 0,
  };

  for (const change of changes) {
    const segs = splitPath(change.path);
    let node = root;
    for (let i = 0; i < segs.length; i++) {
      const seg = segs[i]!;
      const isFile = i === segs.length - 1;
      const path = segs.slice(0, i + 1).join("/");
      let child = node.children.find((c) => c.name === seg && c.isFile === isFile);
      if (!child) {
        child = {
          name: seg,
          path,
          isFile,
          children: [],
          additions: 0,
          deletions: 0,
        };
        node.children.push(child);
      }
      if (isFile) {
        child.change = change;
        child.additions = change.additions;
        child.deletions = change.deletions;
      }
      node = child;
    }
  }

  function aggregate(n: ChangeTreeNode): void {
    if (n.isFile) return;
    for (const c of n.children) {
      aggregate(c);
      n.additions += c.additions;
      n.deletions += c.deletions;
    }
    n.children.sort((a, b) => {
      if (a.isFile !== b.isFile) return a.isFile ? 1 : -1;
      return a.name.localeCompare(b.name);
    });
  }

  for (const c of root.children) aggregate(c);
  root.children.sort((a, b) => {
    if (a.isFile !== b.isFile) return a.isFile ? 1 : -1;
    return a.name.localeCompare(b.name);
  });

  return root.children;
}

export interface FlatChangeRow {
  node: ChangeTreeNode;
  depth: number;
}

export function flattenChangeTree(
  nodes: ChangeTreeNode[],
  expanded: Set<string>,
  depth = 0,
): FlatChangeRow[] {
  const rows: FlatChangeRow[] = [];
  for (const node of nodes) {
    rows.push({ node, depth });
    if (!node.isFile && expanded.has(node.path)) {
      rows.push(...flattenChangeTree(node.children, expanded, depth + 1));
    }
  }
  return rows;
}
