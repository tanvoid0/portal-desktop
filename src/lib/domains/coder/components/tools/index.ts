import type { Component } from 'svelte';
import FilePlus from '~icons/fa6-solid/file-circle-plus';
import FileLines from '~icons/fa6-solid/file-lines';
import FolderOpen from '~icons/fa6-solid/folder-open';
import Globe from '~icons/fa6-solid/globe';
import ListCheck from '~icons/fa6-solid/list-check';
import Search from '~icons/fa6-solid/magnifying-glass';
import Pen from '~icons/fa6-solid/pen-to-square';
import Robot from '~icons/fa6-solid/robot';
import Terminal from '~icons/fa6-solid/terminal';
import Wrench from '~icons/fa6-solid/wrench';
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

/** Row icon per tool kind. */
export const toolIcons: Record<ToolKind, typeof Wrench> = {
  read: FileLines,
  write: FilePlus,
  edit: Pen,
  'multi-edit': Pen,
  command: Terminal,
  search: Search,
  list: FolderOpen,
  todo: ListCheck,
  subagent: Robot,
  web: Globe,
  fallback: Wrench,
};

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
