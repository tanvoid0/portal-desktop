// UX-013 verification: resolve URLs against the REAL route tree using SvelteKit's own
// manifest builder + route matcher. Read-only: builds the manifest in memory, writes nothing.
//
//   node .claude/qa/scripts/UX-013-routes.mjs
//
// Prints, for each URL, which route id the client router would pick. Exits 1 on any mismatch.

import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const kit = (p) => path.join(ROOT, 'node_modules/@sveltejs/kit/src', p);

const { validate_config, process_config } = await import(
  `file:///${kit('core/config/index.js').replace(/\\/g, '/')}`
);
const { default: create_manifest_data } = await import(
  `file:///${kit('core/sync/create_manifest_data/index.js').replace(/\\/g, '/')}`
);
const { exec } = await import(`file:///${kit('utils/routing.js').replace(/\\/g, '/')}`);

// No svelte.config.js in this repo — kit options live inline in vite.config.ts and set
// nothing that affects routing, so plain defaults are faithful here.
const config = process_config(validate_config({}), ROOT);
const manifest = create_manifest_data(config, ROOT);

// Same order the client router walks (create_manifest_data already sorts).
const pages = manifest.routes.filter((r) => r.page);

function resolve(pathname) {
  for (const route of pages) {
    const match = route.pattern.exec(pathname);
    if (!match) continue;
    const params = exec(match, route.params, {});
    if (params === undefined) continue;
    return { id: route.id, params };
  }
  return null;
}

