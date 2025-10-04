<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  // import { FitAddon } from 'xterm-addon-fit';
  import { commandHistoryStore, type CommandHistoryEntry } from '../stores/commandHistoryStore';
  import { terminalStore, terminalActions } from '../stores/terminalStore';
  import { TerminalService } from '../services/terminalService';
  import { cleanTerminalOutput } from '../utils/textUtils';

  let terminalContainer: HTMLDivElement;
  let terminal: Terminal;
  // let fitAddon: FitAddon;
  let isConnected = false;
  let currentProcess: any = null;
  let commandOutputs: Array<{
    id: string;
    command: string;
    output: string;
    exitCode?: number;
    timestamp: number;
    isActive: boolean;
  }> = [];

  // Subscribe to command history to get new commands
  let unsubscribeHistory = commandHistoryStore.subscribe((state) => {
    // When a new command is added, create a containerized output
    if (state.entries.length > 0) {
      const latestEntry = state.entries[0];
      const existingOutput = commandOutputs.find(output => output.id === latestEntry.id);
      
      if (!existingOutput) {
        // Add new command output container
        commandOutputs = [{
          id: latestEntry.id,
          command: latestEntry.command,
          output: latestEntry.output,
          exitCode: latestEntry.exitCode,
          timestamp: latestEntry.timestamp.getTime(),
          isActive: true
        }, ...commandOutputs];
      } else {
        // Update existing output
        commandOutputs = commandOutputs.map(output => 
          output.id === latestEntry.id 
            ? { ...output, output: latestEntry.output, exitCode: latestEntry.exitCode }
            : output
        );
      }
    }
  });

  onMount(async () => {
    await initializeTerminal();
  });

  onDestroy(() => {
    if (terminal) {
      terminal.dispose();
    }
    if (unsubscribeHistory) {
      unsubscribeHistory();
    }
  });

  async function initializeTerminal() {
    // Initialize xterm.js
    terminal = new Terminal({
      theme: {
        background: '#1a1a1a',
        foreground: '#ffffff',
        cursor: '#ffffff',
        // selection: '#ffffff40',
        black: '#000000',
        red: '#ff5555',
        green: '#50fa7b',
        yellow: '#f1fa8c',
        blue: '#bd93f9',
        magenta: '#ff79c6',
        cyan: '#8be9fd',
        white: '#f8f8f2',
        brightBlack: '#6272a4',
        brightRed: '#ff6e6e',
        brightGreen: '#69ff94',
        brightYellow: '#ffffa5',
        brightBlue: '#d6acff',
        brightMagenta: '#ff92df',
        brightCyan: '#a4ffff',
        brightWhite: '#ffffff'
      },
      fontSize: 14,
      fontFamily: 'Monaco, Consolas, "Courier New", monospace',
      cursorBlink: true,
      cursorStyle: 'block',
      scrollback: 1000,
      allowTransparency: false
    });

    // Add fit addon (commented out for now)
    // fitAddon = new FitAddon();
    // terminal.loadAddon(fitAddon);

    // Mount terminal
    terminal.open(terminalContainer);
    // fitAddon.fit();

    // Set up event listeners
    terminal.onData(onData);
    terminal.onResize(onResize);

    // Try to connect to a terminal process
    await connectToTerminal();
  }

  async function connectToTerminal() {
    try {
      const processes = await TerminalService.getAllProcesses();
      if (processes.length > 0) {
        currentProcess = processes[0];
        isConnected = true;
        
        // Set up output listener (using public method)
        // TerminalService.setupOutputListener(currentProcess.id, handleOutput);
        
        terminal.write('\x1b[32m✓ Connected to terminal process\x1b[0m\r\n');
        terminal.write(`\x1b[36mProcess ID: ${currentProcess.id}\x1b[0m\r\n`);
        terminal.write(`\x1b[36mShell: ${currentProcess.command}\x1b[0m\r\n`);
        terminal.write(`\x1b[36mPID: ${currentProcess.pid || 'N/A'}\x1b[0m\r\n`);
        terminal.write('\r\n');
      } else {
        setupSimulatedTerminal();
      }
    } catch (error) {
      console.error('Failed to connect to terminal:', error);
      setupSimulatedTerminal();
    }
  }

  function setupSimulatedTerminal() {
    isConnected = false;
    terminal.write('\x1b[33m⚠️ No terminal process available\x1b[0m\r\n');
    terminal.write('\x1b[36mAvailable commands:\x1b[0m\r\n');
    terminal.write('  \x1b[32mhelp\x1b[0m - Show this help message\r\n');
    terminal.write('  \x1b[32mstatus\x1b[0m - Show connection status\r\n');
    terminal.write('  \x1b[32mclear\x1b[0m - Clear the terminal\r\n');
    terminal.write('\r\n');
  }

  function onData(data: string) {
    if (isConnected && currentProcess) {
      TerminalService.sendInput(currentProcess.id, data).catch((error) => {
        console.error('Failed to send input:', error);
      });
    } else {
      handleSimulatedInput(data);
    }
  }

  function onResize() {
    if (isConnected && currentProcess) {
      // Simple resize handling without fit addon
      const cols = 80; // Default columns
      const rows = 24; // Default rows
      TerminalService.resizeTerminal(currentProcess.id, cols, rows).catch(console.error);
    }
  }

  function handleOutput(output: any) {
    // Don't write to terminal directly - we'll handle this through command containers
    // Just trigger a re-render of command outputs
  }

  function handleSimulatedInput(data: string) {
    if (data === '\r') {
      const line = terminal.buffer.active.getLine(terminal.buffer.active.cursorY)?.translateToString(true) || '';
      const command = line.replace(/^.*[>#\$]\s*/, '').trim();
      
      if (command) {
        processSimulatedCommand(command);
      }
    } else {
      terminal.write(data);
    }
  }

  function processSimulatedCommand(command: string) {
    terminal.write('\r\n');
    
    switch (command.toLowerCase()) {
      case 'help':
        terminal.write('\x1b[36mAvailable commands:\x1b[0m\r\n');
        terminal.write('  \x1b[32mhelp\x1b[0m - Show this help message\r\n');
        terminal.write('  \x1b[32mstatus\x1b[0m - Show connection status\r\n');
        terminal.write('  \x1b[32mclear\x1b[0m - Clear the terminal\r\n');
        break;
      case 'status':
        terminal.write(`\x1b[36mConnection Status: ${isConnected ? 'Connected' : 'Disconnected'}\x1b[0m\r\n`);
        if (currentProcess) {
          terminal.write(`\x1b[36mProcess ID: ${currentProcess.id}\x1b[0m\r\n`);
        }
        break;
      case 'clear':
        terminal.clear();
        break;
      default:
        terminal.write(`\x1b[31mCommand not found: ${command}\x1b[0m\r\n`);
    }
    
    terminal.write('\r\n');
  }

  function getOutputBackgroundColor(exitCode?: number): string {
    if (exitCode === undefined) return 'bg-gray-800'; // Running/unknown
    if (exitCode === 0) return 'bg-gray-800'; // Success - dark gray
    return 'bg-red-900'; // Error - dark red
  }

  function getOutputBorderColor(exitCode?: number): string {
    if (exitCode === undefined) return 'border-gray-600'; // Running/unknown
    if (exitCode === 0) return 'border-gray-600'; // Success
    return 'border-red-600'; // Error
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }
</script>

<div class="containerized-terminal h-full w-full flex flex-col">
  <!-- Terminal Input Area -->
  <div class="terminal-input-area flex-1 min-h-0">
    <div bind:this={terminalContainer} class="h-full w-full"></div>
  </div>

  <!-- Command Output Containers -->
  <div class="command-outputs flex-1 overflow-y-auto p-4 space-y-4">
    {#each commandOutputs as output (output.id)}
      <div class="command-output-container rounded-lg border-2 p-4 {getOutputBackgroundColor(output.exitCode)} {getOutputBorderColor(output.exitCode)}">
        <!-- Command Header -->
        <div class="command-header flex items-center justify-between mb-3">
          <div class="flex items-center space-x-3">
            <span class="text-sm font-mono text-gray-300">
              {formatTimestamp(output.timestamp)}
            </span>
            <span class="text-sm font-mono text-white bg-gray-700 px-2 py-1 rounded">
              {output.command}
            </span>
            {#if output.exitCode !== undefined}
              <span class="text-xs px-2 py-1 rounded {output.exitCode === 0 ? 'bg-green-600 text-white' : 'bg-red-600 text-white'}">
                {output.exitCode === 0 ? 'SUCCESS' : `ERROR (${output.exitCode})`}
              </span>
            {/if}
          </div>
        </div>

        <!-- Command Output -->
        <div class="command-output">
          <pre class="text-sm font-mono text-white whitespace-pre-wrap overflow-x-auto">{cleanTerminalOutput(output.output)}</pre>
        </div>
      </div>
    {:else}
      <div class="text-center text-gray-500 py-8">
        <div class="text-4xl mb-2">📝</div>
        <div class="text-sm">No commands executed yet</div>
        <div class="text-xs mt-1">Command outputs will appear here with colored backgrounds</div>
      </div>
    {/each}
  </div>
</div>

<style>
  .containerized-terminal {
    background: #1a1a1a;
  }

  .terminal-input-area {
    border-bottom: 1px solid #374151;
  }

  .command-outputs {
    background: #111111;
  }

  .command-output-container {
    transition: all 0.2s ease;
  }

  .command-output-container:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  /* Custom scrollbar */
  .command-outputs::-webkit-scrollbar {
    width: 8px;
  }

  .command-outputs::-webkit-scrollbar-track {
    background: #1f2937;
  }

  .command-outputs::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
  }

  .command-outputs::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
