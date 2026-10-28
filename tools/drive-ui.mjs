// Drive the Studio UI in headless Chromium through the DevTools protocol.
// It needs Node.js 22 or later and `chromium` on the PATH. `tools/node`
// gives Node.js when the PATH has none. It is a check tool for agents and
// developers. Studio itself does not use it.
//
// Usage: tools/node tools/drive-ui.mjs <studio-url-with-token> <out-dir> '<steps-json>'
//
// Each step is an object: {"wait": ms, "js": "expression", "shot": "name"}.
// A step {"size": [width, height], "touch": true} sets the window size and
// the touch emulation (a coarse pointer). Both keys are optional.
// A step {"webauthn": true} adds a virtual authenticator to the page. It is a
// CTAP2 platform authenticator with a resident key, user verification and the
// PRF extension, and it confirms each request by itself. It lasts until the
// run ends, so one run must hold a whole passkey flow. A step
// {"webauthn": "replace"} removes the earlier virtual authenticator and adds
// a new one with no credential. A page cannot add a second passkey to one
// authenticator, so a flow with two passkeys needs this step. The page needs
// a secure context, such as `http://localhost:PORT`. An IP address is not a
// relying party ID.
// A step {"file": [selector, path]} sets the file of a file input (an absolute
// path on this computer) through `DOM.setFileInputFiles`, and the input then
// fires its `change` event. Use a step for the click that opens the sheet
// first, because the input must be in the page.
// A step {"press": selector} clicks the center of the first element that the
// selector finds, through `Input.dispatchMouseEvent`. The page gets a real
// user gesture, which an audio element needs before it plays.
// A step {"manifest": true} sends `Page.getAppManifest`. It prints the
// manifest `url`, the `errors` and the parsed `name` and `display`. An empty
// `errors` list means that Chromium found no manifest error.
// The page has these helpers: $set(selector, value), $click(text),
// $text() and $small() (the controls lower than 44 px, except a control in
// a label of 44 px or more, and a control that is hidden on purpose).
// $edges() opens each disclosure in a group, and gives each text and each
// control that is nearer than 12 px to the left edge or the right edge of
// its group (ADR 0029 items 11 and 12: a group holds rows, and a row has
// inner space). The content of a dialog in a group is not content of that
// group. An empty list means that no content touches an edge.
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
  window.$edges = () => {
    document.querySelectorAll('.group details').forEach((d) => { d.open = true; });
    const near = [];
    for (const group of document.querySelectorAll('.group')) {
      if (!(group.checkVisibility ? group.checkVisibility() : group.offsetParent)) continue;
      const g = group.getBoundingClientRect();
      // The visible part of the box r: an ancestor that hides its overflow, such as
      // a summary with an ellipsis, cuts the box.
      const visible = (node, r) => {
        let left = r.left;
        let right = r.right;
        for (let e = node.parentElement; e && e !== group; e = e.parentElement) {
          if (getComputedStyle(e).overflowX === 'visible') continue;
          const b = e.getBoundingClientRect();
          left = Math.max(left, b.left);
          right = Math.min(right, b.right);
        }
        return { left, right, width: Math.max(0, right - left), height: r.height };
      };
      const test = (r, what) => {
        if (r.width === 0 || r.height === 0) return;
        if (r.left - g.left < 12 || g.right - r.right < 12) near.push(what.slice(0, 60) + ' @ ' + Math.round(r.left - g.left) + '/' + Math.round(g.right - r.right));
      };
      // A dialog in the group shows in the top layer, not in the group. Its
      // own groups get their own test.
      const inDialog = (node) => { const d = node.closest('dialog'); return !!d && group.contains(d); };
      const walker = document.createTreeWalker(group, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent.trim() || inDialog(n.parentElement)) continue;
        const range = document.createRange();
        range.selectNodeContents(n);
        for (const r of range.getClientRects()) test(visible(n, r), n.textContent.trim());
      }
      for (const c of group.querySelectorAll('input, select, textarea')) {
        if (inDialog(c)) continue;
        if (c.checkVisibility ? c.checkVisibility() : c.offsetParent) test(visible(c, c.getBoundingClientRect()), c.outerHTML);
      }
    }
    return near;
  };
  window.$small = () => [...document.querySelectorAll('button, input, select, a')].filter((e) => !e.classList.contains('visually-hidden') && (e.checkVisibility ? e.checkVisibility() : e.offsetParent) && e.getBoundingClientRect().height < 44 && !(e.closest('label') && e.closest('label').getBoundingClientRect().height >= 44)).map((e) => e.outerHTML.slice(0, 70));
`;
let authenticatorId;
const steps = JSON.parse(process.argv[4] || '[]');
await send('Page.navigate', { url });
await sleep(1500);
await evaluate(helpers);
for (const step of steps) {
  if (step.wait) await sleep(step.wait);
  if (step.size) await send('Emulation.setDeviceMetricsOverride', { width: step.size[0], height: step.size[1], deviceScaleFactor: 1, mobile: step.touch === true });
  if (step.touch !== undefined) await send('Emulation.setTouchEmulationEnabled', { enabled: step.touch });
  if (step.webauthn) {
    await send('WebAuthn.enable');
    if (step.webauthn === 'replace' && authenticatorId) {
      await send('WebAuthn.removeVirtualAuthenticator', { authenticatorId });
    }
    const added = await send('WebAuthn.addVirtualAuthenticator', {
      options: {
        protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true,
        isUserVerified: true, hasPrf: true, automaticPresenceSimulation: true,
      },
    });
    authenticatorId = added.result?.authenticatorId;
    console.log('webauthn =>', added.error ? 'ERROR ' + JSON.stringify(added.error) : 'virtual authenticator ' + authenticatorId);
  }
  if (step.file) {
    const [selector, path] = step.file;
    const { root } = (await send('DOM.getDocument', { depth: 0 })).result;
    const found = await send('DOM.querySelector', { nodeId: root.nodeId, selector });
    if (!found.result?.nodeId) throw new Error('no input ' + selector);
    await send('DOM.setFileInputFiles', { nodeId: found.result.nodeId, files: [path] });
    console.log('file =>', path);
  }
  if (step.press) {
    const box = await evaluate(`(() => { const e = document.querySelector(${JSON.stringify(step.press)}); if (!e) return null;
      e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
    if (!box) throw new Error('no element ' + step.press);
    const [x, y] = box;
    await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
    await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
    console.log('press =>', step.press);
  }
  if (step.manifest) {
    const m = (await send('Page.getAppManifest')).result;
    let parsed = {};
    try { parsed = JSON.parse(m.data || '{}'); } catch (_) {}
    console.log('manifest =>', JSON.stringify({ url: m.url, errors: m.errors, name: parsed.name, display: parsed.display }));
  }
  if (step.js) { await evaluate(helpers); console.log(step.js.slice(0, 60), '=>', JSON.stringify(await evaluate(step.js))?.slice(0, 300)); }
  if (step.shot) await shot(step.shot);
}
console.log(logs.length ? logs.join('\n') : 'no console errors');
ws.close();
chrome.kill();
process.exit(0);
