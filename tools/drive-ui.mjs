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
// A step {"key": "Tab"} sends one key press through `Input.dispatchKeyEvent`.
// Add "shift": true for Shift+Tab. The focus that it moves is keyboard focus,
// so `:focus-visible` applies. The keys are Tab, Enter and Escape.
// A step {"manifest": true} sends `Page.getAppManifest`. It prints the
// manifest `url`, the `errors` and the parsed `name` and `display`. An empty
// `errors` list means that Chromium found no manifest error.
// The page has these helpers: $set(selector, value), $click(text),
// $text() and $small() (the controls lower than 44 px, except a control in
// a label of 44 px or more, and a control that is hidden on purpose).
// $edges(), $overflow(), $overlaps(), $wordBreaks() and $unsized() are edgeCuts,
// overflows, overlaps, wordBreaks and unsizedIcons of tools/fit-check.mjs (musicindex ADR 0065
// R-FIT-5 and R-FIT-7). Each gives
// a list of findings { fault, text, detail }. $edges() opens each disclosure
// in a group.
// A step {"fit": true} runs the five helpers on the current page at 390 px
// and at 761 px (musicindex ADR 0065 R-FIT-2). Then it sets the earlier size
// again. When no step has "fit", the script runs this check after the last
// step. A text that must break inside a word, such as a URL or a GUID, has the
// attribute data-fit-allow-break (R-FIT-3 item 3).
// A step {"save": "name.json"} writes the value of window.__walkChecks to that
// file in the out dir.
// A step {"sheet": "name"} runs the sheet check of musicindex ADR 0230
// R-SHEET-2 on the open sheet, dialog or popover at 390 px and at 761 px. It
// runs the five helpers and sheetFits. The overflow and overlap helpers
// examine only the controls in the sheet. sheetFits finds a sheet that is
// wider than the window less the window margin, content in the 16 px inner
// padding of a bottom sheet, content that goes past an edge of the sheet,
// and a horizontal scroll in the sheet. Then it sets the earlier size again.
// A step {"sheets": "tools/sheets.json"} opens each sheet of that list
// (R-SHEET-2 item 2, ADR 0214). For each entry and each width, it loads the
// start address again, runs the steps of `open` at the earlier size, and
// runs the sheet check at that width with the `name` of the entry. The
// check examines the panels that `panel` selects, and the default is
// "dialog[open]". With no such panel open, the entry fails. An entry
// has a `data` key when it needs a condition that the usual walk data does
// not have. The step {"sheets": path, "data": "collections"} opens only the
// entries with that key. The default is "walk".
// A finding of the sheet check is a fit finding.
// $fieldText() gives each visible text field and select with text smaller
// than 16 px. iOS Safari zooms the page when such a field gets the focus
// (musicindex ADR 0206 R-TYPE-1, Body on Touch). Use it with "touch": true.
// The script prints each result and each console error. It exits with status 1
// when the fit check has a finding, after the last step and its screenshot.
// DRIVE_UI_PORT sets the DevTools port. The default is 9333.
import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { edgeCuts, overflows, overlaps, wordBreaks, unsizedIcons, fitMessage } from './fit-check.mjs';

const [url, out] = process.argv.slice(2);
const port = Number(process.env.DRIVE_UI_PORT || 9333);
// Chromium needs a home folder that it can write. A sandbox can refuse the
// home folder of the user and the crash reporter.
const home = mkdtempSync(join(tmpdir(), 'drive-ui-'));
const chrome = spawn('chromium', ['--headless=new', '--no-sandbox', '--disable-gpu', '--disable-crash-reporter', '--disable-breakpad',
  '--no-proxy-server', `--user-data-dir=${home}/profile`, `--remote-debugging-port=${port}`,
  '--window-size=1100,1000', '--force-dark-mode', 'about:blank'], { stdio: 'ignore', env: { ...process.env, HOME: home } });
