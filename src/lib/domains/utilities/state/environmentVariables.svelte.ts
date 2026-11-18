import { tick } from 'svelte';
import { beforeNavigate, goto } from '$app/navigation';
import { toast } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import { isTauriEnvironment } from '$lib/utils/tauri';
import { hasMultipleSegments } from '../utils/envSegments';
import type { EnvPermissions, EnvRow, EnvVariable } from '../types';
import {
  applyEnvironmentChanges,
  buildChanges,
  getEnvironmentPermissions,
  hasSystemChanges,
  listEnvironmentVariables,
  refreshProcessEnvironment,
} from '../services/environmentService';

export type EditableScope = 'user' | 'system';

function matchesSearch(row: EnvRow, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return row.name.toLowerCase().includes(q) || row.value.toLowerCase().includes(q);
}

/** Rows, edits, save/discard and the unsaved-changes guard behind EnvironmentVariablesPage. Construct during component init. */
export class EnvironmentVariablesState {
  loading = $state(true);
  saving = $state(false);
  error = $state<string | null>(null);
  tab = $state<EditableScope>('user');
  userSearch = $state('');
  systemSearch = $state('');
  permissions = $state<EnvPermissions | null>(null);
  original = $state<EnvVariable[]>([]);
  rows = $state<EnvRow[]>([]);
  expandedValues = $state<Set<string>>(new Set());
  private nextId = 1;

  private rowsForScope(scope: EditableScope): EnvRow[] {
    return this.rows.filter((row) => {
      if (row.isDeleted) return false;
      if (scope === 'user') {
        return row.scope === 'user' || row.scope === 'session';
      }
      return row.scope === 'system';
    });
  }

  userRows = $derived(
    this.rowsForScope('user').filter((row) => matchesSearch(row, this.userSearch))
  );

  systemRows = $derived(
    this.rowsForScope('system').filter((row) => matchesSearch(row, this.systemSearch))
  );

  dirtyCount = $derived(this.rows.filter((r) => r.isDirty || r.isNew || r.isDeleted).length);

  userDirtyCount = $derived.by(() => {
    const changes = buildChanges(this.original, this.rows);
    return changes.filter((c) => c.scope === 'user' || c.scope === 'session').length;
  });

  systemDirtyCount = $derived.by(() => {
    const changes = buildChanges(this.original, this.rows);
    return changes.filter((c) => c.scope === 'system').length;
  });

  needsElevation = $derived.by(() => {
    if (!this.permissions) return false;
    const changes = buildChanges(this.original, this.rows);
    return hasSystemChanges(changes) && !this.permissions.canEditSystem;
  });

  constructor() {
    let leaveConfirmed = false;
    beforeNavigate((nav) => {
      const { to, willUnload, cancel } = nav;
      if (leaveConfirmed || willUnload || !to?.url || to.url.origin !== location.origin) return;
      if (buildChanges(this.original, this.rows).length === 0) return;
      cancel();
      confirmAction('You have unsaved changes. Leave and discard them?', 'Unsaved changes', {
        confirmLabel: 'Leave',
        destructive: true,
      }).then((ok) => {
        if (!ok) return;
        leaveConfirmed = true;
        if (nav.type === 'popstate') history.go(nav.delta);
        else goto(to.url);
      });
    });
  }

  toggleExpanded = (id: string) => {
    const next = new Set(this.expandedValues);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    this.expandedValues = next;
  };

  load = async () => {
    if (!isTauriEnvironment()) {
      this.loading = false;
      this.error = 'Environment variable editing is only available in the desktop app.';
      return;
    }

    this.loading = true;
    this.error = null;
    try {
      const [vars, perms] = await Promise.all([
        listEnvironmentVariables(),
        getEnvironmentPermissions(),
      ]);
      this.original = vars;
      this.permissions = perms;
      this.rows = vars.map((v) => ({
        id: `row-${this.nextId++}`,
        name: v.name,
        value: v.value,
        scope: v.scope,
      }));
      this.expandedValues = new Set(
        this.rows.filter((r) => hasMultipleSegments(r.value)).map((r) => r.id)
      );
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Failed to load environment variables';
    } finally {
      this.loading = false;
    }
  };

  addRow = async (scope: EditableScope) => {
    this.tab = scope;
    // an active search would hide the empty new row
    if (scope === 'user') this.userSearch = '';
    else this.systemSearch = '';
    const id = `row-${this.nextId++}`;
    this.rows = [
      ...this.rows,
      {
        id,
        name: '',
        value: '',
        scope,
        isNew: true,
        isDirty: true,
      },
    ];
    await tick();
    document.getElementById(`name-${id}`)?.focus();
  };

  updateRow = (id: string, patch: Partial<EnvRow>) => {
    this.rows = this.rows.map((row) => {
      if (row.id !== id) return row;
      const updated = { ...row, ...patch, isDirty: true };
      if (!row.isNew) updated.isDirty = true;
      return updated;
    });
  };

  removeRow = (id: string) => {
    this.rows = this.rows.map((row) => {
      if (row.id !== id) return row;
      if (row.isNew) return { ...row, isDeleted: true };
      return { ...row, isDeleted: true, isDirty: true };
    });
  };

  discardChanges = () => {
    this.rows = this.original.map((v) => ({
      id: `row-${this.nextId++}`,
      name: v.name,
      value: v.value,
      scope: v.scope,
    }));
    toast.info('Changes discarded');
  };

  save = async () => {
    const changes = buildChanges(this.original, this.rows);
    if (changes.length === 0) {
      toast.info('No changes to save');
      return;
    }

    const invalid = this.rows.some((r) => !r.isDeleted && (r.isNew || r.isDirty) && !r.name.trim());
    if (invalid) {
      toast.error('All variables need a name before saving');
      return;
    }

    this.saving = true;
    try {
      const result = await applyEnvironmentChanges(changes);
      if (result.success) {
        toast.success(result.message);
        await refreshProcessEnvironment();
        await this.load();
      } else {
        toast.warning(result.message);
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to save environment variables');
    } finally {
      this.saving = false;
    }
  };
}
