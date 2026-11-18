export interface InstructionTemplate {
  id: string;
  name: string;
  instruction: string;
  isDefault?: boolean;
  createdAt: Date;
}

const STORAGE_KEY = 'task-instruction-templates';

export const defaultTemplates: InstructionTemplate[] = [
  {
    id: 'default-1',
    name: 'Standard Breakdown',
    instruction:
      'Break down into logical subtasks, prioritize developer-friendly descriptions, and estimate time for each subtask.',
    isDefault: true,
    createdAt: new Date(),
  },
  {
    id: 'default-2',
    name: 'Detailed Subtasks',
    instruction:
      'Create detailed subtasks with clear acceptance criteria. Focus on breaking down complex work into smaller, actionable items.',
    isDefault: true,
    createdAt: new Date(),
  },
  {
    id: 'default-3',
    name: 'API Focus',
    instruction:
      'Focus on API endpoints, request/response structures, and error handling. Break down by endpoint or resource.',
    isDefault: true,
    createdAt: new Date(),
  },
];

export function hasStoredTemplates(): boolean {
  return !!localStorage.getItem(STORAGE_KEY);
}

export function loadTemplatesFromStorage(): InstructionTemplate[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return parsed.map((t: any) => ({
        ...t,
        createdAt: new Date(t.createdAt),
      }));
    }
  } catch (error) {
    console.error('Failed to load templates from localStorage:', error);
  }
  return [...defaultTemplates];
}

export function saveTemplatesToStorage(templates: InstructionTemplate[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
  } catch (error) {
    console.error('Failed to save templates:', error);
  }
}
