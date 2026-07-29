import { describe, expect, it } from "vitest";
import { splitThinking } from "./thinking.js";

describe("splitThinking", () => {
  it("leaves plain answers untouched", () => {
    expect(splitThinking("Hello there")).toEqual({
      reasoning: null,
      answer: "Hello there",
      closed: true,
    });
  });

  it("splits a completed think block", () => {
    expect(splitThinking("<think>weighing options</think>\n\nHi!")).toEqual({
      reasoning: "weighing options",
      answer: "Hi!",
      closed: true,
    });
  });

  it("reports an unterminated block as still open", () => {
    expect(splitThinking("<think>still going")).toEqual({
      reasoning: "still going",
      answer: "",
      closed: false,
    });
  });

  it("only treats a leading block as reasoning", () => {
    const mid = "Answer first <think>late</think>";
    expect(splitThinking(mid).reasoning).toBeNull();
    expect(splitThinking(mid).answer).toBe(mid);
  });
});
