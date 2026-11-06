// Repro: Tasks header toolbar squeezes the description and, at the 1024px minimum, pushes "New Task" off-screen.
import { open } from './app.mjs';
const app = await open('tasks');
const p = app.page;
await p.setViewportSize({ width: 1280, height: 800 }).catch(() => {});
await p.waitForTimeout(800);
const desc = await p.getByText('Organize and track your work efficiently').boundingBox();
console.log('1280: description box w x h =', Math.round(desc.width), 'x', Math.round(desc.height), '(expected one line, h ~ 24)');
await p.setViewportSize({ width: 1024, height: 600 }).catch(() => {});
await p.waitForTimeout(800);
const nt = await p.getByRole('button', { name: 'New Task' }).first().boundingBox();
console.log('1024: New Task right edge =', Math.round(nt.x + nt.width), 'viewport 1024', nt.x + nt.width > 1024 ? '-> OFF-SCREEN' : 'ok');
await app.shot('QA-032-before-tasks-header-min-window');
await p.setViewportSize({ width: 1280, height: 800 }).catch(() => {});
await app.done();
