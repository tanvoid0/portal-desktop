import { toast } from '$lib/utils/toast';
import { coderSession, type SubAgentNotificationEvent } from '../state/coderSession.svelte.js';

interface Nav {
  selectThread: (id: string) => Promise<void>;
  openSubAgent: (childThreadId: string, coordinatorId: string) => Promise<void>;
}

/** Turns a sub-agent lifecycle event into a toast with a "View" action. */
export function toastSubAgentEvent(event: SubAgentNotificationEvent, nav: Nav) {
  switch (event.type) {
    case 'spawned': {
      const label = `${event.count} sub-agent${event.count === 1 ? '' : 's'}`;
      if (coderSession.activeThreadId === event.coordinatorId) {
        toast.success(`Spawned ${label}`);
        break;
      }
      toast.success(`Spawned ${label}`, {
        action: {
          label: 'View',
          onClick: () => void nav.selectThread(event.coordinatorId),
        },
      });
      break;
    }
    case 'finished': {
      const sa = event.subAgent;
      const statusLabel =
        sa.status === 'completed' ? 'completed' : sa.status === 'failed' ? 'failed' : 'finished';
      const toastFn = sa.status === 'failed' ? toast.error : toast.success;
      toastFn(`Sub-agent ${statusLabel}`, {
        description: sa.title,
        action: {
          label: 'View',
          onClick: () => void nav.openSubAgent(sa.childThreadId, event.coordinatorId),
        },
      });
      break;
    }
    case 'complete': {
      const done = event.subAgents.filter((s) => s.status === 'completed').length;
      const failed = event.subAgents.filter((s) => s.status === 'failed').length;
      const parts = [`${done} completed`];
      if (failed > 0) parts.push(`${failed} failed`);
      toast.info('Multitask complete', {
        description: parts.join(', '),
        action: {
          label: 'View',
          onClick: () => void nav.selectThread(event.coordinatorId),
        },
      });
      break;
    }
  }
}
