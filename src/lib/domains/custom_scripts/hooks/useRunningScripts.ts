/**
 * Hook for managing multiple running scripts
 * Supports the same script running multiple times or different scripts running simultaneously
 */

import { TerminalService } from '$lib/domains/terminal/services/terminalService';
import type { TerminalProcess } from '$lib/domains/terminal/types';
import type { CustomScript } from '../services/customScriptService';

export interface RunningScript {
	id: string; // Unique ID for this running instance
	scriptId: number; // The script's database ID
	script: CustomScript; // The script object
	processId: string; // Terminal process ID
	tabId: string; // Terminal tab ID
	startTime: Date;
	stopCallback: () => Promise<void>;
	output: string; // Terminal output for this instance
	outputUnsubscribe?: () => void; // Function to unsubscribe from output
}

class RunningScriptsManager {
	private runningScripts = new Map<string, RunningScript>();
	private listeners = new Set<(scripts: RunningScript[]) => void>();

	/**
	 * Subscribe to running scripts changes
	 */
	subscribe(callback: (scripts: RunningScript[]) => void): () => void {
		this.listeners.add(callback);
		// Immediately call with current state
		callback(Array.from(this.runningScripts.values()));

		return () => {
			this.listeners.delete(callback);
		};
	}

	/**
	 * Notify all listeners of state changes
	 */
	private notify() {
		const scripts = Array.from(this.runningScripts.values());
		this.listeners.forEach((callback) => callback(scripts));
	}

	/**
	 * Add a running script
	 */
	add(
		script: CustomScript,
		processId: string,
		tabId: string,
		stopCallback: () => Promise<void>,
		outputUnsubscribe?: () => void
	): string {
		const id = `${script.id}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
		const runningScript: RunningScript = {
			id,
			scriptId: script.id,
			script,
			processId,
			tabId,
			startTime: new Date(),
			stopCallback,
			output: '',
			outputUnsubscribe,
		};

		this.runningScripts.set(id, runningScript);
		this.notify();
		return id;
	}

	/**
	 * Update output for a running script
	 */
	updateOutput(id: string, output: string) {
		const runningScript = this.runningScripts.get(id);
		if (runningScript) {
			runningScript.output = output;
			this.notify();
		}
	}

	/**
	 * Append output to a running script
	 */
	appendOutput(id: string, content: string) {
		const runningScript = this.runningScripts.get(id);
		if (runningScript) {
			runningScript.output += content;
			this.notify();
		}
	}

	/**
	 * Remove a running script
	 */
	remove(id: string) {
		const runningScript = this.runningScripts.get(id);
		if (runningScript?.outputUnsubscribe) {
			runningScript.outputUnsubscribe();
		}
		this.runningScripts.delete(id);
		this.notify();
	}

	/**
	 * Get all running scripts
	 */
	getAll(): RunningScript[] {
		return Array.from(this.runningScripts.values());
	}

	/**
	 * Get running scripts for a specific script ID
	 */
	getByScriptId(scriptId: number): RunningScript[] {
		return Array.from(this.runningScripts.values()).filter((rs) => rs.scriptId === scriptId);
	}

	/**
	 * Get a running script by its unique ID
	 */
	getById(id: string): RunningScript | undefined {
		return this.runningScripts.get(id);
	}

	/**
	 * Check if a script is running (any instance)
	 */
	isRunning(scriptId: number): boolean {
		return this.getByScriptId(scriptId).length > 0;
	}

	/**
	 * Get count of running instances for a script
	 */
	getRunningCount(scriptId: number): number {
		return this.getByScriptId(scriptId).length;
	}
}

// Singleton instance
const runningScriptsManager = new RunningScriptsManager();

/**
 * Hook to manage running scripts
 */
export function useRunningScripts() {
	return {
		subscribe: runningScriptsManager.subscribe.bind(runningScriptsManager),
		add: runningScriptsManager.add.bind(runningScriptsManager),
		remove: runningScriptsManager.remove.bind(runningScriptsManager),
		getAll: runningScriptsManager.getAll.bind(runningScriptsManager),
		getByScriptId: runningScriptsManager.getByScriptId.bind(runningScriptsManager),
		getById: runningScriptsManager.getById.bind(runningScriptsManager),
		isRunning: runningScriptsManager.isRunning.bind(runningScriptsManager),
		getRunningCount: runningScriptsManager.getRunningCount.bind(runningScriptsManager),
		updateOutput: runningScriptsManager.updateOutput.bind(runningScriptsManager),
		appendOutput: runningScriptsManager.appendOutput.bind(runningScriptsManager),
	};
}

