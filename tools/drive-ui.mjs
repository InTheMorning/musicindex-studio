// Drive the Studio UI in headless Chromium through the DevTools protocol.
// It needs Node.js 22 or later and `chromium` on the PATH. It is a check
// tool for agents and developers. Studio itself does not use it.
//
// Usage: node tools/drive-ui.mjs <studio-url-with-token> <out-dir> '<steps-json>'
//
// Each step is an object: {"wait": ms, "js": "expression", "shot": "name"}.
// The page has these helpers: $set(selector, value), $click(text),
// $text() and $small() (the controls lower than 44 px, except a control in
// a label of 44 px or more, and a control that is hidden on purpose).
// The script prints each result and each console error.
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const [url, out] = process.argv.slice(2);
const port = 9333;
const chrome = spawn('chromium', ['--headless=new', '--no-sandbox', '--disable-gpu', `--remote-debugging-port=${port}`,
  '--window-size=1100,1000', '--force-dark-mode', 'about:blank'], { stdio: 'ignore' });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try { target = await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: 'PUT' })).json(); } catch (_) {}
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r));
let id = 0;
const pending = new Map();
const logs = [];
ws.addEventListener('message', (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
  if (msg.method === 'Runtime.exceptionThrown') logs.push('EXCEPTION ' + JSON.stringify(msg.params.exceptionDetails.exception?.description || msg.params.exceptionDetails.text));
  if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') logs.push('CONSOLE ' + msg.params.args.map((a) => a.value || a.description).join(' '));
  if (msg.method === 'Log.entryAdded' && msg.params.entry.level === 'error') logs.push('LOG ' + msg.params.entry.text);
});
const send = (method, params = {}) => new Promise((r) => { const n = ++id; pending.set(n, r); ws.send(JSON.stringify({ id: n, method, params })); });
await send('Runtime.enable');
await send('Log.enable');
await send('Page.enable');
const evaluate = async (expression) => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (r.result.exceptionDetails) throw new Error(expression + ' -> ' + JSON.stringify(r.result.exceptionDetails));
  return r.result.result.value;
};
const shot = async (name) => {
  const r = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(`${out}/${name}.png`, Buffer.from(r.result.data, 'base64'));
};
// Page helpers.
const helpers = `
  window.$set = (sel, v) => { const e = document.querySelector(sel); e.value = v; e.dispatchEvent(new Event('input', { bubbles: true })); };
  window.$click = (text) => { const e = [...document.querySelectorAll('button, label')].find((b) => b.textContent.trim().startsWith(text)); if (!e) throw new Error('no ' + text); e.click(); };
  window.$text = () => document.body.innerText;
  window.$small = () => [...document.querySelectorAll('button, input, select, a')].filter((e) => !e.classList.contains('visually-hidden') && (e.checkVisibility ? e.checkVisibility() : e.offsetParent) && e.getBoundingClientRect().height < 44 && !(e.closest('label') && e.closest('label').getBoundingClientRect().height >= 44)).map((e) => e.outerHTML.slice(0, 70));
`;
const steps = JSON.parse(process.argv[4] || '[]');
await send('Page.navigate', { url });
await sleep(1500);
await evaluate(helpers);
for (const step of steps) {
  if (step.wait) await sleep(step.wait);
  if (step.js) { await evaluate(helpers); console.log(step.js.slice(0, 60), '=>', JSON.stringify(await evaluate(step.js))?.slice(0, 300)); }
  if (step.shot) await shot(step.shot);
}
console.log(logs.length ? logs.join('\n') : 'no console errors');
ws.close();
chrome.kill();
process.exit(0);
