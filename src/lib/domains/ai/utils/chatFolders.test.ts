import { describe, expect, it } from "vitest";
import {
  EMPTY_CHAT_FOLDERS,
  addFolder,
  assignFolder,
  groupByFolder,
  removeFolder,
} from "./chatFolders.js";

const items = [{ id: "a" }, { id: "b" }, { id: "c" }];

describe("chatFolders", () => {
  it("adds folders and ignores duplicates", () => {
    let f = addFolder(EMPTY_CHAT_FOLDERS, "Work");
    f = addFolder(f, "Work");
    f = addFolder(f, "  ");
    expect(f.names).toEqual(["Work"]);
  });

  it("groups assigned items and keeps empty folders", () => {
    let f = addFolder(EMPTY_CHAT_FOLDERS, "Work");
    f = addFolder(f, "Later");
    f = assignFolder(f, "b", "Work");

    expect(groupByFolder(items, f)).toEqual([
      { name: "Work", items: [{ id: "b" }] },
      { name: "Later", items: [] },
      { name: null, items: [{ id: "a" }, { id: "c" }] },
    ]);
  });

  it("returns an item to the ungrouped bucket when unassigned", () => {
    let f = assignFolder(addFolder(EMPTY_CHAT_FOLDERS, "Work"), "a", "Work");
    f = assignFolder(f, "a", null);
    expect(f.assignments).toEqual({});
  });

  it("frees a deleted folder's items instead of dropping them", () => {
    let f = assignFolder(addFolder(EMPTY_CHAT_FOLDERS, "Work"), "a", "Work");
    f = removeFolder(f, "Work");
    expect(f.names).toEqual([]);
    expect(groupByFolder(items, f)).toEqual([{ name: null, items }]);
  });
});
