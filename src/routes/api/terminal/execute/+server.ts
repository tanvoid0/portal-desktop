import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { spawn } from 'child_process';

export const POST: RequestHandler = async ({ request }) => {
  try {
    const { command } = await request.json();
    
    if (!command || typeof command !== 'string') {
      return new Response('Invalid command', { status: 400 });
    }

    // Execute the command
    const result = await executeCommand(command);
    
    return new Response(result, {
      headers: {
        'Content-Type': 'text/plain'
      }
    });
  } catch (error) {
    console.error('Command execution error:', error);
    return new Response(`Error: ${error.message}`, { status: 500 });
  }
};

function executeCommand(command: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const isWindows = process.platform === 'win32';
    const shell = isWindows ? 'cmd.exe' : 'bash';
    const args = isWindows ? ['/c', command] : ['-c', command];
    
    const child = spawn(shell, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      shell: false
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('close', (code) => {
      if (code === 0) {
        resolve(stdout || 'Command executed successfully');
      } else {
        resolve(stderr || `Command failed with exit code ${code}`);
      }
    });

    child.on('error', (error) => {
      reject(error);
    });
  });
}
