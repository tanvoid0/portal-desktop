// Repro: /deployments/new enables "Deploy (build & run)" for whitespace-only name/path and a nonsense port. Never clicks Deploy.
import { open } from './app.mjs';
const app = await open('deployments/new');
const p = app.page;
const deploy = p.getByRole('button', { name: /Deploy \(build/ });
await p.getByPlaceholder('My Deployment').fill('   ');
await p.getByPlaceholder('/path/to/project').fill('   ');
await p.locator('#exposedPort').fill('99999');
console.log('Deploy disabled with blank name, blank path, port 99999:', await deploy.isDisabled(), '(expected true)');
await app.shot('QA-044-before-deploy-enabled-with-invalid-input');
await app.done();
