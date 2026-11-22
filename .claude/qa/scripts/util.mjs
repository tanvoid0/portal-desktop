// dismiss the cold vault dialog (QA-160) if it shows within 3s
export async function dismissVault(p, ms = 3000) {
  const nn = p.getByRole('button', { name: 'Not now' });
  try { await nn.waitFor({ timeout: ms }); await nn.click(); await p.waitForTimeout(400); return true; } catch { return false; }
}
