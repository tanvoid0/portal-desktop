<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { terminalStore, terminalActions, activeTab, activeProcess } from '../stores/terminalStore';
  import { TerminalService } from '../services/terminalService';
  import type { TerminalSettings, TerminalProcess, TerminalOutput } from '../types';
  
  export let tabId: string;
  export let settings: TerminalSettings;
  export let initialProfile: string = ''; // Optional profile to start with

  let terminal: Terminal;
  let currentProcess: TerminalProcess | null = null;
  let isConnected = false;
  let outputBuffer = '';
  let inputBuffer = '';
  let unsubscribe: (() => void) | null = null;
  let systemInfo: any = null;
  let availableProfiles: any[] = [];
  let selectedProfile: string = '';

  const options = {
    theme: {
      background: '#1f2937',
      foreground: '#f9fafb',
      cursor: '#10b981'
    },
    fontSize: settings.fontSize,
    fontFamily: settings.fontFamily,
    cursorStyle: settings.cursorStyle,
    scrollback: settings.scrollbackLines
  };

  onMount(async () => {
    await loadSystemInfo();
    setupCommandInterceptors();
    setupOutputParsers();
    
    // Initialize xterm.js terminal
    terminal = new Terminal(options);
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    
    // Open terminal in the div
    const terminalElement = document.getElementById('terminal');
    if (terminalElement) {
      terminal.open(terminalElement);
      fitAddon.fit();
    }
    
    // Set up event handlers
    terminal.onData(onData);
    terminal.onKey(onKey);
    
    // Focus the terminal
    terminal.focus();
    
    // Initialize terminal
    onLoad();
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
    if (currentProcess) {
      TerminalService.killProcess(currentProcess.id);
    }
  });

  async function loadSystemInfo() {
    try {
      systemInfo = await TerminalService.getSystemInfo();
      
      // Extract available profiles from native system detection
      availableProfiles = [];
      if (systemInfo.terminalProfiles) {
        Object.entries(systemInfo.terminalProfiles).forEach(([category, profiles]: [string, any]) => {
          if (category === 'available_shells' && typeof profiles === 'object') {
            Object.entries(profiles).forEach(([name, profile]: [string, any]) => {
              availableProfiles.push({
                name,
                category,
                command: profile.command,
                args: profile.args || [],
                icon: profile.icon || 'terminal'
              });
            });
          }
        });
      }
      
      // Set default profile
      if (availableProfiles.length > 0) {
        if (initialProfile && availableProfiles.find(p => p.name === initialProfile)) {
          selectedProfile = initialProfile;
        } else {
          selectedProfile = availableProfiles[0].name;
        }
        settings.defaultShell = availableProfiles.find(p => p.name === selectedProfile)?.command || availableProfiles[0].command;
      }
    } catch (error) {
      console.error('Failed to load system info:', error);
    }
  }

  async function initializeTerminal() {
    try {
      // Use selected profile or fallback to settings
      const shellCommand = availableProfiles.find(p => p.name === selectedProfile)?.command || 
                          (navigator.userAgent.includes('Windows') ? 'powershell.exe' : 'bash');
      
      // Create real terminal process using domain-specific backend
      currentProcess = await TerminalService.createProcess(tabId, {
        shell: shellCommand,
        working_directory: settings.workingDirectory,
        cols: 80,
        rows: 24
      });

      // Subscribe to output
      unsubscribe = await TerminalService.subscribeToOutput(
        currentProcess.id,
        handleOutput
      );

      isConnected = true;
      terminal.write('Portal Desktop Terminal - Full Control Mode\r\n');
      terminal.write('Connected to real terminal process!\r\n');
      terminal.write(`Process ID: ${currentProcess.id}\r\n`);
      terminal.write(`Shell: ${shellCommand}\r\n`);
      terminal.write(`Working Directory: ${settings.workingDirectory}\r\n`);
      terminal.write('Type commands and press Enter to execute them.\r\n');
      terminal.write('Commands are intercepted and output is parsed.\r\n\r\n');
      
      // Send a command to get the shell to show a prompt
      setTimeout(() => {
        if (currentProcess) {
          console.log('Sending test command to process:', currentProcess.id);
          TerminalService.sendInput(currentProcess.id, 'echo "Terminal ready"\r\n').catch(error => {
            console.error('Failed to send test command:', error);
          });
        }
      }, 100);
    } catch (error) {
      console.error('Failed to initialize terminal:', error);
      terminal.write('Failed to connect to terminal process!\r\n');
      terminal.write('Falling back to simulated terminal...\r\n\r\n');
      setupSimulatedTerminal();
    }
  }

  function setupSimulatedTerminal() {
    terminal.write('Welcome to Portal Desktop Terminal!\r\n');
    terminal.write('Type commands and press Enter to execute them.\r\n');
    terminal.write('Available commands: help, clear, echo, ls, pwd, whoami, date\r\n\r\n');
    writePrompt();
  }

  function onLoad() {
    // Initialize terminal after it's loaded
    initializeTerminal();

    // Handle resize
    window.addEventListener('resize', () => {
      if (currentProcess) {
        TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
      }
    });
  }

  function onData(data: string) {
    // Parse the data if it's a JSON string (happens when data is wrapped in quotes)
    let processedData = data;
    try {
      // Check if data is wrapped in quotes and parse it
      if ((data.startsWith('"') && data.endsWith('"')) || 
          (data.startsWith("'") && data.endsWith("'"))) {
        processedData = JSON.parse(`"${data.slice(1, -1)}"`);
      }
    } catch (e) {
      console.warn('Failed to parse input data:', e);
    }
    
    if (isConnected && currentProcess) {
      // Handle special control characters
      if (processedData === '\r' || processedData === '\n') {
        // Enter pressed - send the buffered command
        if (inputBuffer.trim()) {
          console.log('Sending complete command to process:', currentProcess.id, 'command:', inputBuffer);
          TerminalService.sendInput(currentProcess.id, inputBuffer + '\r').catch(error => {
            console.error('Failed to send input:', error);
          });
          inputBuffer = '';
        }
      } else if (processedData === '\x7f' || processedData === '\b' || processedData === '\u007f') {
        // Backspace - remove last character from buffer and terminal
        if (inputBuffer.length > 0) {
          inputBuffer = inputBuffer.slice(0, -1);
          terminal.write('\b \b');
        }
      } else if (processedData === '\x03') {
        // Ctrl+C - send interrupt signal
        console.log('Sending Ctrl+C to process:', currentProcess.id);
        TerminalService.sendInput(currentProcess.id, '\x03').catch(error => {
          console.error('Failed to send Ctrl+C:', error);
        });
        inputBuffer = '';
      } else if (processedData.length > 0 && processedData !== '\x1b') {
        // Regular character - add to buffer and echo to terminal
        inputBuffer += processedData;
        terminal.write(processedData);
      }
    } else {
      console.log('Not connected, using simulated terminal');
      // Handle simulated terminal
      handleSimulatedInput(data);
    }
  }

  function onKey(data: { key: string; domEvent: KeyboardEvent }) {
    // Handle special keys
    if (data.key === 'c' && data.domEvent.ctrlKey) {
      // Ctrl+C - interrupt process
      if (currentProcess) {
        TerminalService.sendInput(currentProcess.id, '\x03');
      }
    }
  }

  function handleOutput(output: TerminalOutput) {
    terminal.write(output.content);
  }

  function handleSimulatedInput(data: string) {
    // Simulated terminal input handling
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
    switch (command.toLowerCase()) {
      case 'clear':
        terminal.clear();
        break;
      case 'help':
        terminal.write('Available commands:\r\n');
        terminal.write('  clear - Clear the terminal\r\n');
        terminal.write('  help - Show this help message\r\n');
        terminal.write('  echo <text> - Echo text\r\n');
        terminal.write('  ls - List files\r\n');
        terminal.write('  pwd - Show current directory\r\n');
        terminal.write('  whoami - Show current user\r\n');
        terminal.write('  date - Show current date\r\n');
        terminal.write('  connect - Try to connect to real terminal\r\n');
        break;
      case 'connect':
        terminal.write('Attempting to connect to real terminal...\r\n');
        initializeTerminal();
        break;
      default:
        if (command.startsWith('echo ')) {
          const text = command.substring(5);
          terminal.write(text + '\r\n');
        } else {
          // Try to execute as a real command using Tauri backend
          try {
            const result = await TerminalService.executeCommand(command);
            terminal.write(result + '\r\n');
          } catch (error) {
            terminal.write(`Command not found: ${command}\r\n`);
          }
        }
    }
  }

  function writePrompt() {
    if (!isConnected) {
      terminal.write('$ ');
    }
  }

  async function handleProfileChange() {
    if (currentProcess) {
      // Kill current process
      await TerminalService.killProcess(currentProcess.id);
      currentProcess = null;
      isConnected = false;
    }
    
    // Reinitialize with new profile
    await initializeTerminal();
  }

  function clearTerminal() {
    terminal.clear();
    if (!isConnected) {
      writePrompt();
    }
  }

  function focusTerminal() {
    terminal.focus();
  }

  function setupCommandInterceptors() {
    // Example: Intercept git commands for UI integration
    TerminalService.addCommandInterceptor({
      pattern: /^git\s+/,
      handler: async (command, process) => {
        terminal.write(`\r\n[INTERCEPTED] Git command: ${command}\r\n`);
        terminal.write('This command was intercepted for special handling.\r\n');
        // You can add custom logic here
        return true; // Intercepted
      }
    });

    // Example: Intercept npm commands
    TerminalService.addCommandInterceptor({
      pattern: /^npm\s+/,
      handler: async (command, process) => {
        terminal.write(`\r\n[INTERCEPTED] NPM command: ${command}\r\n`);
        // Add package management UI integration
        return false; // Don't intercept, let it run
      }
    });
  }

  function setupOutputParsers() {
    // Example: Parse error outputs
    TerminalService.addOutputParser({
      pattern: /error:|Error:|ERROR:/,
      handler: (output, process) => {
        // Highlight errors in UI
        console.log('Error detected:', output);
      }
    });

    // Example: Parse file paths
    TerminalService.addOutputParser({
      pattern: /\/[^\s]+/,
      handler: (output, process) => {
        // Make file paths clickable
        console.log('File path detected:', output);
      }
    });
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
          {#if $activeTab}
            {$activeTab.name}
          {:else}
            Portal Terminal
          {/if}
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
            PID: {currentProcess.id.slice(0, 8)}
          </div>
        {/if}
      </div>
    </div>

    <!-- Bottom Row: Terminal Info & Controls -->
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4 text-xs text-gray-400">
        {#if currentProcess}
          <span>Shell: {settings.defaultShell}</span>
          <span>Dir: {settings.workingDirectory}</span>
          <span>Size: {terminal?.cols || 80}×{terminal?.rows || 24}</span>
        {/if}
      </div>
      
      <div class="flex items-center space-x-2">
        <button
          on:click={clearTerminal}
          class="text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
          title="Clear terminal"
        >
          Clear
        </button>
        <button
          on:click={focusTerminal}
          class="text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
          title="Focus terminal"
        >
          Focus
        </button>
        {#if !isConnected}
          <button
            on:click={initializeTerminal}
            class="text-xs text-blue-400 hover:text-blue-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            title="Connect to real terminal"
          >
            Connect
          </button>
        {/if}
        {#if currentProcess}
          <button
            on:click={() => currentProcess && TerminalService.killProcess(currentProcess.id)}
            class="text-xs text-red-400 hover:text-red-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            title="Kill process"
          >
            Kill
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Main Content Area -->
  <div class="flex h-full" style="height: calc(100% - 80px);">
            <!-- Terminal Container -->
            <div class="terminal-container flex-1 h-full">
              <div id="terminal" class="h-full w-full"></div>
            </div>

    <!-- Control Panel Sidebar -->
    <div class="control-panel w-80 bg-gray-800 border-l border-gray-700 p-4 overflow-y-auto">
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
                <span>Process ID:</span>
                <span class="font-mono">{currentProcess.id.slice(0, 8)}</span>
              </div>
              <div class="flex justify-between">
                <span>Shell:</span>
                <span>{settings.defaultShell}</span>
              </div>
              <div class="flex justify-between">
                <span>Working Dir:</span>
                <span class="font-mono text-xs">{settings.workingDirectory}</span>
              </div>
              <div class="flex justify-between">
                <span>Terminal Size:</span>
                <span>{terminal?.cols || 80}×{terminal?.rows || 24}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Command Interceptors -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Command Interceptors</h3>
          <div class="space-y-2">
            <div class="bg-gray-700 rounded p-2 text-xs">
              <div class="text-green-400 font-mono">^git\s+</div>
              <div class="text-gray-400 mt-1">Intercepts git commands</div>
            </div>
            <div class="bg-gray-700 rounded p-2 text-xs">
              <div class="text-blue-400 font-mono">^npm\s+</div>
              <div class="text-gray-400 mt-1">Monitors npm commands</div>
            </div>
          </div>
        </div>

        <!-- Output Parsers -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Output Parsers</h3>
          <div class="space-y-2">
            <div class="bg-gray-700 rounded p-2 text-xs">
              <div class="text-red-400 font-mono">error:|Error:|ERROR:</div>
              <div class="text-gray-400 mt-1">Highlights errors</div>
            </div>
            <div class="bg-gray-700 rounded p-2 text-xs">
              <div class="text-yellow-400 font-mono">\/[^\s]+</div>
              <div class="text-gray-400 mt-1">Detects file paths</div>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Quick Actions</h3>
          <div class="space-y-2">
            <button
              on:click={clearTerminal}
              class="w-full text-left text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            >
              Clear Terminal
            </button>
            <button
              on:click={focusTerminal}
              class="w-full text-left text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
            >
              Focus Terminal
            </button>
            {#if !isConnected}
              <button
                on:click={initializeTerminal}
                class="w-full text-left text-xs text-blue-400 hover:text-blue-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
              >
                Connect to Real Terminal
              </button>
            {/if}
            {#if currentProcess}
              <button
                on:click={() => currentProcess && TerminalService.killProcess(currentProcess.id)}
                class="w-full text-left text-xs text-red-400 hover:text-red-300 px-2 py-1 rounded hover:bg-gray-700 transition-colors"
              >
                Kill Process
              </button>
            {/if}
          </div>
        </div>

                 <!-- Terminal Profile -->
                 <div>
                   <h3 class="text-sm font-medium text-gray-300 mb-3">Terminal Profile</h3>
                   <div class="space-y-2">
                     <select
                       bind:value={selectedProfile}
                       on:change={handleProfileChange}
                       class="w-full bg-gray-700 text-gray-200 text-xs px-2 py-1 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                     >
                       {#each availableProfiles as profile}
                         <option value={profile.name}>
                           {profile.icon} {profile.name}
                         </option>
                       {/each}
                     </select>
                     {#if selectedProfile}
                       {@const currentProfile = availableProfiles.find(p => p.name === selectedProfile)}
                       {#if currentProfile}
                         <div class="text-xs text-gray-400 space-y-1">
                           <div class="flex justify-between">
                             <span>Command:</span>
                             <span class="font-mono">{currentProfile.command}</span>
                           </div>
                           {#if currentProfile.args.length > 0}
                             <div class="flex justify-between">
                               <span>Args:</span>
                               <span class="font-mono text-xs">{currentProfile.args.join(' ')}</span>
                             </div>
                           {/if}
                         </div>
                       {/if}
                     {/if}
                   </div>
                 </div>

                 <!-- Terminal Settings -->
                 <div>
                   <h3 class="text-sm font-medium text-gray-300 mb-3">Settings</h3>
                   <div class="space-y-2 text-xs text-gray-400">
                     <div class="flex justify-between">
                       <span>Font Size:</span>
                       <span>{settings.fontSize}px</span>
                     </div>
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

        <!-- Help -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">Help</h3>
          <div class="text-xs text-gray-400 space-y-1">
            <div>• Type commands normally</div>
            <div>• Ctrl+C to interrupt</div>
            <div>• Commands are intercepted</div>
            <div>• Output is parsed</div>
            <div>• Full process control</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style global>
  .terminal-wrapper {
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
  }
  
  .terminal-container {
    position: relative;
    background: #1f2937;
  }
  
  .control-panel {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }
  
  .control-panel h3 {
    border-bottom: 1px solid #374151;
    padding-bottom: 0.5rem;
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
