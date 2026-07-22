import { describe, expect, it } from "vitest";
import {
  appendCapped,
  MAX_BLOCK_OUTPUT,
  TRUNCATION_NOTICE,
} from "./commandBlockStore";

describe("appendCapped", () => {
  it("concatenates normally below the cap", () => {
    expect(appendCapped("foo", "bar")).toBe("foobar");
  });

  it("keeps output bounded once the cap is exceeded", () => {
    const line = "x".repeat(99) + "\n";
    let output = "";
    for (let i = 0; i < 5000; i++) {
      output = appendCapped(output, line);
    }
    expect(output.length).toBeLessThanOrEqual(
      MAX_BLOCK_OUTPUT + TRUNCATION_NOTICE.length,
    );
  });

  it("keeps the tail, not the head", () => {
    const output = appendCapped("old".repeat(MAX_BLOCK_OUTPUT), "NEWEST\n");
    expect(output.endsWith("NEWEST\n")).toBe(true);
  });

  it("emits exactly one truncation notice across repeated appends", () => {
    let output = "";
    for (let i = 0; i < 200; i++) {
      output = appendCapped(output, "y".repeat(2000) + "\n");
    }
    expect(output.split(TRUNCATION_NOTICE)).toHaveLength(2);
  });

  it("cuts at a line boundary so partial lines are not shown", () => {
    const body = Array.from({ length: 4000 }, (_, i) => `line-${i}`).join("\n");
    const output = appendCapped(body.repeat(10), "tail\n");
    const firstRealLine = output.slice(TRUNCATION_NOTICE.length).split("\n")[0];
    expect(firstRealLine).toMatch(/^line-\d+$/);
  });

  it("still bounds a single line with no newline to cut on", () => {
    const output = appendCapped("z".repeat(MAX_BLOCK_OUTPUT * 2), "end");
    expect(output.length).toBeLessThanOrEqual(
      MAX_BLOCK_OUTPUT + TRUNCATION_NOTICE.length,
    );
    expect(output.endsWith("end")).toBe(true);
  });
});
