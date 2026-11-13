import { invokeClient } from '$lib/utils/invokeClient';
import type { ColumnInfo, DbSource, QueryResult, TableInfo } from '../generated';

// Rust types are already camelCase (ts-rs), so no normalize step.
// Tauri maps camelCase arg keys to the snake_case Rust params.
export const listSources = () => invokeClient.post<DbSource[]>('db_list_sources');
export const openSqlite = (path: string) => invokeClient.post<DbSource>('db_open_sqlite', { path });
export const listTables = (sourceId: string) =>
  invokeClient.post<TableInfo[]>('db_list_tables', { sourceId });
export const describeTable = (sourceId: string, table: string) =>
  invokeClient.post<ColumnInfo[]>('db_describe_table', { sourceId, table });
export const fetchRows = (sourceId: string, table: string, limit: number, offset: number) =>
  invokeClient.post<QueryResult>('db_fetch_rows', { sourceId, table, limit, offset });
export const runQuery = (sourceId: string, sql: string) =>
  invokeClient.post<QueryResult>('db_run_query', { sourceId, sql });
