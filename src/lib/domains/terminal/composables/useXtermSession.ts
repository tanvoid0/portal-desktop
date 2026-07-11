import { Terminal as XTerminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import type { ITheme } from "@xterm/xterm";
import { logger } from "$lib/domains/shared";
import { isTauriEnvironment } from "$lib/utils/tauri";
import type { TerminalConfig, TerminalOutput, TerminalProcess } from "../types";
import { sessionStore } from "../stores/sessionStore";
import { injectOsc8Links } from "../utils/osc8";
import { resolveXtermTheme } from "../theme";
import {
  createTerminalProcess,
  getProcessExitCode,
  killTerminalProcess,
  resizeTerminalProcess,
  sendProcessInput,
  subscribeProcessOutput,
} from "./useTerminalProcess";
import { commandBlockStore } from "../stores/commandBlockStore";

export type XtermSessionMode = "interactive" | "oneshot" | "display";

export interface XtermSessionOptions {
  tabId: string;
  settings: TerminalConfig;
  mode?: XtermSessionMode;
  oneshotCommand?: string;
  /** Static buffer rendered in display mode (no PTY). */
  displayContent?: string;
  /** Per-instance xterm theme override merged over the resolved app theme. */
  themeOverride?: Partial<ITheme>;
  /** Extra environment overrides forwarded to the PTY. */
  env?: Record<string, string>;
  /**
   * Kill the backend process on destroy. Defaults to true for interactive
   * sessions and false for oneshot (which has already exited).
   */
  killOnDestroy?: boolean;
  onOutputChunk?: (content: string) => void;
  onComplete?: (exitCode: number | null, outputBuffer: string) => void;
}

export class XtermSession {
  terminal: XTerminal | null = null;
  fitAddon: FitAddon | null = null;
  currentProcess: TerminalProcess | null = null;
  isConnected = false;
  outputBuffer = "";

  private osc8Tail = "";
  private sessionSaveTimer: ReturnType<typeof setTimeout> | null = null;
  private resizeHandler: (() => void) | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private outputUnsubscribe: (() => void) | null = null;
  private readonly log = logger.createScoped("XtermSession");
  private readonly isTauri = isTauriEnvironment();
  private mounted = false;
  private disposed = false;

  constructor(private readonly opts: XtermSessionOptions) {}

  async mount(element: HTMLElement): Promise<void> {
    if (this.mounted) {
      await this.destroy();
    }
    this.mounted = true;
    this.disposed = false;

    const isDisplay = this.opts.mode === "display";
    const t = new XTerminal({
      theme: resolveXtermTheme(this.opts.settings, this.opts.themeOverride),
      fontSize: this.opts.settings.fontSize,
      fontFamily: this.opts.settings.fontFamily,
      cursorStyle: this.opts.settings.cursorStyle,
      scrollback: this.opts.settings.scrollbackLines,
      allowTransparency: false,
      disableStdin: isDisplay,
      cursorBlink: !isDisplay,
    });

    const f = new FitAddon();
    t.loadAddon(f);
    t.open(element);
    f.fit();
    t.focus();

    this.terminal = t;
    this.fitAddon = f;

    t.onData((data) => {
      if (!this.isConnected || !this.currentProcess) return;
      sendProcessInput(this.currentProcess.id, data, this.opts.tabId).catch(
        (err) => this.log.warn("Failed to send input", { err }),
      );
    });

    t.onKey((e) => {
      if (e.domEvent.ctrlKey && e.key === "c" && this.currentProcess) {
        e.domEvent.preventDefault();
        sendProcessInput(this.currentProcess.id, "\x03", this.opts.tabId).catch(
          (err) => this.log.warn("Failed to interrupt process", { err }),
        );
      } else if (e.domEvent.ctrlKey && e.key === "l") {
        e.domEvent.preventDefault();
        t.clear();
      }
    });

    if (this.opts.mode === "display") {
      await this.startDisplay();
    } else if (this.opts.mode === "oneshot" && this.opts.oneshotCommand) {
      await this.startOneshot(this.opts.oneshotCommand);
    } else {
      await this.startInteractive();
    }

    this.resizeHandler = () => this.fit();
    window.addEventListener("resize", this.resizeHandler);

    this.resizeObserver = new ResizeObserver(() => {
      this.fit();
    });
    this.resizeObserver.observe(element);

    // Layout may not be ready on first paint (resizable panes, tabs). A single
    // rAF handles the common case; the ResizeObserver above covers any later
    // layout settling without the old fire-and-forget setTimeout(100/300) calls
    // that could run after the session was already disposed.
    requestAnimationFrame(() => this.fit());
  }

  private restoreSession(
    session: Awaited<ReturnType<typeof sessionStore.loadSession>>,
  ) {
    if (!this.terminal || !session) return;
    this.terminal.clear();
    if (session.scrollback_buffer?.length) {
      for (const line of session.scrollback_buffer) {
        this.terminal.write(line + "\r\n");
      }
    }
    const [x, y] = session.cursor_position;
    this.terminal.write("\x1b[" + y + ";" + x + "H");
  }

  private async startDisplay(): Promise<void> {
    if (!this.terminal) return;
    const content = this.opts.displayContent ?? "";
    if (content) {
      this.outputBuffer = content;
      this.terminal.write(content.replace(/\n/g, "\r\n"));
    }
  }

  private async startInteractive(): Promise<void> {
    if (!this.isTauri || !this.terminal) return;

    this.currentProcess = await createTerminalProcess({
      tabId: this.opts.tabId,
      shell: this.opts.settings.defaultShell,
      workingDirectory: this.opts.settings.workingDirectory,
      cols: this.terminal.cols,
      rows: this.terminal.rows,
      env: this.opts.env,
    });

    if (!this.currentProcess) return;

    this.outputUnsubscribe = await subscribeProcessOutput(
      this.currentProcess.id,
      (output) => this.handleOutput(output),
    );

    this.isConnected = true;
    commandBlockStore.registerProcessTab(
      this.currentProcess.id,
      this.opts.tabId,
    );

    this.fit();
  }

  private async startOneshot(command: string): Promise<void> {
    if (!this.isTauri || !this.terminal) return;

    this.terminal.write(`\x1b[90m$ ${command}\x1b[0m\r\n\r\n`);

    // The backend runs the command directly in the PTY (via the shell's
    // `-c`/`/C`) and exits with its real code — no fragile stdin `exec`
    // injection, and it works on PowerShell/cmd, not just POSIX shells.
    this.currentProcess = await createTerminalProcess({
      tabId: this.opts.tabId,
      shell: this.opts.settings.defaultShell,
      workingDirectory: this.opts.settings.workingDirectory,
      cols: this.terminal.cols,
      rows: this.terminal.rows,
      command,
      env: this.opts.env,
    });

    if (!this.currentProcess) return;

    this.isConnected = true;
    this.outputUnsubscribe = await subscribeProcessOutput(
      this.currentProcess.id,
      (output) => this.handleOutput(output),
    );
  }

  handleOutput(output: TerminalOutput): void {
    if (!this.terminal) return;

    this.outputBuffer += output.content;
    this.opts.onOutputChunk?.(output.content);

    // Live-stream output into the running command block (Warp-style blocks
    // view). The block itself is created by shell-integration events.
    if (
      this.opts.mode !== "oneshot" &&
      output.output_type === "stdout" &&
      this.currentProcess
    ) {
      commandBlockStore.appendToRunningBlock(
        this.currentProcess.id,
        output.content,
      );
    }

    const injected = injectOsc8Links(output.content, this.osc8Tail, {
      maxTailChars: 256,
    });
    this.osc8Tail = injected.newTail;
    if (injected.transformed) {
      this.terminal.write(injected.transformed);
    }

    if (output.output_type === "exit") {
      const flushed = injectOsc8Links("", this.osc8Tail, {
        maxTailChars: 256,
        flush: true,
      });
      this.osc8Tail = "";
      if (flushed.transformed) this.terminal.write(flushed.transformed);

      // Report the real exit code from the backend rather than always null.
      const processId = this.currentProcess?.id;
      if (processId) {
        getProcessExitCode(processId)
          .then((code) => this.opts.onComplete?.(code, this.outputBuffer))
          .catch(() => this.opts.onComplete?.(null, this.outputBuffer));
      } else {
        this.opts.onComplete?.(null, this.outputBuffer);
      }
    }

    this.scheduleSessionSave();
  }

  private scheduleSessionSave() {
    if (this.sessionSaveTimer) clearTimeout(this.sessionSaveTimer);
    this.sessionSaveTimer = setTimeout(async () => {
      try {
        await this.saveSession();
      } catch (e) {
        this.log.warn("Failed to save terminal session", { e });
      }
    }, 2000);
  }

  async saveSession(): Promise<void> {
    if (!this.terminal || !this.isTauri) return;

    const buffer = this.terminal.buffer.active;
    const scrollbackLines: string[] = [];

    for (let i = 0; i < Math.min(buffer.length, 10000); i++) {
      const line = buffer.getLine(i);
      if (!line) continue;
      const text = line.translateToString(true);
      if (text.trim().length > 0) scrollbackLines.push(text);
    }

    await sessionStore.saveSession({
      tab_id: this.opts.tabId,
      working_directory: this.opts.settings.workingDirectory,
      environment: this.currentProcess?.environment || {},
      scrollback_buffer: scrollbackLines,
      cursor_position: [
        this.terminal.buffer.active.cursorX,
        this.terminal.buffer.active.cursorY,
      ],
      terminal_size: [this.terminal.cols, this.terminal.rows],
      last_activity: new Date().toISOString(),
      process_id: this.currentProcess?.id,
    });
  }

  fit(): void {
    // Guard against fits scheduled (rAF / ResizeObserver) after teardown, and
    // against fitting before the element is actually in the DOM.
    if (this.disposed || !this.fitAddon || !this.terminal) return;
    if (!this.terminal.element?.isConnected) return;
    try {
      this.fitAddon.fit();
      if (this.currentProcess) {
        resizeTerminalProcess(
          this.currentProcess.id,
          this.terminal.cols,
          this.terminal.rows,
        ).catch((err) => this.log.warn("Resize failed", { err }));
      }
    } catch {
      // ignore
    }
  }

  clear(): void {
    this.terminal?.clear();
    this.outputBuffer = "";
  }

  write(data: string): void {
    this.terminal?.write(data);
  }

  async sendCommand(command: string): Promise<void> {
    if (!this.currentProcess) return;
    const trimmed = command.replace(/\r?\n$/, "");
    const lineEnding = navigator.userAgent.includes("Windows") ? "\r\n" : "\n";
    // Bracketed paste for multi-line commands so the shell takes the block
    // as one unit instead of stalling at continuation prompts.
    const payload = trimmed.includes("\n")
      ? `\x1b[200~${trimmed}\x1b[201~\r`
      : `${trimmed}${lineEnding}`;
    await sendProcessInput(this.currentProcess.id, payload, this.opts.tabId);
  }

  async destroy(): Promise<void> {
    this.disposed = true;
    if (this.resizeHandler) {
      window.removeEventListener("resize", this.resizeHandler);
    }
    this.resizeObserver?.disconnect();
    if (this.sessionSaveTimer) clearTimeout(this.sessionSaveTimer);

    try {
      if (this.terminal && this.isTauri) await this.saveSession();
    } catch (e) {
      this.log.warn("Failed to save session on destroy", { e });
    }

    this.outputUnsubscribe?.();

    const shouldKill =
      this.opts.killOnDestroy ?? this.opts.mode === "interactive";
    if (this.currentProcess && shouldKill) {
      await killTerminalProcess(this.currentProcess.id).catch(() => {});
    }

    this.terminal?.dispose();
    this.terminal = null;
    this.fitAddon = null;
    this.mounted = false;
  }

  async kill(): Promise<void> {
    if (this.currentProcess) {
      await killTerminalProcess(this.currentProcess.id);
      this.isConnected = false;
    }
  }
}
