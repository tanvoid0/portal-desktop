// Repro: deleting a parent task leaves its subtasks behind as orphans that show up as top-level tasks.
import { open } from './app.mjs';
const app = await open('tasks/create');
const p = app.page;
await p.getByLabel('Title *').fill('QA cascade parent');
await p.getByRole('button', { name: 'Create Task' }).click();
await p.waitForURL(/\/tasks\/\d+$/);
await p.waitForTimeout(1000);
await p.getByRole('button', { name: 'Add Subtask' }).first().click();
await p.getByPlaceholder('Enter subtask title...').fill('QA cascade child');
await p.getByRole('button', { name: 'Add Subtask', exact: true }).last().click();
await p.waitForTimeout(1200);
await p.getByRole('button', { name: 'Delete', exact: true }).click();
await p.locator('[role=alertdialog] button').last().click();
await p.waitForURL(/\/tasks$/);
await p.waitForTimeout(1500);
await p.getByRole('button', { name: 'List', exact: true }).click();
await p.waitForTimeout(600);
const orphan = await p.locator('[aria-label="Select task: QA cascade child"]').count();
console.log('orphan "QA cascade child" listed after parent delete:', orphan, '(expected 0)');
await app.shot('QA-040-before-orphan-subtask');
// cleanup: delete the orphan
if (orphan) {
  await p.locator('[aria-label="Select task: QA cascade child"]').click();
  await p.waitForURL(/\/tasks\/\d+$/);
  await p.waitForTimeout(800);
  await p.getByRole('button', { name: 'Delete', exact: true }).click();
  await p.locator('[role=alertdialog] button').last().click();
  await p.waitForTimeout(1200);
}
await app.done();
