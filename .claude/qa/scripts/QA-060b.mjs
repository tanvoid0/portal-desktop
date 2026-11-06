// QA-060 follow-up: edit a credential (QA credential only).
import { open } from './app.mjs';
const app = await open('credentials'); const p = app.page; await p.waitForTimeout(1500);
await p.getByRole('button',{name:'Edit QA credential'}).click(); await p.waitForTimeout(600);
const d = p.locator('[role=dialog]');
console.log('buttons:', await d.getByRole('button').allInnerTexts());
await d.locator('#name').fill('QA credential edited');
await d.getByRole('button',{name:/Update|Save/}).click(); await p.waitForTimeout(2000);
console.log('dialog still open:', await d.count(), '| error text:', await d.locator('.text-destructive').allInnerTexts(), '| toast:', await p.locator('[role=status],[data-sonner-toast]').allInnerTexts());
await d.screenshot({path:'.claude/qa/shots/QA-060-before-edit-credential-dialog.png'}).catch(()=>{});
await app.done();
