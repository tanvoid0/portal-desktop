import { describe, expect, it, vi } from "vitest";
import { cache } from "./cache";

describe("cache service", () => {
  it("stores and retrieves list data within TTL", () => {
    const items = [{ id: 1, name: "alpha" }];
    cache.setList("projects", items, 60_000);

    expect(cache.getList("projects")).toEqual(items);
  });

  it("returns null for expired entries", () => {
    vi.useFakeTimers();
    cache.setList("tasks", [{ id: "t1" }], 1_000);

    vi.advanceTimersByTime(1_001);
    expect(cache.getList("tasks")).toBeNull();
    vi.useRealTimers();
  });

  it("deletes a specific list key", () => {
    cache.setList("projects", [{ id: 1 }], 60_000);
    cache.delete("projects");
    expect(cache.getList("projects")).toBeNull();
  });
});
