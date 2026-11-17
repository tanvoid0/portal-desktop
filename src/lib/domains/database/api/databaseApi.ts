import { invokeClient } from '$lib/utils/invokeClient';
import type {
  ColumnInfo,
  DbSource,
  EditPreview,
  MongoPage,
  QueryResult,
  TableInfo,
} from '../generated';

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

// Editing (P2) — read-only is the default, toggled per source.
export const setEditable = (sourceId: string, editable: boolean) =>
  invokeClient.post<void>('db_set_editable', { sourceId, editable });

export const previewUpdate = (
  sourceId: string,
  table: string,
  pkColumn: string,
  pkValue: unknown,
  column: string,
  newValue: unknown
) =>
  invokeClient.post<EditPreview>('db_preview_update', {
    sourceId,
    table,
    pkColumn,
    pkValue,
    column,
    newValue,
  });
export const applyUpdate = (
  sourceId: string,
  table: string,
  pkColumn: string,
  pkValue: unknown,
  column: string,
  newValue: unknown
) =>
  invokeClient.post<QueryResult>('db_apply_update', {
    sourceId,
    table,
    pkColumn,
    pkValue,
    column,
    newValue,
  });

export const previewInsert = (sourceId: string, table: string, values: [string, unknown][]) =>
  invokeClient.post<EditPreview>('db_preview_insert', { sourceId, table, values });
export const applyInsert = (sourceId: string, table: string, values: [string, unknown][]) =>
  invokeClient.post<QueryResult>('db_apply_insert', { sourceId, table, values });

export const previewDelete = (
  sourceId: string,
  table: string,
  pkColumn: string,
  pkValue: unknown
) => invokeClient.post<EditPreview>('db_preview_delete', { sourceId, table, pkColumn, pkValue });
export const applyDelete = (sourceId: string, table: string, pkColumn: string, pkValue: unknown) =>
  invokeClient.post<QueryResult>('db_apply_delete', { sourceId, table, pkColumn, pkValue });

// MongoDB (P3) — documents travel as extended-JSON text (see mongo.rs); `id` is
// the `_id` text a MongoDoc carries, sent back verbatim.
export const mongoListDatabases = (sourceId: string) =>
  invokeClient.post<string[]>('db_mongo_list_databases', { sourceId });
export const mongoListCollections = (sourceId: string, database: string) =>
  invokeClient.post<string[]>('db_mongo_list_collections', { sourceId, database });
export const mongoFind = (
  sourceId: string,
  database: string,
  collection: string,
  filter: string,
  limit: number,
  skip: number
) =>
  invokeClient.post<MongoPage>('db_mongo_find', {
    sourceId,
    database,
    collection,
    filter,
    limit,
    skip,
  });
export const mongoInsert = (sourceId: string, database: string, collection: string, doc: string) =>
  invokeClient.post<string>('db_mongo_insert', { sourceId, database, collection, doc });
export const mongoReplace = (
  sourceId: string,
  database: string,
  collection: string,
  id: string,
  doc: string
) => invokeClient.post<void>('db_mongo_replace', { sourceId, database, collection, id, doc });
export const mongoDelete = (sourceId: string, database: string, collection: string, id: string) =>
  invokeClient.post<void>('db_mongo_delete', { sourceId, database, collection, id });
