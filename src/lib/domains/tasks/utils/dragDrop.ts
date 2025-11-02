import type { Task } from '../types';

export interface DragDropState {
	isDragging: boolean;
	draggedTask: Task | null;
	dragOverColumn: string | null;
}

export function createDragDropState() {
	return {
		isDragging: false,
		draggedTask: null,
		dragOverColumn: null
	};
}

export function handleDragStart(
	event: DragEvent,
	task: Task,
	state: DragDropState
) {
	if (!event.dataTransfer) return;
	
	state.isDragging = true;
	state.draggedTask = task;
	
	// Set drag data
	event.dataTransfer.setData('text/plain', task.id);
	event.dataTransfer.effectAllowed = 'move';
	
	// Add visual feedback
	if (event.target instanceof HTMLElement) {
		event.target.style.opacity = '0.5';
	}
}

export function handleDragEnd(
	event: DragEvent,
	state: DragDropState
) {
	state.isDragging = false;
	state.draggedTask = null;
	state.dragOverColumn = null;
	
	// Reset visual feedback
	if (event.target instanceof HTMLElement) {
		event.target.style.opacity = '1';
	}
}

export function handleDragOver(
	event: DragEvent,
	columnId: string,
	state: DragDropState
) {
	event.preventDefault();
	event.dataTransfer!.dropEffect = 'move';
	state.dragOverColumn = columnId;
}

export function handleDragLeave(
	event: DragEvent,
	state: DragDropState
) {
	// Only clear if we're actually leaving the drop zone
	const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
	const x = event.clientX;
	const y = event.clientY;
	
	if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
		state.dragOverColumn = null;
	}
}

export function handleDrop(
	event: DragEvent,
	columnId: string,
	state: DragDropState,
	onTaskMove: (taskId: string, newStatus: string) => void
) {
	event.preventDefault();
	
	const taskId = event.dataTransfer?.getData('text/plain');
	if (!taskId || !state.draggedTask) return;
	
	// Map column IDs to status values
	const columnToStatus: Record<string, string> = {
		'pending': 'pending',
		'in-progress': 'in-progress',
		'completed': 'completed',
		'cancelled': 'cancelled'
	};
	
	const newStatus = columnToStatus[columnId];
	if (newStatus && newStatus !== state.draggedTask.status) {
		onTaskMove(taskId, newStatus);
	}
	
	state.isDragging = false;
	state.draggedTask = null;
	state.dragOverColumn = null;
}

export function getDropZoneClasses(
	columnId: string,
	state: DragDropState,
	baseClasses: string = ''
): string {
	const classes = [baseClasses];
	
	if (state.dragOverColumn === columnId) {
		classes.push('ring-2 ring-primary bg-primary/5');
	}
	
	if (state.isDragging) {
		classes.push('transition-all duration-200');
	}
	
	return classes.join(' ');
}

export function getTaskCardClasses(
	task: Task,
	state: DragDropState,
	baseClasses: string = ''
): string {
	const classes = [baseClasses];
	
	if (state.draggedTask?.id === task.id) {
		classes.push('opacity-50 scale-95');
	}
	
	return classes.join(' ');
}
