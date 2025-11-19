<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { terminalStore, terminalActions } from '../stores/terminalStore';
  import { TerminalService } from '../services/terminalService';
  import { commandHistoryStore } from '../stores/commandHistoryStore';
  import { sessionStore, type TerminalSession } from '../stores/sessionStore';
  import CommandHistory from './CommandHistory.svelte';
  import CommandHistorySearch from './CommandHistorySearch.svelte';
  import CommandBlocks from './CommandBlocks.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import ErrorSummary from './ErrorSummary.svelte';
  import { parseTerminalOutput, extractErrorSummary } from '../utils/outputParser';
  import type { TerminalConfig, TerminalProcess, TerminalOutput, TerminalSystemInfo, TerminalOutputEvent } from '../types';
  import type { CommandHistoryEntry } from '../stores/commandHistoryStore';
  
  interface Props {
    tabId: string;
    settings: TerminalConfig;
  }

  let {
    tabId,
    settings
  }: Props = $props();

  let terminal = $state<Terminal | null>(null);
  let fitAddon = $state<FitAddon | null>(null);
  let currentProcess = $state<TerminalProcess | null>(null);
  let isConnected = $state(false);
  let outputBuffer = $state('');
  let inputBuffer = $state('');
  let unsubscribe = $state<(() => void) | null>(null);
  let systemInfo = $state<TerminalSystemInfo | null>(null);
  let selectedEntry = $state<CommandHistoryEntry | null>(null);
  let showModal = $state(false);
  let savedOutput = $state('');
  let outputSaveInterval = $state<NodeJS.Timeout | null>(null);
  
  // Error tracking
  let errorCount = $state(0);
  let warningCount = $state(0);
  let infoCount = $state(0);
  let recentErrors = $state<string[]>([]);
  let terminalOutput = $state('');
  let sessionSaveTimeout = $state<NodeJS.Timeout | null>(null);

  // Shell-specific Quick Commands
  const quickCommands = {
    // Windows CMD commands
    windows: [
      { command: 'dir', description: 'List files', category: 'file' },
      { command: 'cd', description: 'Show current directory', category: 'navigation' },
      { command: 'cd ..', description: 'Go up one directory', category: 'navigation' },
      { command: 'git status', description: 'Check git status', category: 'git' },
      { command: 'npm list', description: 'List npm packages', category: 'npm' },
      { command: 'tasklist', description: 'List running processes', category: 'system' },
      { command: 'dir /s', description: 'List all files recursively', category: 'file' },
      { command: 'date /t', description: 'Show current date', category: 'utility' },
      { command: 'time /t', description: 'Show current time', category: 'utility' },
      { command: 'whoami', description: 'Show current user', category: 'utility' },
      { command: 'doskey /history', description: 'Show command history', category: 'utility' },
      { command: 'type package.json', description: 'View package.json', category: 'file' },
      { command: 'git log --oneline -5', description: 'Recent git commits', category: 'git' },
      { command: 'npm run dev', description: 'Start dev server', category: 'npm' },
      { command: 'npm install', description: 'Install dependencies', category: 'npm' },
      { command: 'mkdir temp', description: 'Create temp directory', category: 'file' },
      { command: 'echo. > test.txt', description: 'Create test file', category: 'file' },
      { command: 'del test.txt', description: 'Remove test file', category: 'file' },
      { command: 'findstr /r "TODO" *.js', description: 'Find TODO comments', category: 'search' },
      { command: 'dir /s *.js', description: 'Find JavaScript files', category: 'search' }
    ],
    // PowerShell commands
    powershell: [
      { command: 'Get-ChildItem', description: 'List files', category: 'file' },
      { command: 'Get-Location', description: 'Show current directory', category: 'navigation' },
      { command: 'Set-Location ..', description: 'Go up one directory', category: 'navigation' },
      { command: 'git status', description: 'Check git status', category: 'git' },
      { command: 'npm list', description: 'List npm packages', category: 'npm' },
      { command: 'Get-Process', description: 'List running processes', category: 'system' },
      { command: 'Get-Date', description: 'Show current date/time', category: 'utility' },
      { command: 'whoami', description: 'Show current user', category: 'utility' },
      { command: 'Get-History', description: 'Show command history', category: 'utility' },
      { command: 'Get-Content package.json', description: 'View package.json', category: 'file' },
      { command: 'git log --oneline -5', description: 'Recent git commits', category: 'git' },
      { command: 'npm run dev', description: 'Start dev server', category: 'npm' },
      { command: 'npm install', description: 'Install dependencies', category: 'npm' },
      { command: 'New-Item -ItemType Directory -Name temp', description: 'Create temp directory', category: 'file' },
      { command: 'New-Item -ItemType File -Name test.txt', description: 'Create test file', category: 'file' },
      { command: 'Remove-Item test.txt', description: 'Remove test file', category: 'file' },
      { command: 'Select-String -Pattern "TODO" -Path *.js', description: 'Find TODO comments', category: 'search' },
      { command: 'Get-ChildItem -Recurse -Filter "*.js"', description: 'Find JavaScript files', category: 'search' }
    ],
    // Linux/WSL commands
    linux: [
      { command: 'ls', description: 'List files', category: 'file' },
      { command: 'pwd', description: 'Show current directory', category: 'navigation' },
      { command: 'cd ..', description: 'Go up one directory', category: 'navigation' },
      { command: 'git status', description: 'Check git status', category: 'git' },
      { command: 'npm list', description: 'List npm packages', category: 'npm' },
      { command: 'ps aux', description: 'List running processes', category: 'system' },
      { command: 'df -h', description: 'Show disk usage', category: 'system' },
      { command: 'free -h', description: 'Show memory usage', category: 'system' },
      { command: 'date', description: 'Show current date/time', category: 'utility' },
      { command: 'whoami', description: 'Show current user', category: 'utility' },
      { command: 'history', description: 'Show command history', category: 'utility' },
      { command: 'cat package.json', description: 'View package.json', category: 'file' },
      { command: 'git log --oneline -5', description: 'Recent git commits', category: 'git' },
      { command: 'npm run dev', description: 'Start dev server', category: 'npm' },
      { command: 'npm install', description: 'Install dependencies', category: 'npm' },
      { command: 'mkdir temp', description: 'Create temp directory', category: 'file' },
      { command: 'touch test.txt', description: 'Create test file', category: 'file' },
      { command: 'rm test.txt', description: 'Remove test file', category: 'file' },
      { command: 'grep -r "TODO" .', description: 'Find TODO comments', category: 'search' },
      { command: 'find . -name "*.js"', description: 'Find JavaScript files', category: 'search' }
    ]
  };

  // Detect shell type and get appropriate commands
  const shellType = $derived(detectShellType(settings.defaultShell));
  const availableCommands = $derived(quickCommands[shellType] || quickCommands.linux);
  
  // Function to generate random commands
  function generateRandomCommands() {
    return availableCommands
      .sort(() => Math.random() - 0.5)
      .slice(0, Math.floor(Math.random() * 3) + 3);
  }
  
  let randomCommands = $state(generateRandomCommands());
  
  // Update random commands when available commands change
  $effect(() => {
    randomCommands = generateRandomCommands();
  });

  const options = {
    // Use more native-looking theme
    theme: {
      background: '#0c0c0c',  // More traditional terminal black
      foreground: '#cccccc',  // Standard terminal white
      cursor: '#ffffff',      // Standard white cursor
      selection: '#ffffff40'  // Subtle selection highlight
    },
    fontSize: settings.fontSize,
    fontFamily: settings.fontFamily,
    cursorStyle: settings.cursorStyle,
    scrollback: settings.scrollbackLines,
    windowsMode: true,
    // Remove custom styling that makes it look non-native
    allowTransparency: false,
    bellStyle: 'none'
  };

  onMount(async () => {
    console.log('Terminal component mounted for tab:', tabId);
    await loadSystemInfo();
    setupCommandInterceptors();
    setupOutputParsers();
    
    // Initialize xterm.js terminal
    terminal = new Terminal(options);
    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    
    // Open terminal in the div
    const terminalElement = document.getElementById(`terminal-${tabId}`);
    if (terminalElement) {
      terminal.open(terminalElement);
      fitAddon.fit();
    }
    
    // Set up event handlers
    terminal.onData(onData);
    terminal.onKey(onKey);
    
    // Focus the terminal
    terminal.focus();
    
    // Check for saved terminal output first
    const savedData = terminalActions.getSavedTerminalOutput(tabId);
    if (savedData && (savedData as any).output) {
      console.log('Restoring saved terminal output for tab:', tabId);
      savedOutput = (savedData as any).output;
      
      // Write the saved output to the terminal
      terminal.write(savedOutput);
      
      // Update working directory if saved
      if ((savedData as any).workingDirectory) {
        settings.workingDirectory = (savedData as any).workingDirectory;
      }
    }
    
    // Try to reconnect to existing process first
    await attemptProcessReconnection();
  });

  onDestroy(() => {
    console.log('Terminal component destroyed for tab:', tabId);
    
    // Save current terminal output before destroying
    if (terminal) {
      try {
        const buffer = terminal.buffer.active;
        const lines: string[] = [];
        for (let i = 0; i < terminal.rows; i++) {
          const line = buffer.getLine(i);
          if (line) {
            lines.push(line.translateToString());
          }
        }
        const currentOutput = lines.join('\n');
        terminalActions.saveTerminalOutput(tabId, currentOutput, currentProcess?.working_directory);
      } catch (error) {
        console.warn('Failed to save terminal output on destroy:', error);
      }
    }
    
    // Save session state before destruction
    saveSession();
    
    // Clear output save interval
    if (outputSaveInterval) {
      clearInterval(outputSaveInterval);
    }
    
    // Clear session save timeout
    if (sessionSaveTimeout) {
      clearTimeout(sessionSaveTimeout);
    }
    
    if (unsubscribe) {
      unsubscribe();
    }
    // Don't kill the process - keep it running in the background
    // The process will only be killed when the tab is explicitly closed
    console.log('Terminal component destroyed, keeping process running for tab:', tabId);
  });

  // Handle tab switching - ensure terminal is properly sized when shown
  $effect(() => {
    if (tabId) {
      console.log('Tab ID changed to:', tabId);
      // Resize terminal when tab becomes active
      setTimeout(() => {
        if (fitAddon) {
          fitAddon.fit();
        }
      }, 100);
    }
  });

  async function loadSystemInfo() {
    try {
      console.log('Terminal: Loading system info for tab:', tabId);
      const info = await TerminalService.getSystemInfo();
      systemInfo = info as unknown as TerminalSystemInfo;
      console.log('Terminal: System info loaded:', systemInfo);
    } catch (error) {
      console.error('Terminal: Failed to load system info:', error);
    }
  }

  async function attemptProcessReconnection() {
    // Check if there's an existing process for this tab
    const existingProcess = terminalActions.getProcessByTabId(tabId);
    if (existingProcess) {
      console.log('Found existing process for tab:', tabId, existingProcess);
      
      try {
        // Check if the process is still alive by trying to get its info
        const processInfo = await TerminalService.getProcess((existingProcess as any).id);
        if (processInfo && processInfo.status === 'running') {
          console.log('Process is still alive, reconnecting to PID:', processInfo.pid);
          currentProcess = processInfo;
          isConnected = true;
          
          // Subscribe to output for the existing process
          unsubscribe = await TerminalService.subscribeToOutput(
            (existingProcess as any).id,
            handleOutput
          );
          
          // Update the process status in the store
          terminalActions.updateProcess((existingProcess as any).id, { status: 'running' });
          
          // Set up periodic output saving
          setupOutputSaving();
          
          console.log('Successfully reconnected to existing process with PID:', processInfo.pid);
          return; // Successfully reconnected, don't create new process
        } else {
          console.log('Process is dead (backend restarted), creating new process with restored state...');
        }
      } catch (error) {
        console.log('Failed to reconnect to process (backend restarted), creating new process with restored state:', error);
      }
    }
    
    // If we get here, either no existing process or reconnection failed
    // This is expected after page reload since backend processes are killed
    console.log('Creating new terminal process with restored state...');
    onLoad();
  }

  function setupOutputSaving() {
    // Clear any existing interval
    if (outputSaveInterval) {
      clearInterval(outputSaveInterval);
    }
    
    // Set up periodic output saving
    outputSaveInterval = setInterval(() => {
      if (terminal && currentProcess) {
        try {
          // Get terminal content using the correct xterm.js API
          const buffer = terminal.buffer.active;
          const lines: string[] = [];
          for (let i = 0; i < terminal.rows; i++) {
            const line = buffer.getLine(i);
            if (line) {
              lines.push(line.translateToString());
            }
          }
          const currentOutput = lines.join('\n');
          terminalActions.saveTerminalOutput(tabId, currentOutput, currentProcess.working_directory);
        } catch (error) {
          console.warn('Failed to save terminal output:', error);
        }
      }
    }, 5000); // Save every 5 seconds
  }

  async function initializeTerminal() {
    try {
      // Load command history from backend
      await commandHistoryStore.loadFromBackend(tabId);
      
      // Use the shell from settings (set by the tab creation)
      const shellCommand = settings.defaultShell;
      
      console.log('Creating terminal process with shell:', shellCommand);
      
      // Create real terminal process using domain-specific backend
      currentProcess = await TerminalService.createProcess(tabId, {
        shell: shellCommand,
        working_directory: settings.workingDirectory,
        cols: 80,
        rows: 24
      });

      // console.log('Terminal process created:', currentProcess);

      // Load session state
      await loadSession();

      // Subscribe to output
      unsubscribe = await TerminalService.subscribeToOutput(
        currentProcess.id,
        handleOutput
      );

      // After process is ready, send an initial resize to match the renderer
      try {
        if (fitAddon && terminal) {
          fitAddon.fit();
          TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
        }
      } catch (e) {
        console.warn('Initial resize failed:', e);
      }

      isConnected = true;
      
      // Set up periodic output saving
      setupOutputSaving();
      
      // Don't inject custom messages - let the shell show its natural prompt
      console.log('Terminal connected successfully');
    } catch (error) {
      console.error('Failed to initialize terminal:', error);
      if (terminal) {
        terminal.write('\r\n❌ Failed to connect to terminal process!\r\n');
        terminal.write('🔄 Falling back to simulated terminal...\r\n\r\n');
        setupSimulatedTerminal();
      }
    }
  }

  function setupSimulatedTerminal() {
    if (!terminal) return;
    terminal.write('🎉 Welcome to Portal Desktop Terminal!\r\n');
    terminal.write('💡 Type commands and press Enter to execute them.\r\n');
    terminal.write('📋 Available commands: help, clear, echo, ls, pwd, whoami, date, connect\r\n');
    terminal.write('🔧 Try typing "connect" to attempt a real terminal connection.\r\n\r\n');
    writePrompt();
  }

  function onLoad() {
    // Initialize terminal after it's loaded
    initializeTerminal();

    // Handle resize
    window.addEventListener('resize', () => {
      if (!terminal) return;
      // Fit the renderer to the container first, then notify backend
      fitAddon?.fit();
      if (currentProcess) {
        TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
      }
    });
  }

  function onData(data: string) {
    // Forward keystrokes directly to the PTY when connected to avoid double-echo
    // and to allow the shell to handle line editing/history.
    let chunk = data;

    // Normalize Enter/newline across platforms: always send CRLF to the PTY on Enter
    if (chunk === '\n' || chunk === '\r') {
      chunk = '\r\n';
    }

    if (isConnected && currentProcess) {
      // Add a small delay to ensure proper input handling
      setTimeout(() => {
        TerminalService.sendInput(currentProcess!.id, chunk, tabId).catch((error) => {
        console.error('Failed to send input:', error);
      });
      }, 10);
      return;
    }

    // Fallback simulated terminal when not connected
    handleSimulatedInput(data);
  }

  function onKey(data: { key: string; domEvent: KeyboardEvent }) {
    // Handle special keys
    if (data.key === 'c' && data.domEvent.ctrlKey) {
      // Ctrl+C - interrupt process
      if (currentProcess) {
        TerminalService.sendInput(currentProcess.id, '\x03', tabId);
      }
    } else if (data.key === 'l' && data.domEvent.ctrlKey) {
      // Ctrl+L - clear terminal
      data.domEvent.preventDefault();
      clearTerminal();
    } else if (data.key === '+' && data.domEvent.ctrlKey) {
      // Ctrl++ - increase font size
      data.domEvent.preventDefault();
      adjustFontSize(1);
    } else if (data.key === '-' && data.domEvent.ctrlKey) {
      // Ctrl+- - decrease font size
      data.domEvent.preventDefault();
      adjustFontSize(-1);
    } else if (data.key === '0' && data.domEvent.ctrlKey) {
      // Ctrl+0 - fit terminal
      data.domEvent.preventDefault();
      fitTerminal();
    }
  }

  function handleOutput(output: TerminalOutput) {
    if (!terminal) return;
    terminal.write(output.content);
    
    // Update error summary
    updateErrorSummary(output.content);
    
    // Save output periodically when new content arrives
    if (currentProcess && terminal) {
      try {
        const buffer = terminal.buffer.active;
        const lines: string[] = [];
        for (let i = 0; i < terminal.rows; i++) {
          const line = buffer.getLine(i);
          if (line) {
            lines.push(line.translateToString());
          }
        }
        const currentOutput = lines.join('\n');
        terminalActions.saveTerminalOutput(tabId, currentOutput, currentProcess.working_directory);
        
        // Throttle session saving to prevent excessive calls
        if (sessionSaveTimeout) {
          clearTimeout(sessionSaveTimeout);
        }
        sessionSaveTimeout = setTimeout(() => {
          saveSession();
        }, 2000); // Save every 2 seconds max
      } catch (error) {
        console.warn('Failed to save terminal output on new content:', error);
      }
    }
    
    // Check if this looks like a command completion (new prompt)
    // Note: We don't automatically complete with exit code 0 anymore
    // The exit code will be handled by the process exit event
    if (output.content.includes('D:\\>') || output.content.includes('PS D:\\>') || 
        output.content.includes('$ ') || output.content.includes('# ')) {
      // Command completed, but let the exit event handle the exit code
      // This is just a fallback for cases where exit events don't fire
      if (!output.content.includes('Process exited with code:')) {
        TerminalService.completeCurrentCommand(undefined, tabId);
      }
    }
  }

  function handleSimulatedInput(data: string) {
    // Simulated terminal input handling
    if (!terminal) return;
    if (data === '\r') {
      terminal.write('\r\n');
      const command = outputBuffer.trim();
      if (command) {
        processSimulatedCommand(command);
      }
      outputBuffer = '';
      writePrompt();
    } else if (data === '\u007f') { // Backspace
      if (outputBuffer.length > 0) {
        outputBuffer = outputBuffer.slice(0, -1);
        terminal.write('\b \b');
      }
    } else if (data >= ' ') { // Printable characters
      outputBuffer += data;
      terminal.write(data);
    }
  }

  async function processSimulatedCommand(command: string) {
    if (!terminal) return;
    switch (command.toLowerCase()) {
      case 'clear':
        terminal.clear();
        break;
      case 'help':
        terminal.write('📚 Available commands:\r\n');
        terminal.write('  🧹 clear - Clear the terminal\r\n');
        terminal.write('  ❓ help - Show this help message\r\n');
        terminal.write('  📢 echo <text> - Echo text\r\n');
        terminal.write('  📁 ls - List files\r\n');
        terminal.write('  📍 pwd - Show current directory\r\n');
        terminal.write('  👤 whoami - Show current user\r\n');
        terminal.write('  📅 date - Show current date\r\n');
        terminal.write('  🔌 connect - Try to connect to real terminal\r\n');
        terminal.write('  🎯 status - Show terminal status\r\n');
        break;
      case 'connect':
        terminal.write('🔌 Attempting to connect to real terminal...\r\n');
        initializeTerminal();
        break;
      case 'status':
        terminal.write('📊 Terminal Status:\r\n');
        terminal.write(`  🔗 Connected: ${isConnected ? '✅ Yes' : '❌ No'}\r\n`);
        terminal.write(`  🆔 Process ID: ${currentProcess?.id.slice(0, 8) || 'None'}...\r\n`);
        terminal.write(`  🖥️  Shell: ${currentProcess?.command || 'Simulated'}\r\n`);
        terminal.write(`  📁 Working Dir: ${settings.workingDirectory}\r\n`);
        break;
      case 'ls':
        terminal.write('📁 Directory listing (simulated):\r\n');
        terminal.write('  📄 file1.txt\r\n');
        terminal.write('  📄 file2.txt\r\n');
        terminal.write('  📁 folder1/\r\n');
        terminal.write('  📁 folder2/\r\n');
        break;
      case 'pwd':
        terminal.write(`📍 Current directory: ${settings.workingDirectory}\r\n`);
        break;
      case 'whoami':
        terminal.write('👤 Current user: portal-user\r\n');
        break;
      case 'date':
        terminal.write(`📅 Current date: ${new Date().toLocaleString()}\r\n`);
        break;
      default:
        if (command.startsWith('echo ')) {
          const text = command.substring(5);
          terminal.write(`📢 ${text}\r\n`);
        } else {
          // Try to execute as a real command using Tauri backend
          try {
            const result = await TerminalService.executeCommand(command);
            terminal.write(result + '\r\n');
          } catch (error) {
            terminal.write(`❌ Command not found: ${command}\r\n`);
            terminal.write('💡 Type "help" to see available commands\r\n');
          }
        }
    }
  }

  function writePrompt() {
    if (!terminal || isConnected) return;
    terminal.write('$ ');
  }

  function clearTerminal() {
    if (!terminal) return;
    terminal.clear();
    if (!isConnected) {
      writePrompt();
    }
  }

  function killCurrentProcess() {
    if (currentProcess) {
      TerminalService.killProcess(currentProcess.id);
    }
  }

  function rerunCommand(command: string) {
    if (terminal && command) {
      terminal.write(command + '\r');
    }
  }

  function updateErrorSummary(output: string) {
    terminalOutput += output;
    
    // Limit terminal output to prevent memory issues
    if (terminalOutput.length > 100000) {
      terminalOutput = terminalOutput.slice(-50000); // Keep last 50k characters
    }
    
    const parsed = parseTerminalOutput(terminalOutput);
    const summary = extractErrorSummary(parsed);
    
    errorCount = summary.errorCount;
    warningCount = summary.warningCount;
    infoCount = summary.infoCount;
    recentErrors = summary.errors;
  }

  function clearErrorSummary() {
    errorCount = 0;
    warningCount = 0;
    infoCount = 0;
    recentErrors = [];
    terminalOutput = '';
  }

  async function saveSession() {
    if (!terminal) return;
    try {
      // Get scrollback buffer
      const buffer = terminal.buffer.active;
      const scrollbackLines: string[] = [];
      
      for (let i = 0; i < Math.min(buffer.length, 10000); i++) {
        const line = buffer.getLine(i);
        if (line) {
          const lineText = line.translateToString(true);
          if (lineText.trim()) {
            scrollbackLines.push(lineText);
          }
        }
      }
      
      const session: TerminalSession = {
        tab_id: tabId,
        working_directory: settings.workingDirectory,
        environment: {
          TERM: 'xterm-256color',
          COLORTERM: 'truecolor',
          SHELL: settings.defaultShell,
          HOME: settings.workingDirectory,
          USER: 'user'
        },
        scrollback_buffer: scrollbackLines,
        cursor_position: [terminal.buffer.active.cursorX, terminal.buffer.active.cursorY],
        terminal_size: [terminal.cols, terminal.rows],
        last_activity: new Date().toISOString(),
        process_id: currentProcess?.id
      };
      
      await sessionStore.saveSession(session);
    } catch (error) {
      console.error('Failed to save session:', error);
    }
  }

  async function loadSession() {
    try {
      const session = await sessionStore.loadSession(tabId);
      if (session && terminal) {
        // Restore scrollback buffer
        if (session.scrollback_buffer.length > 0) {
          terminal.clear();
          for (const line of session.scrollback_buffer) {
            terminal.write(line + '\r\n');
          }
        }
        
        // Restore cursor position
        terminal.write('\x1b[' + session.cursor_position[1] + ';' + session.cursor_position[0] + 'H');
        
        console.log('Session restored for tab:', tabId);
      }
    } catch (error) {
      console.error('Failed to load session:', error);
    }
  }

  function focusTerminal() {
    if (!terminal) return;
    terminal.focus();
  }

  function executeQuickCommand(command: string) {
    if (isConnected && currentProcess) {
      console.log('Executing quick command:', command);
      
      // Start tracking this command for history
      TerminalService.startCommandTracking(command, tabId);
      
      // Send the command to the terminal
      TerminalService.sendInput(currentProcess.id, command + '\r\n', tabId).catch((error) => {
        console.error('Failed to execute quick command:', error);
      });
    } else {
      console.warn('Terminal not connected, cannot execute command');
    }
  }

  function getCategoryIcon(category: string): string {
    switch (category) {
      case 'file': return '📁';
      case 'navigation': return '🧭';
      case 'git': return '🌿';
      case 'npm': return '📦';
      case 'system': return '⚙️';
      case 'utility': return '🔧';
      case 'search': return '🔍';
      default: return '💻';
    }
  }

  function getCategoryColor(category: string): string {
    switch (category) {
      case 'file': return 'text-blue-400';
      case 'navigation': return 'text-green-400';
      case 'git': return 'text-orange-400';
      case 'npm': return 'text-red-400';
      case 'system': return 'text-purple-400';
      case 'utility': return 'text-yellow-400';
      case 'search': return 'text-pink-400';
      default: return 'text-gray-400';
    }
  }

  function detectShellType(shellCommand: string): 'windows' | 'powershell' | 'linux' {
    const shell = shellCommand.toLowerCase();
    
    if (shell.includes('powershell') || shell.includes('pwsh')) {
      return 'powershell';
    } else if (shell.includes('cmd') || shell.includes('command')) {
      return 'windows';
    } else if (shell.includes('wsl') || shell.includes('bash') || shell.includes('zsh') || shell.includes('fish')) {
      return 'linux';
    } else {
      // Default to linux for unknown shells
      return 'linux';
    }
  }

  function adjustFontSize(delta: number) {
    const newSize = Math.max(8, Math.min(24, settings.fontSize + delta));
    settings.fontSize = newSize;
    if (terminal) {
      terminal.options.fontSize = newSize;
    }
  }

  function resetFontSize() {
    settings.fontSize = 14;
    if (terminal) {
      terminal.options.fontSize = 14;
    }
  }

  function resizeTerminal(colsDelta: number, rowsDelta: number) {
    if (terminal && currentProcess) {
      const newCols = Math.max(20, Math.min(200, (terminal.cols || 80) + colsDelta));
      const newRows = Math.max(10, Math.min(100, (terminal.rows || 24) + rowsDelta));
      
      terminal.resize(newCols, newRows);
      TerminalService.resizeTerminal(currentProcess.id, newCols, newRows);
    }
  }

  function fitTerminal() {
    if (fitAddon) {
      fitAddon.fit();
      if (currentProcess && terminal) {
        TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
      }
    }
  }

  function formatTimestamp(date: Date): string {
    return date.toLocaleTimeString();
  }

  function formatCommandOutput(output: string, maxLength: number = 100): string {
    if (output.length <= maxLength) return output;
    return output.substring(0, maxLength) + '...';
  }

  function getStatusIcon(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return '🎯';
    if (entry.exitCode === 0) return '✅';
    if (entry.exitCode && entry.exitCode !== 0) return '❌';
    return '⏳';
  }

  function getStatusColor(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return 'text-blue-400';
    if (entry.exitCode === 0) return 'text-green-400';
    if (entry.exitCode && entry.exitCode !== 0) return 'text-red-400';
    return 'text-yellow-400';
  }

  function showEntryDetails(entry: CommandHistoryEntry) {
    selectedEntry = entry;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    selectedEntry = null;
  }

  function setupCommandInterceptors() {
    // All interceptors removed - commands pass through directly to backend
    console.log('No command interceptors - all commands pass through to backend');
  }

  function setupOutputParsers() {
    // All output parsers removed - clean output flow
    console.log('No output parsers - clean output flow to terminal');
  }
