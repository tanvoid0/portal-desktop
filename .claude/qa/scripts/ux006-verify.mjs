// UX-006 verification: /logs line capacity, chrome height, controls, filters, a11y names.
import { open } from './i2.mjs';

const app = await open('logs', { width: 1280, height: 800 });
const page = app.page;

// Generate log traffic so the list overflows.
for (const r of ['/', '/projects', '/tasks', '/settings', '/sdk', '/documents', '/terminal']) {
  await page.goto('http://localhost:1421' + r);
  await page.waitForTimeout(250);
}
await app.go('logs');
await page.waitForTimeout(800);

const measure = async (label) => {
  const vp = page.viewportSize();
  const m = await page.evaluate(() => {
    const rows = [...document.querySelectorAll('[data-log-row], .font-mono')].filter(
      (e) => e.children.length >= 2 && e.textContent.match(/\d{4}-\d{2}-\d{2}/)
    );
    const vh = window.innerHeight;
    const visible = rows.filter((r) => {
      const b = r.getBoundingClientRect();
      return b.top >= 0 && b.bottom <= vh + 1 && b.height > 0;
    });
    const h = rows.length ? Math.round(rows[0].getBoundingClientRect().height) : null;
    // chrome = everything above the first row
    const firstTop = rows.length ? Math.round(rows[0].getBoundingClientRect().top) : null;
    // scroll container: nearest ancestor of a row that scrolls
    let sc = rows[0];
    while (sc && sc.scrollHeight <= sc.clientHeight + 2) sc = sc.parentElement;
    const scBox = sc ? sc.getBoundingClientRect() : null;
    return {
      totalRows: rows.length,
      visibleRows: visible.length,
      rowHeight: h,
      firstRowTop: firstTop,
      scrollerBottom: scBox ? Math.round(scBox.bottom) : null,
      scrollerTop: scBox ? Math.round(scBox.top) : null,
      vh,
    };
  });
  console.log(label, JSON.stringify(m));
  return m;
};

await measure(`[${1280}x${800}]`);
await app.shot('UX-006-after-logs-1280');

// a11y names on controls
const names = await page.evaluate(() => {
  const out = [];
  for (const el of document.querySelectorAll('button, [role="combobox"], input')) {
    const b = el.getBoundingClientRect();
    if (b.top > 140 || b.width === 0) continue; // chrome row only
    out.push({
      tag: el.tagName,
      text: (el.textContent || '').trim().slice(0, 30),
      label: el.getAttribute('aria-label'),
      title: el.getAttribute('title'),
      type: el.getAttribute('type'),
      ph: el.getAttribute('placeholder'),
    });
  }
  return out;
});
console.log('chrome controls:', JSON.stringify(names, null, 1));

// level glyph accessible name
const glyph = await page.evaluate(() => {
  const row = [...document.querySelectorAll('div')].find(
    (e) => e.textContent.match(/^\s*[IWEDT]\s*\d{4}-\d{2}-\d{2}/) && e.children.length >= 2
  );
  if (!row) return null;
  const first = row.children[0];
  return {
    html: first.outerHTML.slice(0, 220),
    srText: first.textContent,
    aria: first.getAttribute('aria-label') || first.getAttribute('title'),
  };
});
console.log('level glyph:', JSON.stringify(glyph));

// Showing N of M + search filter
const showing = async () =>
  (await page.evaluate(() => (document.body.innerText.match(/Showing \d+ of \d+/) || [null])[0]));
console.log('showing (unfiltered):', await showing());
await page.getByPlaceholder(/Search logs/i).fill('Theme');
await page.waitForTimeout(500);
console.log('showing (search "Theme"):', await showing());
await app.shot('UX-006-after-logs-search');
await page.getByPlaceholder(/Search logs/i).fill('');
await page.waitForTimeout(400);

await app.done();
