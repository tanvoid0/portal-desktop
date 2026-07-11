/**
 * Workspace sidebar + main panel state for the coder session view.
 */

export type WorkspacePanel =
  | "chat"
  | "terminal"
  | "files"
  | "file"
  | "browser"
  | "changes"
  | "git-changes";

export interface WorkspaceTab {
  id: string;
  panel: WorkspacePanel;
  label: string;
  /** Terminal tab id when panel is terminal */
  terminalId?: string;
  /** Workspace-relative path when panel is file */
  filePath?: string;
}

class CoderWorkspaceStore {
  sidebarOpen = $state(true);
  expanded = $state<Record<string, boolean>>({
    terminal: true,
    files: false,
    browser: false,
    changes: false,
    "git-changes": false,
  });
  activePanel = $state<WorkspacePanel>("chat");
  openTabs = $state<WorkspaceTab[]>([]);
  activeTabId = $state<string | null>(null);
  browserUrl = $state("http://localhost:1420");

  private bumpTabs() {
    this.openTabs = [...this.openTabs];
  }

  expandSection(section: string) {
    if (!this.expanded[section]) {
      this.expanded[section] = true;
      this.expanded = { ...this.expanded };
    }
  }

  toggleSection(section: string) {
    this.expanded[section] = !this.expanded[section];
    this.expanded = { ...this.expanded };
  }

  isExpanded(section: string): boolean {
    return !!this.expanded[section];
  }

  private upsertTab(tab: WorkspaceTab) {
    const i = this.openTabs.findIndex((t) => t.id === tab.id);
    if (i >= 0) {
      const existing = this.openTabs[i];
      const unchanged =
        existing.label === tab.label &&
        existing.panel === tab.panel &&
        existing.terminalId === tab.terminalId;
      const alreadyActive =
        this.activeTabId === tab.id && this.activePanel === tab.panel;
      if (unchanged && alreadyActive) return;
      if (!unchanged) {
        this.openTabs = this.openTabs.map((t, idx) => (idx === i ? tab : t));
      }
    } else {
      this.openTabs = [...this.openTabs, tab];
    }
    if (this.activeTabId !== tab.id) this.activeTabId = tab.id;
    if (this.activePanel !== tab.panel) this.activePanel = tab.panel;
  }

  openChat() {
    this.activePanel = "chat";
    this.activeTabId = null;
  }

  openTerminal(threadId: string, terminalId: string, label: string) {
    const id = `terminal:${terminalId}`;
    this.upsertTab({
      id,
      panel: "terminal",
      label,
      terminalId,
    });
    void threadId;
  }

  openChanges(label = "Agent changes") {
    this.upsertTab({ id: "changes", panel: "changes", label });
  }

  openGitChanges(label = "Git changes") {
    this.upsertTab({ id: "git-changes", panel: "git-changes", label });
  }

  openFiles(label = "Files") {
    this.upsertTab({ id: "files", panel: "files", label });
  }

  openFile(filePath: string, label?: string) {
    this.upsertTab({
      id: `file:${filePath}`,
      panel: "file",
      label: label ?? filePath.split(/[/\\]/).pop() ?? filePath,
      filePath,
    });
  }

  openBrowser(url?: string, label = "Browser") {
    if (url) this.browserUrl = url;
    this.upsertTab({ id: "browser", panel: "browser", label });
  }

  selectTab(tabId: string) {
    const tab = this.openTabs.find((t) => t.id === tabId);
    if (!tab) return;
    this.activeTabId = tabId;
    this.activePanel = tab.panel;
  }

  closeTab(tabId: string) {
    this.openTabs = this.openTabs.filter((t) => t.id !== tabId);
    if (this.activeTabId === tabId) {
      const next = this.openTabs[this.openTabs.length - 1];
      if (next) {
        this.activeTabId = next.id;
        this.activePanel = next.panel;
      } else {
        this.openChat();
      }
    }
  }

  /** Active terminal id when terminal panel is showing. */
  activeTerminalId(): string | null {
    if (this.activePanel !== "terminal") return null;
    const tab = this.openTabs.find((t) => t.id === this.activeTabId);
    return tab?.terminalId ?? null;
  }
}

export const coderWorkspaceStore = new CoderWorkspaceStore();
