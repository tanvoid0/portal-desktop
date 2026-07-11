import { describe, expect, it } from "vitest";
import { parseTerminalError } from "./parseTerminalError";

describe("parseTerminalError", () => {
  it("parses PowerShell command-not-found", () => {
    const output = `asdfasd : The term 'asdfasd' is not recognized as the name of a cmdlet, function, script file, or operable program. Check the spelling of the name, or if a path was included, verify that the path is correct and try again.
At line:1 char:1
+ asdfasd
+ ~~~~~~~
    + CategoryInfo          : ObjectNotFound: (asdfasd:String) [], CommandNotFoundException
    + FullyQualifiedErrorId : CommandNotFoundException`;

    const parsed = parseTerminalError(output, "asdfasd");
    expect(parsed?.category).toBe("command-not-found");
    expect(parsed?.title).toContain("asdfasd");
    expect(parsed?.hint).toBeTruthy();
  });

  it("parses bash command-not-found", () => {
    const parsed = parseTerminalError("bash: foobar: command not found", "foobar");
    expect(parsed?.category).toBe("command-not-found");
    expect(parsed?.title).toContain("foobar");
  });

  it("parses zsh command-not-found", () => {
    const parsed = parseTerminalError("zsh: command not found: foobar", "foobar");
    expect(parsed?.category).toBe("command-not-found");
    expect(parsed?.title).toContain("foobar");
  });

  it("parses permission denied", () => {
    const parsed = parseTerminalError(
      "touch: cannot touch 'file.txt': Permission denied",
    );
    expect(parsed?.category).toBe("permission-denied");
    expect(parsed?.hint).toContain("privileges");
  });

  it("parses npm errors", () => {
    const parsed = parseTerminalError("npm ERR! missing script: build");
    expect(parsed?.category).toBe("package-error");
    expect(parsed?.title).toBe("npm error");
  });

  it("parses git fatal errors", () => {
    const parsed = parseTerminalError("fatal: not a git repository");
    expect(parsed?.category).toBe("git-error");
    expect(parsed?.message).toContain("not a git repository");
  });

  it("returns null for empty output", () => {
    expect(parseTerminalError("")).toBeNull();
    expect(parseTerminalError("   ")).toBeNull();
  });

  it("falls back for unknown errors", () => {
    const parsed = parseTerminalError("Something unexpected happened");
    expect(parsed?.category).toBe("unknown");
    expect(parsed?.title).toContain("Something unexpected");
  });
});
