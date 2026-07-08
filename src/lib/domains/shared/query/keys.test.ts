import { describe, expect, it } from "vitest";
import { queryKeys } from "./keys";

describe("queryKeys", () => {
  it("builds stable dashboard key", () => {
    expect(queryKeys.dashboard.overview).toEqual(["dashboard", "overview"]);
  });

  it("builds project detail keys with id", () => {
    expect(queryKeys.projects.detail(42)).toEqual(["projects", 42]);
    expect(queryKeys.projects.detail("abc")).toEqual(["projects", "abc"]);
  });

  it("builds cloud resource keys", () => {
    expect(queryKeys.cloud.resources("pod", "default")).toEqual([
      "cloud",
      "pod",
      "default",
    ]);
  });
});
