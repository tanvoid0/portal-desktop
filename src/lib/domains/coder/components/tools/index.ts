import type { Component } from 'svelte';
import type { ToolKind } from '../../utils/toolKinds.js';
import ToolCommand from './ToolCommand.svelte';
import ToolEditDiff from './ToolEditDiff.svelte';
import ToolFallback from './ToolFallback.svelte';
import ToolFileContent from './ToolFileContent.svelte';
import ToolFileList from './ToolFileList.svelte';
import ToolSubagent from './ToolSubagent.svelte';
import ToolTodoList from './ToolTodoList.svelte';
import ToolWeb from './ToolWeb.svelte';
import type { ToolBodyProps } from './types.js';

/** Body renderer per tool kind; `ToolCallCard` owns only the row + collapse. */
export const toolBodies: Record<ToolKind, Component<ToolBodyProps>> = {
  read: ToolFileContent,
  write: ToolFileContent,
  edit: ToolEditDiff,
  'multi-edit': ToolEditDiff,
  command: ToolCommand,
  search: ToolFileList,
  list: ToolFileList,
  todo: ToolTodoList,
  subagent: ToolSubagent,
  web: ToolWeb,
  fallback: ToolFallback,
};
