/**
 * Settings Domain Types
 */

export interface AppSettings {
	// General settings
	theme: 'light' | 'dark' | 'system';
	language: string;
	timezone: string;
	dateFormat: string;
	timeFormat: '12h' | '24h';
	
	// Window settings
	windowState: WindowState;
	startupBehavior: StartupBehavior;
	
	// Notifications
	notifications: NotificationSettings;
	
	// Privacy
	privacy: PrivacySettings;
	
	// Updates
	updates: UpdateSettings;
}

export interface WindowState {
	width: number;
	height: number;
	x?: number;
	y?: number;
	maximized: boolean;
	rememberPosition: boolean;
}

export interface StartupBehavior {
	openLastSession: boolean;
	restoreWindows: boolean;
	showWelcomeScreen: boolean;
	minimizeToTray: boolean;
	startMinimized: boolean;
}

export interface NotificationSettings {
	enabled: boolean;
	desktopNotifications: boolean;
	soundEnabled: boolean;
	showInTaskbar: boolean;
	types: NotificationTypeSettings;
}

export interface NotificationTypeSettings {
	success: boolean;
	info: boolean;
	warning: boolean;
	error: boolean;
	updates: boolean;
	security: boolean;
}

export interface PrivacySettings {
	analytics: boolean;
	crashReports: boolean;
	telemetry: boolean;
	usageData: boolean;
	marketing: boolean;
}

export interface UpdateSettings {
	autoCheck: boolean;
	autoDownload: boolean;
	autoInstall: boolean;
	checkInterval: number; // hours
	channel: 'stable' | 'beta' | 'alpha';
	notifyOnUpdate: boolean;
}

export interface EditorSettings {
	// Code editor
	fontFamily: string;
	fontSize: number;
	lineHeight: number;
	tabSize: number;
	insertSpaces: boolean;
	wordWrap: boolean;
	showLineNumbers: boolean;
	showMinimap: boolean;
	showWhitespace: boolean;
	
	// Syntax highlighting
	syntaxHighlighting: boolean;
	bracketMatching: boolean;
	autoIndent: boolean;
	
	// Code completion
	autoComplete: boolean;
	suggestions: boolean;
	parameterHints: boolean;
	
	// Themes
	editorTheme: string;
	terminalTheme: string;
	
	// Keybindings
	keybindings: KeybindingSettings;
}

export interface KeybindingSettings {
	[key: string]: string; // action -> key combination
}

export interface TerminalSettings {
	// Terminal appearance
	fontFamily: string;
	fontSize: number;
	lineHeight: number;
	cursorStyle: 'block' | 'underline' | 'line';
	cursorBlink: boolean;
	
	// Terminal behavior
	scrollback: number;
	bellStyle: 'none' | 'visual' | 'sound';
	rightClickSelectsWord: boolean;
	selectionMode: 'normal' | 'column';
	
	// Shell integration
	shellIntegration: boolean;
	commandHistory: boolean;
	commandSuggestions: boolean;
	
	// Terminal themes
	theme: TerminalTheme;
	
	// Advanced
	encoding: string;
	locale: string;
}

export interface TerminalTheme {
	name: string;
	background: string;
	foreground: string;
	cursor: string;
	selection: string;
	colors: {
		black: string;
		red: string;
		green: string;
		yellow: string;
		blue: string;
		magenta: string;
		cyan: string;
		white: string;
		brightBlack: string;
		brightRed: string;
		brightGreen: string;
		brightYellow: string;
		brightBlue: string;
		brightMagenta: string;
		brightCyan: string;
		brightWhite: string;
	};
}

export interface ThemeSettings {
	// Color scheme
	primaryColor: string;
	secondaryColor: string;
	accentColor: string;
	backgroundColor: string;
	surfaceColor: string;
	textColor: string;
	
	// UI elements
	borderRadius: number;
	shadowIntensity: number;
	animationSpeed: 'slow' | 'normal' | 'fast';
	
	// Custom themes
	customThemes: CustomTheme[];
	activeTheme: string;
}

export interface CustomTheme {
	id: string;
	name: string;
	description: string;
	colors: Record<string, string>;
	createdAt: Date;
	updatedAt: Date;
}

export interface KeyboardShortcuts {
	// Global shortcuts
	global: ShortcutGroup;
	
	// Application shortcuts
	application: ShortcutGroup;
	
	// Editor shortcuts
	editor: ShortcutGroup;
	
	// Terminal shortcuts
	terminal: ShortcutGroup;
}

export interface ShortcutGroup {
	[key: string]: ShortcutDefinition;
}

export interface ShortcutDefinition {
	key: string;
	modifiers?: string[];
	description: string;
	category: string;
	enabled: boolean;
}

export interface SettingsExport {
	version: string;
	exportedAt: Date;
	settings: {
		app: AppSettings;
		editor: EditorSettings;
		terminal: TerminalSettings;
		theme: ThemeSettings;
		shortcuts: KeyboardShortcuts;
	};
}

export interface SettingsImport {
	file: File;
	settings: SettingsExport;
	validation: ImportValidation;
}

export interface ImportValidation {
	valid: boolean;
	errors: string[];
	warnings: string[];
	compatibility: {
		version: string;
		compatible: boolean;
		missingFeatures: string[];
	};
}

export interface SettingsBackup {
	id: string;
	name: string;
	description?: string;
	createdAt: Date;
	size: number;
	settings: SettingsExport;
}

export interface SettingsCategory {
	id: string;
	name: string;
	description: string;
	icon: string;
	order: number;
	sections: SettingsSection[];
}

export interface SettingsSection {
	id: string;
	name: string;
	description: string;
	order: number;
	fields: SettingsField[];
}

export interface SettingsField {
	id: string;
	name: string;
	description: string;
	type: 'text' | 'number' | 'boolean' | 'select' | 'color' | 'file' | 'keybinding';
	value: any;
	defaultValue: any;
	options?: { value: any; label: string }[];
	validation?: {
		required?: boolean;
		min?: number;
		max?: number;
		pattern?: string;
		custom?: (value: any) => string | null;
	};
	dependsOn?: string[];
	visible?: boolean;
	disabled?: boolean;
}
