import type { TableInfo } from '../generated';

// Portal's own DB has 280+ tables: past this a SQL source starts collapsed and gets a filter,
// so it can't bury every source listed below it.
const COLLAPSE_OVER = 30;

/**
 * What a SQL source's table list shows. `toggled` is the user's own expand/collapse choice
 * (undefined = never touched); a live filter beats it, since its matches are the point.
 */
export function tableList(tables: TableInfo[], filter: string, toggled?: boolean) {
  const big = tables.length > COLLAPSE_OVER;
  // A small list has no filter box, so a stale query must not keep narrowing it.
  const query = big ? filter.trim().toLowerCase() : '';
  return {
    big,
    query,
    visible: query ? tables.filter((t) => t.name.toLowerCase().includes(query)) : tables,
    expanded: query !== '' || (toggled ?? !big),
  };
}
