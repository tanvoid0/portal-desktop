import { tauriTaskService } from "../services/tauriTaskService";
import type { Task } from "../types";

export async function fetchAllTasks(): Promise<Task[]> {
  return tauriTaskService.getTasks();
}

export async function fetchTaskById(id: string): Promise<Task | null> {
  return tauriTaskService.getTask(id);
}
