export function parseGitHubRemote(
  remote?: string | null,
): { owner: string; repo: string } | null {
  if (!remote) return null;

  const trimmed = remote.trim().replace(/\.git$/, "");
  let path: string | undefined;

  if (trimmed.startsWith("https://github.com/")) {
    path = trimmed.slice("https://github.com/".length);
  } else if (trimmed.startsWith("git@github.com:")) {
    path = trimmed.slice("git@github.com:".length);
  } else {
    return null;
  }

  const [owner, repo] = path.split("/");
  if (!owner || !repo) return null;
  return { owner, repo };
}
