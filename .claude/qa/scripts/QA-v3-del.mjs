import { open } from './app.mjs';
const app = await open('tasks/create'); const p = app.page;
const d=p.locator('#description'); await d.fill('abcdef'); await d.press('Home'); await d.press('Delete'); await d.press('Delete');
console.log('textarea after 2x Delete from Home:', JSON.stringify(await d.inputValue()), 'dialogs', await p.getByRole('alertdialog').count()+await p.getByRole('dialog').count(), p.url());
// board: selection exists, focus in search box, Delete must edit text not delete tasks
await p.goto(new URL('/tasks?project=all&view=board',p.url()).href); await p.waitForTimeout(1200);
await p.keyboard.press('Control+m'); await p.locator('body').click({position:{x:700,y:60}}); await p.keyboard.press('Control+a');
const s=p.getByPlaceholder('Search tasks...'); await s.fill('xyz'); await s.press('Home'); await s.press('Delete'); await p.waitForTimeout(400);
console.log('search value', await s.inputValue(), 'alertdialog', await p.getByRole('alertdialog').count());
await s.fill(''); await p.keyboard.press('Escape'); await app.done();
