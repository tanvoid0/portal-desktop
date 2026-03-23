/**
 * Command Interception Service
 * Detects when commands need user input and handles interception
 */

export interface InputPrompt {
  type: 'password' | 'text' | 'confirm';
  prompt: string;
  pattern: RegExp;
}

export interface InterceptionResult {
  needsInput: boolean;
  prompt?: InputPrompt;
  shouldPause: boolean;
}

// Common input prompt patterns
const INPUT_PROMPTS: InputPrompt[] = [
  // Sudo password prompts (MOST SPECIFIC FIRST)
  {
    type: 'password',
    prompt: 'Password required for sudo',
    pattern: /\[sudo\]\s*password\s+for\s+\w+\s*:/i
  },
  {
    type: 'password',
    prompt: 'Password required',
    pattern: /password\s+for\s+\w+\s*:/i
  },
  {
    type: 'password',
    prompt: 'Password required',
    pattern: /password\s*[:\?]\s*$/im
  },
  {
    type: 'password',
    prompt: 'Enter password',
    pattern: /enter.*?password/i
  },
  {
    type: 'password',
    prompt: 'Password',
    pattern: /password:/i
  },
  
  // Confirmation prompts
  {
    type: 'confirm',
    prompt: 'Confirm action',
    pattern: /\(y\/n\)/i
  },
  {
    type: 'confirm',
    prompt: 'Confirm action',
    pattern: /\[y\/N\]/i
  },
  {
    type: 'confirm',
    prompt: 'Confirm action',
    pattern: /continue\?/i
  },
  {
    type: 'confirm',
    prompt: 'Confirm action',
    pattern: /are you sure/i
  },
  
  // Text input prompts
  {
    type: 'text',
    prompt: 'Input required',
    pattern: /enter\s+(?:your\s+)?(?:.*?):\s*$/i
  },
  {
    type: 'text',
    prompt: 'Input required',
    pattern: /please\s+enter/i
  }
];

export class CommandInterceptionService {
  /**
   * Check if output contains a prompt that requires user input
   */
  static checkForInputPrompt(output: string): InterceptionResult {
    if (!output) {
      return {
        needsInput: false,
        shouldPause: false
      };
    }

    // Check the entire output (not just last 5 lines) for sudo prompts
    // Sudo prompts might appear with lots of ANSI codes
    for (const prompt of INPUT_PROMPTS) {
      if (prompt.pattern.test(output)) {
        return {
          needsInput: true,
          prompt,
          shouldPause: true
        };
      }
    }

    // Also check just the last few lines (for other prompts)
    const lines = output.split('\n').slice(-10).join('\n');
    for (const prompt of INPUT_PROMPTS) {
      if (prompt.pattern.test(lines)) {
        return {
          needsInput: true,
          prompt,
          shouldPause: true
        };
      }
    }

    return {
      needsInput: false,
      shouldPause: false
    };
  }
  
  /**
   * Extract the prompt text from output
   */
  static extractPromptText(output: string, prompt: InputPrompt): string {
    const lines = output.split('\n');
    const lastLines = lines.slice(-5);
    
    for (const line of lastLines.reverse()) {
      if (prompt.pattern.test(line)) {
        return line.trim();
      }
    }
    
    return prompt.prompt;
  }
  
  /**
   * Check if a command should be intercepted before execution
   * (e.g., dangerous commands, commands that need confirmation)
   */
  static shouldInterceptBeforeExecution(command: string): boolean {
    const dangerousPatterns = [
      /^rm\s+-rf/i,
      /^sudo\s+rm\s+-rf/i,
      /^dd\s+if=/i,
      /^mkfs/i,
      /^fdisk/i,
      /^format/i
    ];
    
    return dangerousPatterns.some(pattern => pattern.test(command));
  }
  
  /**
   * Get interception reason for a command
   */
  static getInterceptionReason(command: string): string | null {
    if (/^rm\s+-rf/i.test(command)) {
      return 'This command will permanently delete files. Are you sure?';
    }
    if (/^sudo\s+rm\s+-rf/i.test(command)) {
      return 'This command will permanently delete files with sudo privileges. Are you sure?';
    }
    if (/^dd\s+if=/i.test(command)) {
      return 'This command can overwrite disk data. Are you sure?';
    }
    
    return null;
  }
  
  /**
   * Check if output indicates command is waiting for input
   */
  static isWaitingForInput(output: string): boolean {
    return this.checkForInputPrompt(output).needsInput;
  }
  
  /**
   * Normalize input for sending to terminal
   */
  static normalizeInput(input: string, type: 'password' | 'text' | 'confirm'): string {
    if (type === 'confirm') {
      const normalized = input.trim().toLowerCase();
      if (normalized === 'y' || normalized === 'yes') {
        return 'y\n';
      }
      if (normalized === 'n' || normalized === 'no') {
        return 'n\n';
      }
      return input + '\n';
    }
    
    return input + '\n';
  }
}