process.on('exit', () => {
  chrome.kill();
  // Chromium can write to its profile for a short time after the kill.
  try { rmSync(home, { recursive: true, force: true, maxRetries: 5 }); } catch (_) {}
});
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
  window.$go = (t, root) => { const v = (e) => (e.checkVisibility ? e.checkVisibility() : e.offsetParent); const n = (e) => e.textContent.trim().replace(/\\s+/g, ' ');
    const all = [...(root ? document.querySelector(root) : document).querySelectorAll('button, a, summary, [role=tab], [role=menuitem], label')].filter(v);
    const e = all.find((x) => n(x) === t || x.getAttribute('aria-label') === t) || all.find((x) => n(x).startsWith(t)) || all.find((x) => n(x).includes(t));
    if (!e) throw new Error('no ' + t); e.click(); return n(e).slice(0, 40); };
  window.$tab = (t) => { const s = document.querySelector('select.page-tab-menu'); if (s && (s.checkVisibility ? s.checkVisibility() : s.offsetParent)) { const o = [...s.options].find((x) => x.text.startsWith(t)); s.value = o.value; s.dispatchEvent(new Event('change', { bubbles: true })); return o.text; } return $go(t, '.page-tabs'); };
  window.$actions = (rowText, root = 'main') => { const r = [...document.querySelector(root).querySelectorAll('.address-row, .link-row, .row, .recipient, li, .card')].find((x) => x.textContent.includes(rowText) && x.querySelector('[aria-haspopup], .menu-button, button')); if (!r) throw new Error('no row ' + rowText); (r.querySelector('[aria-haspopup]') || r.querySelector('button')).click(); return rowText; };
  window.$edges = ${edgeCuts};
  window.$overflow = ${overflows};
  window.$overlaps = ${overlaps};
  window.$wordBreaks = ${wordBreaks};
  window.$unsized = ${unsizedIcons};
  window.$fieldText = () => [...document.querySelectorAll('input, select, textarea')].filter((e) => !['checkbox', 'radio', 'range', 'file', 'hidden', 'color'].includes(e.type) && (e.checkVisibility ? e.checkVisibility() : e.offsetParent) && parseFloat(getComputedStyle(e).fontSize) < 16).map((e) => getComputedStyle(e).fontSize + ' ' + e.outerHTML.slice(0, 70));
  window.$small = () => [...document.querySelectorAll('button, input, select, a')].filter((e) => !e.classList.contains('visually-hidden') && (e.checkVisibility ? e.checkVisibility() : e.offsetParent) && e.getBoundingClientRect().height < 44 && !(e.closest('label') && e.closest('label').getBoundingClientRect().height >= 44)).map((e) => e.outerHTML.slice(0, 70));