</script>

<div class="terminal-wrapper h-full w-full bg-gray-900">
  <!-- Terminal Header -->
  <div class="terminal-header bg-gray-800 px-4 py-2 border-b border-gray-700">
    <!-- Top Row: Terminal Controls -->
    <div class="flex items-center justify-between mb-2">
      <div class="flex items-center space-x-2">
        <div class="w-3 h-3 bg-red-500 rounded-full"></div>
        <div class="w-3 h-3 bg-yellow-500 rounded-full"></div>
        <div class="w-3 h-3 bg-green-500 rounded-full"></div>
        <span class="ml-4 text-sm text-gray-300 font-medium">
            Portal Terminal
        </span>
      </div>
      
      <!-- Connection Status -->
      <div class="flex items-center space-x-2">
                 <div class="flex items-center space-x-1">
                   <div class="w-2 h-2 rounded-full {isConnected ? 'bg-green-500' : 'bg-yellow-500'}"></div>
                   <span class="text-xs text-gray-400">
                     {isConnected ? 'Connected' : 'Loading...'}
                   </span>
                 </div>
        
        {#if currentProcess}
          <div class="text-xs text-gray-400">
            PID: {currentProcess.pid ?? currentProcess.id.slice(0, 8)}
          </div>
        {/if}
      </div>
    </div>

    <!-- Bottom Row: Terminal Info & Controls -->
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4 text-xs text-gray-400">
        {#if currentProcess}
          <span>Shell: {currentProcess ? currentProcess.command.split(' ')[0] : settings.defaultShell}</span>
          <span>Dir: {currentProcess?.working_directory || settings.workingDirectory}</span>
          <span>Size: {terminal?.cols || 80}×{terminal?.rows || 24}</span>
        {/if}
      </div>
      
      <div class="flex items-center space-x-2">
        <span class="text-xs text-gray-500">
          {shellType === 'windows' ? 'CMD' : shellType === 'powershell' ? 'PowerShell' : 'Linux/WSL'}
        </span>
      </div>
    </div>
  </div>

  <!-- Main Content Area -->
  <div class="flex h-full" style="height: calc(100% - 80px);">
    <!-- Terminal Container -->
    <div class="terminal-container flex-1 h-full overflow-hidden">
      <div id="terminal-{tabId}" class="h-full w-full"></div>
    </div>

    <!-- Right Sidebar - Responsive -->
    <div class="sidebar w-80 lg:w-96 bg-gray-800 border-l border-gray-700 flex flex-col">
      <!-- Command History Section -->
      <div class="command-history-section flex-shrink-0 border-b border-gray-700" style="height: 40%;">
        <div class="h-full flex flex-col">
          <div class="flex items-center justify-between p-3 border-b border-gray-700 bg-gray-750">
            <h2 class="text-sm font-medium text-gray-300">Command History</h2>
            <button
              onclick={() => commandHistoryStore.clearHistory(tabId)}
              class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
              type="button"
              title="Clear history for this tab"
            >
              Clear
            </button>
          </div>
          
          <!-- Search Component -->
          <CommandHistorySearch {tabId} />
          <div class="flex-1 overflow-y-auto">
            {#each commandHistoryStore.getFilteredHistory(tabId) as entry (entry.id)}
              <button
                class="w-full p-3 border-b border-gray-800 hover:bg-gray-800 cursor-pointer transition-colors text-left"
                onclick={() => showEntryDetails(entry)}
                onkeydown={(e) => e.key === 'Enter' && showEntryDetails(entry)}
                type="button"
              >
                <div class="flex items-start justify-between mb-2">
                  <div class="flex items-center space-x-2">
                    <span class="text-xs {getStatusColor(entry)}">
                      {getStatusIcon(entry)}
                    </span>
                    <code class="text-sm text-gray-200 font-mono bg-gray-900 px-2 py-1 rounded">
                      {entry.command}
                    </code>
                  </div>
                  <span class="text-xs text-gray-500">
                    {formatTimestamp(entry.timestamp)}
                  </span>
                </div>
                
                <div class="text-xs text-gray-400">
                  {formatCommandOutput(entry.output, 60)}
                </div>
                
                {#if entry.duration}
                  <div class="text-xs text-gray-500 mt-1">
                    Duration: {entry.duration}ms
                  </div>
                {/if}
              </button>
            {:else}
              <div class="p-6 text-center text-gray-500">
                <div class="text-2xl mb-2">📝</div>
                <div class="text-sm">No commands executed yet</div>
                <div class="text-xs mt-1">Commands will appear here as you use the terminal</div>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Error Summary Section -->
      <div class="error-summary-section flex-shrink-0 border-b border-gray-700" style="height: 20%;">
        <ErrorSummary 
          {errorCount}
          {warningCount}
          {infoCount}
          errors={recentErrors}
          onClear={clearErrorSummary}
        />
      </div>

      <!-- Command Blocks Section -->
      <div class="command-blocks-section flex-shrink-0 border-b border-gray-700" style="height: 30%;">
        <CommandBlocks processId={currentProcess?.id || ''} />
      </div>

      <!-- Control Panel Section -->
      <div class="control-panel-section flex-1 overflow-y-auto p-4">
      <div class="space-y-6">
        <!-- System Information -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">System Info</h3>
          <div class="space-y-2 text-xs text-gray-400">
                     <div class="flex justify-between">
                       <span>Status:</span>
                       <span class="{isConnected ? 'text-green-400' : 'text-yellow-400'}">
                         {isConnected ? 'Connected' : 'Loading...'}
                       </span>
                     </div>
            {#if currentProcess}
              <div class="flex justify-between">
                <span>PID:</span>
                <span class="font-mono">{currentProcess.pid ?? currentProcess.id.slice(0, 8)}</span>
              </div>
              <div class="flex justify-between">
                <span>Shell:</span>
                <span>{currentProcess ? currentProcess.command.split(' ')[0] : settings.defaultShell}</span>
              </div>
              <div class="flex justify-between">
                <span>Working Dir:</span>
                <span class="font-mono text-xs">{currentProcess?.working_directory || settings.workingDirectory}</span>
              </div>
              <div class="flex justify-between">
                <span>Terminal Size:</span>
                <span>{terminal?.cols || 80}×{terminal?.rows || 24}</span>
              </div>
            {/if}
          </div>
        </div>

                      <!-- Quick Commands -->
        <div>
                        <h3 class="text-sm font-medium text-gray-300 mb-3">
                          Quick Commands 
                          <span class="text-xs text-gray-500 ml-2">
                            ({shellType === 'windows' ? 'CMD' : shellType === 'powershell' ? 'PowerShell' : 'Linux/WSL'})
                          </span>
                        </h3>
          <div class="space-y-2">
            {#each randomCommands as cmd (cmd.command)}
              <button
                onclick={() => executeQuickCommand(cmd.command)}
                class="w-full text-left p-2 rounded hover:bg-gray-700 transition-colors group"
                disabled={!isConnected}
                title={cmd.description}
              >
                <div class="flex items-center space-x-2">
                  <span class="text-sm">{getCategoryIcon(cmd.category)}</span>
                  <div class="flex-1 min-w-0">
                    <div class="font-mono text-xs {isConnected ? 'text-gray-200' : 'text-gray-500'} group-hover:text-white transition-colors">
                      {cmd.command}
            </div>
                    <div class="text-xs {getCategoryColor(cmd.category)} truncate">
                      {cmd.description}
            </div>
          </div>
        </div>
              </button>
            {/each}
                          <button
                            onclick={() => {
                              // Refresh random commands for current shell type
                              randomCommands = generateRandomCommands();
                            }}
                            class="w-full text-center text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
                            title="Get new random commands for current shell"
                          >
                            🔄 Refresh Commands
                          </button>
          </div>
        </div>


        <!-- Quick Actions -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Quick Actions</h3>
          <div class="space-y-2">
            <button
              onclick={clearTerminal}
              class="w-full text-left text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            >
              Clear Terminal
            </button>
            <button
              onclick={focusTerminal}
              class="w-full text-left text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            >
              Focus Terminal
            </button>
            {#if !isConnected}
              <button
                onclick={initializeTerminal}
                class="w-full text-left text-xs text-blue-400 hover:text-blue-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
              >
                Connect to Real Terminal
              </button>
            {/if}
            {#if currentProcess}
              <button
                onclick={() => currentProcess && TerminalService.killProcess(currentProcess.id)}
                class="w-full text-left text-xs text-red-400 hover:text-red-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
              >
                Kill Process
              </button>
            {/if}
          </div>
        </div>


                 <!-- Terminal Settings -->
                 <div>
                   <h3 class="text-sm font-medium text-gray-300 mb-3">Settings</h3>
                   <div class="space-y-3 text-xs text-gray-400">
                     <!-- Font Size Control -->
                     <div>
                       <div class="flex justify-between mb-1">
                         <span>Font Size:</span>
                         <span>{settings.fontSize}px</span>
                       </div>
                       <div class="flex space-x-1">
                         <button
                           onclick={() => adjustFontSize(-1)}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Decrease font size"
                         >
                           A-
                         </button>
                         <button
                           onclick={() => adjustFontSize(1)}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Increase font size"
                         >
                           A+
                         </button>
                         <button
                           onclick={() => resetFontSize()}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Reset font size"
                         >
                           Reset
                         </button>
                       </div>
                     </div>
                     
                     <!-- Terminal Resize -->
                     <div>
                       <div class="flex justify-between mb-1">
                         <span>Terminal Size:</span>
                         <span>{terminal?.cols || 80}×{terminal?.rows || 24}</span>
                       </div>
                       <div class="flex space-x-1">
                         <button
                           onclick={() => resizeTerminal(-10, -5)}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Make smaller"
                         >
                           Smaller
                         </button>
                         <button
                           onclick={() => resizeTerminal(10, 5)}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Make larger"
                         >
                           Larger
                         </button>
                         <button
                           onclick={() => fitTerminal()}
                           class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
                           title="Fit to container"
                         >
                           Fit
                         </button>
                       </div>
                     </div>
                     
                     <!-- Read-only Settings -->
                     <div class="space-y-2 pt-2 border-t border-gray-700">
                       <div class="flex justify-between">
                         <span>Font Family:</span>
                         <span class="font-mono text-xs">{settings.fontFamily}</span>
                       </div>
                       <div class="flex justify-between">
                         <span>Scrollback:</span>
                         <span>{settings.scrollbackLines} lines</span>
                       </div>
                       <div class="flex justify-between">
                         <span>Cursor Style:</span>
                         <span>{settings.cursorStyle}</span>
                       </div>
                     </div>
                   </div>
                 </div>

        <!-- Keyboard Shortcuts -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Keyboard Shortcuts</h3>
          <div class="text-xs text-gray-400 space-y-1">
            <div class="flex justify-between">
              <span>Clear terminal:</span>
              <span class="font-mono">Ctrl+L</span>
            </div>
            <div class="flex justify-between">
              <span>Interrupt command:</span>
              <span class="font-mono">Ctrl+C</span>
            </div>
            <div class="flex justify-between">
              <span>Font size up:</span>
              <span class="font-mono">Ctrl++</span>
            </div>
            <div class="flex justify-between">
              <span>Font size down:</span>
              <span class="font-mono">Ctrl+-</span>
            </div>
            <div class="flex justify-between">
              <span>Fit terminal:</span>
              <span class="font-mono">Ctrl+0</span>
            </div>
          </div>
        </div>

        <!-- Help -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Help</h3>
          <div class="text-xs text-gray-400 space-y-1">
            <div>• Type commands normally</div>
            <div>• Commands execute directly</div>
            <div>• History is tracked per tab</div>
            <div>• Multi-tab support</div>
            <div>• Shell-specific quick commands</div>
            <div>• Use sidebar controls for settings</div>
          </div>
        </div>
      </div>
      </div>
    </div>
  </div>

  <!-- Command Palette -->
  <CommandPalette 
    {tabId}
    onKillProcess={killCurrentProcess}
    onClearTerminal={clearTerminal}
    onRerunCommand={rerunCommand}
  />
</div>

<!-- Command Details Modal -->
{#if showModal && selectedEntry}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" 
    onclick={closeModal}
    onkeydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div 
      class="bg-gray-900 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col" 
      role="document"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-700">
        <h2 class="text-lg font-medium text-gray-200">Command Details</h2>
        <button
          onclick={closeModal}
          class="text-gray-400 hover:text-gray-200 text-xl"
        >
          ×
        </button>
      </div>

      <!-- Modal Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <!-- Command -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-2">Command</h3>
          <code class="block bg-gray-800 text-green-400 p-3 rounded font-mono text-sm">
            {selectedEntry.command}
          </code>
        </div>

        <!-- Metadata -->
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-gray-400">Timestamp:</span>
            <span class="text-gray-200 ml-2">{selectedEntry.timestamp.toLocaleString()}</span>
          </div>
          <div>
            <span class="text-gray-400">Status:</span>
            <span class="text-gray-200 ml-2">
              {getStatusIcon(selectedEntry)} {selectedEntry.exitCode !== undefined ? `Exit Code: ${selectedEntry.exitCode}` : 'Running'}
            </span>
          </div>
          {#if selectedEntry.duration}
            <div>
              <span class="text-gray-400">Duration:</span>
              <span class="text-gray-200 ml-2">{selectedEntry.duration}ms</span>
            </div>
          {/if}
          {#if selectedEntry.intercepted}
            <div>
              <span class="text-gray-400">Intercepted:</span>
              <span class="text-blue-400 ml-2">Yes 🎯</span>
            </div>
          {/if}
        </div>

        <!-- Output -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-2">Output</h3>
          <pre class="block bg-gray-800 text-gray-200 p-3 rounded font-mono text-sm whitespace-pre-wrap break-all">
            {selectedEntry.output}
          </pre>
        </div>
      </div>
    </div>
  </div>
{/if}

<style global>
  .terminal-wrapper {
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
  }

  /* Responsive sidebar */
  .sidebar {
    transition: width 0.3s ease;
  }

  /* Mobile responsiveness */
  @media (max-width: 1024px) {
    .sidebar {
      width: 20rem; /* w-80 */
    }
  }

  @media (max-width: 768px) {
    .sidebar {
      width: 18rem; /* w-72 */
    }
    
    .command-history-section {
      height: 35% !important;
    }
  }

  @media (max-width: 640px) {
    .sidebar {
      width: 16rem; /* w-64 */
    }
    
    .command-history-section {
      height: 30% !important;
    }
  }

  /* Custom scrollbars */
  .command-history-section .overflow-y-auto::-webkit-scrollbar,
  .control-panel-section::-webkit-scrollbar {
    width: 6px;
  }

  .command-history-section .overflow-y-auto::-webkit-scrollbar-track,
  .control-panel-section::-webkit-scrollbar-track {
    background: #374151;
  }

  .command-history-section .overflow-y-auto::-webkit-scrollbar-thumb,
  .control-panel-section::-webkit-scrollbar-thumb {
    background: #6b7280;
    border-radius: 3px;
  }

  .command-history-section .overflow-y-auto::-webkit-scrollbar-thumb:hover,
  .control-panel-section::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Terminal scrollbar */
  .terminal-container ::-webkit-scrollbar {
    width: 8px;
  }

  .terminal-container ::-webkit-scrollbar-track {
    background: #1f2937;
  }

  .terminal-container ::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
  }

  .terminal-container ::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
  
  .terminal-container {
    position: relative;
    background: #0c0c0c; /* More native terminal black */
  }
  
  .control-panel {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }
  
  
  .terminal-header {
    border-bottom: 1px solid #374151;
  }
  
  /* Custom scrollbar for control panel */
  .control-panel::-webkit-scrollbar {
    width: 6px;
  }
  
  .control-panel::-webkit-scrollbar-track {
    background: #1f2937;
  }
  
  .control-panel::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 3px;
  }
  
  .control-panel::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
  
  /* Terminal focus styles */
  .terminal-container:focus-within {
    box-shadow: inset 0 0 0 1px #10b981;
  }
  
  /* Button hover effects */
  button {
    transition: all 0.2s ease;
  }
  
  button:hover {
    transform: translateY(-1px);
  }
  
  /* Status indicator animation */
  .w-2.h-2.rounded-full {
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