// url -> expected route id (null = expected 404)
const CASES = [
  // 1. workloads overview must NOT be swallowed by [...rest]
  ['/cloud/workloads', '/cloud/workloads'],

  // 1. old workload list paths -> [...rest] redirect
  ['/cloud/workloads/pods', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/services', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/deployments', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/jobs', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/cronjobs', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/daemonsets', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/statefulsets', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/nonsense', '/cloud/workloads/[...rest]'],
  ['/cloud/workloads/pods/extra/deep', '/cloud/workloads/[...rest]'],

  // 1. non-workload lists served directly by [resource]
  ['/cloud/configmaps', '/cloud/[resource]'],
  ['/cloud/secrets', '/cloud/[resource]'],
  ['/cloud/ingress', '/cloud/[resource]'],
  ['/cloud/pods', '/cloud/[resource]'],
  ['/cloud/services', '/cloud/[resource]'],
  ['/cloud/deployments', '/cloud/[resource]'],
  ['/cloud/jobs', '/cloud/[resource]'],
  ['/cloud/cronjobs', '/cloud/[resource]'],
  ['/cloud/daemonsets', '/cloud/[resource]'],
  ['/cloud/statefulsets', '/cloud/[resource]'],
  ['/cloud/junk', '/cloud/[resource]'], // matches, then load() throws 404

  // 2. detail URLs must reach their own page
  ['/cloud/workloads/pods/my-pod', '/cloud/workloads/pods/[pod]'],
  ['/cloud/workloads/services/my-svc', '/cloud/workloads/services/[service]'],
  ['/cloud/workloads/deployments/my-dep', '/cloud/workloads/deployments/[deployment]'],
  ['/cloud/workloads/jobs/my-job', '/cloud/workloads/jobs/[job]'],
  ['/cloud/workloads/cronjobs/my-cj', '/cloud/workloads/cronjobs/[cronjob]'],
  ['/cloud/workloads/daemonsets/my-ds', '/cloud/workloads/daemonsets/[daemonset]'],
  ['/cloud/workloads/statefulsets/my-sts', '/cloud/workloads/statefulsets/[statefulset]'],
  ['/cloud/configmaps/my-cm', '/cloud/configmaps/[configmap]'],
  ['/cloud/secrets/my-secret', '/cloud/secrets/[secret]'],
  ['/cloud/ingress/my-ing', '/cloud/ingress/[ingress]'],
  ['/cloud/configmaps/new', '/cloud/configmaps/new'],
  ['/cloud/secrets/new', '/cloud/secrets/new'],

  // the descriptor slug "new" trap: /cloud/pods/new has no detail route of its own
  ['/cloud/pods/new', null],
];

let failures = 0;
console.log('route ids in manifest order (cloud only):');
for (const r of pages) if (r.id.startsWith('/cloud')) console.log('   ', r.id);
console.log('');

for (const [url, expected] of CASES) {
  const hit = resolve(url);
  const got = hit ? hit.id : null;
  const ok = got === expected;
  if (!ok) failures++;
  console.log(
    `${ok ? 'ok  ' : 'FAIL'} ${url.padEnd(46)} -> ${got ?? '(no match / 404)'}` +
      (ok ? '' : `   EXPECTED ${expected ?? '(404)'}`)
  );
}

// 3. detailBase sanity: every descriptor's detailBase must resolve as a detail route
const descriptorsSrc = path.join(ROOT, 'src/lib/domains/cloud/core/resourceDescriptors.ts');
const text = (await import('node:fs')).readFileSync(descriptorsSrc, 'utf8');
const slugs = [...text.matchAll(/slug: '([^']+)'/g)].map((m) => m[1]);
const bases = [...text.matchAll(/detailBase: '([^']+)'/g)].map((m) => m[1]);
console.log('\ndetailBase check (slug -> detailBase/<name> resolves to):');
slugs.forEach((slug, i) => {
  const base = bases[i];
  const hit = resolve(`${base}/sample-name`);
  const isDetail = hit && /\[[a-z]+\]$/.test(hit.id) && hit.id !== '/cloud/[resource]';
  if (!isDetail) failures++;
  console.log(
    `${isDetail ? 'ok  ' : 'FAIL'} ${slug.padEnd(14)} ${base.padEnd(34)} -> ${hit?.id ?? '(404)'}`
  );
});

// 6. sidebar active state + digit shortcuts, replicating CloudNavigation's own logic
const SHORTCUTS = Object.fromEntries(
  [
    ...(await import('node:fs'))
      .readFileSync(
        path.join(ROOT, 'src/lib/domains/k8s-navigation/utils/keyboardConstants.ts'),
        'utf8'
      )
      .matchAll(/^\s+([A-Z_]+): '(\d)',$/gm),
  ].map((m) => [m[1], m[2]])
);

const navItems = [
  { id: 'overview', route: '/cloud/workloads', prefixes: [] },
  ...slugs.map((slug, i) => ({
    id: slug,
    route: `/cloud/${slug}`,
    prefixes: [`/cloud/${slug}/`, `${bases[i]}/`],
  })),
];
const activeFor = (p) =>
  navItems.filter((i) => p === i.route || i.prefixes.some((x) => p.startsWith(x))).map((i) => i.id);

console.log('\nsidebar active item + digit shortcut per resource:');
for (const [i, slug] of slugs.entries()) {
  const listActive = activeFor(`/cloud/${slug}`);
  const detailActive = activeFor(`${bases[i]}/sample-name`);
  const key = SHORTCUTS[slug.toUpperCase()] ?? '(none)';
  const ok =
    listActive.length === 1 &&
    listActive[0] === slug &&
    detailActive.length === 1 &&
    detailActive[0] === slug;
  if (!ok) failures++;
  console.log(
    `${ok ? 'ok  ' : 'FAIL'} ${slug.padEnd(14)} list->[${listActive}] detail->[${detailActive}] key=${key}`
  );
}
const overviewActive = activeFor('/cloud/workloads');
const overviewOk = overviewActive.length === 1 && overviewActive[0] === 'overview';
if (!overviewOk) failures++;
console.log(`${overviewOk ? 'ok  ' : 'FAIL'} overview       /cloud/workloads -> [${overviewActive}]`);

console.log(`\n${failures === 0 ? 'ALL PASS' : failures + ' FAILURE(S)'}`);
process.exit(failures === 0 ? 0 : 1);