`;
// The sheet check of musicindex ADR 0230 R-SHEET-1 and R-SHEET-2 item 3. It
// runs in the page and uses no name from outside its body. A sheet is an open
// dialog, an open popover or an open menu of `menu.js`. The window margin is
// that of musicindex ADR 0206 R-SPACE-1 item 2: 16 px up to the narrow width
// of Studio (760 px) and 20 px above it. In a narrow window, a dialog that
// touches the two sides and the bottom edge is a bottom sheet. It passes at
// the window width, and its content must not go into the inner padding.
function sheetFits({ panels = 'dialog[open], .sort-menu', narrow = 760, narrowMargin = 16, wideMargin = 20 } = {}) {
  const shown = (e) => (e.checkVisibility ? e.checkVisibility() : !!e.offsetParent);
  const label = (e) => (e.getAttribute('aria-label') || e.textContent || e.getAttribute('placeholder') || '').trim().replace(/\s+/g, ' ').slice(0, 50) || e.outerHTML.slice(0, 70);
  const width = document.documentElement.clientWidth;
  const margin = width <= narrow ? narrowMargin : wideMargin;
  const found = [];
  const open = [...document.querySelectorAll(panels)].filter(shown);
  for (const p of document.querySelectorAll('[popover]')) {
    try { if (p.matches(':popover-open') && !open.includes(p)) open.push(p); } catch (_) {}
  }
  if (open.length === 0) found.push({ fault: 'no-sheet', text: '', detail: 'no open sheet, dialog or popover' });
  for (const sheet of open) {
    const s = sheet.getBoundingClientRect();
    // A dialog in the sheet shows in the top layer. It is a sheet of its own,
    // not content of this sheet.
    const own = (e) => { const d = e.closest('dialog'); return !d || d === sheet || !sheet.contains(d); };
    const bottom = width <= narrow && sheet.tagName === 'DIALOG' && Math.abs(s.left) < 0.5
      && Math.abs(s.right - width) < 0.5 && s.bottom >= window.innerHeight - 0.5;
    if (bottom) {
      // Each control, picture and text of the bottom sheet stays out of the
      // inner padding. A box with only a background, such as a group or a
      // line, can span the sheet.
      const parts = [];
      for (const e of sheet.querySelectorAll('button, a, input, select, textarea, img, svg, [role=button]')) {
        if (shown(e) && own(e) && !e.parentElement?.closest('svg')) parts.push([e, e.getBoundingClientRect()]);
      }
      const walker = document.createTreeWalker(sheet, NodeFilter.SHOW_TEXT);
      for (let t = walker.nextNode(); t; t = walker.nextNode()) {
        if (!t.data.trim() || !shown(t.parentElement) || !own(t.parentElement) || t.parentElement.closest('button, a, [role=button]')) continue;
        const range = document.createRange();
        range.selectNodeContents(t);
        parts.push([t.parentElement, range.getBoundingClientRect()]);
      }
      const seen = new Set();
      for (const [e, r] of parts) {
        if (r.width === 0 || seen.has(e)) continue;
        const into = Math.max(margin - r.left, r.right - (width - margin));
        if (into <= 0.5) continue;
        seen.add(e);
        found.push({ fault: 'sheet-padding', text: label(e), detail: { into: Math.round(into), margin } });
      }
    } else if (s.left < margin - 0.5 || s.right > width - margin + 0.5) {
      found.push({ fault: 'sheet-width', text: label(sheet.querySelector('h2') || sheet), detail: { width: Math.round(s.width), left: Math.round(s.left), right: Math.round(width - s.right), allowed: width - 2 * margin } });
    }
    // Each element must stay in its nearest box that clips or scrolls, or in
    // the sheet. A box that clips content hides a cut (item 2).
    const outer = [];
    for (const e of sheet.querySelectorAll('*')) {
      if (!shown(e) || !own(e) || outer.some((o) => o.contains(e))) continue;
      const r = e.getBoundingClientRect();
      if (r.width === 0) continue;
      let box = sheet;
      for (let a = e.parentElement; a && a !== sheet; a = a.parentElement) {
        if (getComputedStyle(a).overflowX !== 'visible') { box = a; break; }
      }
      const b = box.getBoundingClientRect();
      if (r.right <= b.right + 0.5 && r.left >= b.left - 0.5) continue;
      outer.push(e);
      found.push({ fault: 'sheet-content', text: label(e), detail: `${Math.round(Math.max(r.right - b.right, b.left - r.left))} px` });
    }
    for (const e of [sheet, ...sheet.querySelectorAll('*')]) {
      if (e.matches('input, textarea, select') || !shown(e) || !own(e)) continue;
      const x = getComputedStyle(e).overflowX;
      if ((x === 'auto' || x === 'scroll') && e.scrollWidth > e.clientWidth + 1) {
        found.push({ fault: 'sheet-scroll', text: label(e), detail: `content ${e.scrollWidth} px in a box of ${e.clientWidth} px` });
      }
      // A box that hides its overflow cuts content that is wider than the
      // box. A text with an ellipsis is a short form, not a cut.
      if ((x === 'hidden' || x === 'clip') && e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).textOverflow !== 'ellipsis') {
        found.push({ fault: 'sheet-clip', text: label(e), detail: `content ${e.scrollWidth} px in a box of ${e.clientWidth} px` });
      }
    }
  }
  return found;
}

// The failure message of musicindex ADR 0230 R-SHEET-2 item 5.
function sheetMessage(finding, name, width) {
  const head = `musicindex ADR 0230 R-SHEET-1: at ${width} px the sheet "${name}"`;
  switch (finding.fault) {
    case 'sheet-width':
      return `${head} is ${finding.detail.width} px wide (${finding.detail.left} px from the left edge and ${finding.detail.right} px from the right edge), and the window allows ${finding.detail.allowed} px.\n`
        + 'Give the sheet the width min(design width, 100vw - 2 × margin), and let each child shrink (min-width: 0).';
    case 'sheet-padding':
      return `musicindex ADR 0230 R-SHEET-1: at ${width} px the content "${finding.text}" of the bottom sheet "${name}" goes ${finding.detail.into} px into the ${finding.detail.margin} px inner padding.\n`
        + 'Give the content the window margin as its padding, and let each child shrink (min-width: 0).';
    case 'sheet-content':
      return `${head} has content "${finding.text}" that goes ${finding.detail} past an edge of the sheet.\n`
        + 'Let each child shrink (min-width: 0), let the row wrap, and give a long key the short form of ADR 0076 R-ADDRESS-1.';
    case 'sheet-scroll':
      return `${head} scrolls horizontally in "${finding.text}" (${finding.detail}).\n`
        + 'A sheet scrolls only vertically (item 5). Let each child shrink (min-width: 0) and let the text wrap.';
    case 'sheet-clip':
      return `${head} cuts content in "${finding.text}" (${finding.detail}).\n`
        + 'Let the text wrap between words, give a long key the short form of ADR 0076 R-ADDRESS-1, and give a URL the attribute data-fit-allow-break.';
    case 'no-sheet':
      return `${head} is not open. Correct the "open" steps of the sheet in tools/sheets.json.`;
    default:
      return `${head}: ${finding.text}.`;
  }
}

// The fit check of musicindex ADR 0065 at the two widths of R-FIT-2.
const fitWidths = [390, 761];
let size = null;
let fitRan = false;
const fitFindings = [];
// With `sheet`, the check also runs sheetFits and names the sheet. The
// overflow and overlap checks then examine only the controls in the sheet: an
// open menu covers the page, and that is not an overlap (R-SHEET-2 item 3).
const inSheet = (controls) => controls.split(', ').flatMap((c) => [`dialog[open] ${c}`, `.sort-menu ${c}`]).join(', ');
const fit = async (sheet = null, widths = fitWidths, panel = 'dialog[open], .sort-menu') => {
  if (sheet === null) fitRan = true;
  const height = size ? size[1] : 1000;
  const scope = sheet === null ? '' : JSON.stringify({
    controls: inSheet('button, [role=button], [role=radio], [role=tab], .segment'),
  });
  const pairs = sheet === null ? '' : JSON.stringify({
    controls: inSheet('button, a, input, select, textarea, [role=button], [role=radio], [role=tab]'),
  });
  for (const width of widths) {
    await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
    await sleep(500);
    const sheetFound = sheet === null ? [] : await evaluate(`(${sheetFits})(${JSON.stringify({ panels: panel })})`);
    // edgeCuts opens each disclosure, so it runs last.
    const found = [
      ...await evaluate(`(${overflows})(${scope})`),
      ...await evaluate(`(${overlaps})(${pairs})`),
      ...await evaluate(`(${wordBreaks})()`),
      ...await evaluate(`(${unsizedIcons})()`),
      ...await evaluate(`(${edgeCuts})()`),
    ];
    for (const f of sheetFound) fitFindings.push(sheetMessage(f, sheet, width));
    for (const f of found) fitFindings.push(sheet === null ? fitMessage(f, width) : `With the sheet "${sheet}" open: ${fitMessage(f, width)}`);
    const count = sheetFound.length + found.length;
    console.log(`${sheet === null ? 'fit' : `sheet "${sheet}"`} at ${width} px =>`, count ? `${count} finding(s)` : 'no finding');
  }
  if (size) await send('Emulation.setDeviceMetricsOverride', { width: size[0], height: size[1], deviceScaleFactor: 1, mobile: size[2] });
  else await send('Emulation.clearDeviceMetricsOverride');
  await sleep(300);
};
let authenticatorId;
const steps = JSON.parse(process.argv[4] || '[]');
// Open each sheet of a list (musicindex ADR 0230 R-SHEET-2). The walk opens
// each entry again for each width from the start address, so that the state
// of one entry does not change the next entry, and a menu that closes when the
// window changes is open at each width.
const walkSheets = async (path, data = 'walk') => {
  const list = JSON.parse(readFileSync(path, 'utf8'));
  const start = size ? [...size] : null;
  for (const entry of list.sheets.filter((e) => (e.data || 'walk') === data)) {
    for (const width of fitWidths) {
      if (start) await runStep({ size: [start[0], start[1]], touch: start[2] });
      await send('Page.navigate', { url: 'about:blank' });
      await sleep(200);
      await send('Page.navigate', { url });
      await sleep(1500);
      await evaluate(helpers);
      try {
        for (const step of entry.open) await runStep(step);
      } catch (e) {
        fitFindings.push(sheetMessage({ fault: 'no-sheet' }, entry.name, start ? start[0] : 0) + `\n${String(e.message).slice(0, 200)}`);
        break;
      }
      await fit(entry.name, [width], entry.panel || 'dialog[open]');
    }
  }
};
await send('Page.navigate', { url });
await sleep(1500);
await evaluate(helpers);
async function runStep(step) {
  if (step.wait) await sleep(step.wait);
  if (step.size) {
    size = [step.size[0], step.size[1], step.touch === true];
    await send('Emulation.setDeviceMetricsOverride', { width: size[0], height: size[1], deviceScaleFactor: 1, mobile: size[2] });
  }
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
  if (step.key) {
    const codes = { Tab: 9, Enter: 13, Escape: 27 };
    if (!codes[step.key]) throw new Error('no key ' + step.key);
    const k = { key: step.key, code: step.key, windowsVirtualKeyCode: codes[step.key], modifiers: step.shift ? 8 : 0 };
    await send('Input.dispatchKeyEvent', { type: 'keyDown', ...k });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', ...k });
    console.log('key =>', step.key);
  }
  if (step.manifest) {
    const m = (await send('Page.getAppManifest')).result;
    let parsed = {};
    try { parsed = JSON.parse(m.data || '{}'); } catch (_) {}
    console.log('manifest =>', JSON.stringify({ url: m.url, errors: m.errors, name: parsed.name, display: parsed.display }));
  }
  if (step.js) { await evaluate(helpers); console.log(step.js.slice(0, 60), '=>', JSON.stringify(await evaluate(step.js))?.slice(0, 300)); }
  if (step.shot) await shot(step.shot);
  if (step.save) writeFileSync(`${out}/${step.save}`, JSON.stringify(await evaluate('window.__walkChecks || {}'), null, 2) + '\n');
  if (step.fit) await fit();
  if (step.sheet) await fit(step.sheet);
  if (step.sheets) await walkSheets(step.sheets, step.data);
}
for (const step of steps) await runStep(step);
if (!fitRan) await fit();
console.log(logs.length ? logs.join('\n') : 'no console errors');
ws.close();
if (fitFindings.length) {
  console.error([...new Set(fitFindings)].join('\n\n'));
  process.exit(1);
}
process.exit(0);
