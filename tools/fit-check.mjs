// The fit check of the Music Index web apps (musicindex ADR 0065 R-FIT-3 and R-FIT-5).
//
// The website and Studio each keep a copy of this file from one musicindex-lib
// commit. A check in each app compares the bytes of the copy with that commit.
// The module runs in a browser with no build step and no dependency.
//
// edgeCuts, overflows, overlaps and wordBreaks run in the page. Each returns a list of
// findings and does not exit. The walk script of the app exits with a status
// that is not zero when a list has a finding (R-FIT-5 item 2).
//
// Each of the four helpers uses no name from outside its own body. Thus a walk
// script can send the source of a helper to the page through the DevTools protocol:
//
//   import { edgeCuts } from './fit-check.mjs';
//   Runtime.evaluate({ expression: `(${edgeCuts})(${JSON.stringify(options)})`, returnByValue: true })
//
// A page can also import the module and call the helpers directly.
//
// Each finding is an object { fault, text, detail }. The fault is 'edge-cut',
// 'overflow', 'overlap' or 'word-break'. An overflow finding also has `box`: 'window'
// or 'control'. fitMessage runs in the walk script and gives the
// failure message of R-FIT-3 item 4.

// The attribute for a text that the app must break inside a word, for example a
// URL or a GUID (R-FIT-3 item 3). wordBreaks does not count a break in an element
// with this attribute or in its descendants. wordBreaks has the same name in its
// default options, because a helper cannot use this constant in the page.
export const ALLOW_BREAK_ATTRIBUTE = 'data-fit-allow-break';

// Gives each text and each control that is nearer than `inset` px to the left
// edge or the right edge of its group. A negative distance is a cut. The helper
// first opens each disclosure in a group. The content of a dialog in a group is
// not content of that group, because the dialog shows in the top layer.
// An ancestor that clips its content, such as a summary with an ellipsis, also
// clips the box that the helper measures.
// The defaults are those of Studio: a group is `.group`, and a row has 12 px of
// inner space (Studio ADR 0029 items 11 and 12).
export function edgeCuts({ groups = '.group', inset = 12, openDetails = true } = {}) {
  const shown = (e) => (e.checkVisibility ? e.checkVisibility() : !!e.offsetParent);
  const found = [];
  if (openDetails) {
    for (const group of document.querySelectorAll(groups)) {
      group.querySelectorAll('details').forEach((d) => { d.open = true; });
    }
  }
  for (const group of document.querySelectorAll(groups)) {
    if (!shown(group)) continue;
    const g = group.getBoundingClientRect();
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
    // Gives true and adds one finding when a box of the node is too near an edge.
    const test = (r, text) => {
      if (r.width === 0 || r.height === 0) return false;
      const left = Math.round(r.left - g.left);
      const right = Math.round(g.right - r.right);
      if (r.left - g.left >= inset && g.right - r.right >= inset) return false;
      found.push({ fault: 'edge-cut', text, detail: `${left} px from the left edge, ${right} px from the right edge` });
      return true;
    };
    const inDialog = (node) => { const d = node.closest('dialog'); return !!d && group.contains(d); };
    const walker = document.createTreeWalker(group, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      const text = n.textContent.trim();
      if (!text || inDialog(n.parentElement)) continue;
      const range = document.createRange();
      range.selectNodeContents(n);
      for (const r of range.getClientRects()) if (test(visible(n, r), text.slice(0, 60))) break;
    }
    for (const c of group.querySelectorAll('input, select, textarea')) {
      if (inDialog(c) || !shown(c)) continue;
      test(visible(c, c.getBoundingClientRect()), c.outerHTML.slice(0, 70));
    }
  }
  return found;
}

// Gives each visible element that goes further than the right edge of the window.
// An element in a box that clips or scrolls its content is not in the list.
// The list holds only the outermost element of each fault, not its descendants.
// When the page scrolls to the side and no element is in the list, the list holds
// the page.
// The list also holds each visible control with content that is wider than the
// control, when the control does not clip its content. Then the content goes into
// the next control, for example a count in a segment.
export function overflows({ controls = 'button, [role=button], [role=radio], [role=tab], .segment' } = {}) {
  const shown = (e) => (e.checkVisibility ? e.checkVisibility() : !!e.offsetParent);
  const label = (e) => (e.getAttribute('aria-label') || e.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 60) || e.outerHTML.slice(0, 70);
  const width = document.documentElement.clientWidth;
  const found = [];
  const outer = [];
  for (const e of document.querySelectorAll('body *')) {
    if (!shown(e)) continue;
    const r = e.getBoundingClientRect();
    if (r.width === 0 || r.right <= width + 0.5) continue;
    if (outer.some((o) => o.contains(e))) continue;
    let clipped = false;
    for (let a = e.parentElement; a && a !== document.body; a = a.parentElement) {
      if (getComputedStyle(a).overflowX !== 'visible' && a.getBoundingClientRect().right <= width + 0.5) { clipped = true; break; }
    }
    if (clipped) continue;
    outer.push(e);
    found.push({ fault: 'overflow', box: 'window', text: e.outerHTML.slice(0, 70), detail: `${Math.round(r.right - width)} px` });
  }
  const scroll = document.documentElement.scrollWidth;
  if (found.length === 0 && scroll > width) {
    found.push({ fault: 'overflow', box: 'window', text: 'the page', detail: `${scroll - width} px` });
  }
  for (const c of document.querySelectorAll(controls)) {
    if (!shown(c) || getComputedStyle(c).overflowX !== 'visible') continue;
    if (c.scrollWidth <= c.clientWidth + 1) continue;
    found.push({ fault: 'overflow', box: 'control', text: label(c), detail: `content ${c.scrollWidth} px in a box of ${c.clientWidth} px` });
  }
  return found;
}

// Gives each pair of visible controls with boxes that intersect by more than 1 px
// in the two directions. A pair in which one control contains the other is not in
// the list. A disabled control or a control with a low opacity is in the list.
// A control under a layer is not in the list: the layer is an open modal dialog, or
// an open dialog or popover that covers the control and does not contain it.
export function overlaps({ controls = 'button, a, input, select, textarea, [role=button], [role=radio], [role=tab]' } = {}) {
  const shown = (e) => (e.checkVisibility ? e.checkVisibility() : !!e.offsetParent);
  const label = (e) => (e.getAttribute('aria-label') || e.textContent || e.getAttribute('placeholder') || '').trim().replace(/\s+/g, ' ').slice(0, 40) || e.outerHTML.slice(0, 70);
  const cross = (a, b) => ({
    width: Math.min(a.right, b.right) - Math.max(a.left, b.left),
    height: Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top),
  });
  const layers = [...document.querySelectorAll('dialog, [popover]')].filter((l) => {
    if (l.tagName === 'DIALOG' && l.open) return true;
    try { return l.matches(':popover-open'); } catch (_) { return false; }
  });
  const under = (e, r) => layers.some((l) => {
    if (l.contains(e)) return false;
    if (l.matches(':modal')) return true;
    const c = cross(l.getBoundingClientRect(), r);
    return c.width > 0 && c.height > 0;
  });
  const items = [];
  for (const e of document.querySelectorAll(controls)) {
    if (!shown(e)) continue;
    const r = e.getBoundingClientRect();
    if (r.width === 0 || r.height === 0 || under(e, r)) continue;
    items.push({ e, r });
  }
  const found = [];
  for (let i = 0; i < items.length; i++) {
    for (let j = i + 1; j < items.length; j++) {
      const a = items[i];
      const b = items[j];
      if (a.e.contains(b.e) || b.e.contains(a.e)) continue;
      const c = cross(a.r, b.r);
      if (c.width <= 1 || c.height <= 1) continue;
      found.push({ fault: 'overlap', text: `"${label(a.e)}" and "${label(b.e)}"`, detail: `${Math.round(c.width)} x ${Math.round(c.height)} px` });
    }
  }
  return found;
}

// Gives each word that a line break divides: a line ends between two letters or
// digits of one word. A break at a space or at a hyphen is not in the list.
// A text in an element that matches `allow` is not in the list (R-FIT-3 item 3).
export function wordBreaks({ allow = '[data-fit-allow-break]' } = {}) {
  const shown = (e) => (e.checkVisibility ? e.checkVisibility() : !!e.offsetParent);
  const word = /[\p{L}\p{M}\p{N}]{2,}/gu;
  const found = [];
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const parent = n.parentElement;
    if (!parent || !n.textContent.trim() || !shown(parent) || parent.closest(allow)) continue;
    for (const m of n.textContent.matchAll(word)) {
      const range = document.createRange();
      range.setStart(n, m.index);
      range.setEnd(n, m.index + m[0].length);
      const rects = [...range.getClientRects()].filter((r) => r.width > 0 && r.height > 0);
      if (rects.length < 2) continue;
      const tops = rects.map((r) => r.top);
      const lowest = Math.min(...rects.map((r) => r.height));
      if (Math.max(...tops) - Math.min(...tops) < lowest / 2) continue;
      found.push({ fault: 'word-break', text: m[0].slice(0, 60), detail: n.textContent.trim().slice(0, 70) });
    }
  }
  return found;
}

// Gives the failure message of R-FIT-3 item 4 for one finding at one window width.
export function fitMessage(finding, width) {
  const head = `musicindex ADR 0065 R-FIT-3: at ${width} px`;
  switch (finding.fault) {
    case 'word-break':
      return `${head} the text "${finding.text}" breaks inside a word (in "${finding.detail}").\n`
        + 'Give the text more width, make the text shorter, or let the row wrap between the controls (ADR 0020 R-FIT-1 item 2). '
        + `If the text must break inside a word, such as a URL or a GUID, give its element the attribute ${ALLOW_BREAK_ATTRIBUTE}.`;
    case 'edge-cut':
      return `${head} "${finding.text}" is too near an edge of its group (${finding.detail}).\n`
        + 'Give the row its inner space, make the content narrower, or let the row wrap (ADR 0020 R-FIT-1 item 2).';
    case 'overflow':
      if (finding.box === 'control') {
        return `${head} the content of the control "${finding.text}" is wider than the control (${finding.detail}).\n`
          + 'Give the control more width, make the text shorter, or let the row wrap between the controls (ADR 0020 R-FIT-1 item 2).';
      }
      return `${head} ${finding.text} goes ${finding.detail} past the right edge of the window.\n`
        + 'Make the element narrower, let it wrap, or put it in a scroll area.';
    case 'overlap':
      return `${head} the controls ${finding.text} overlap (${finding.detail}).\n`
        + 'Give each control its own space: let the row wrap, make a control narrower, or move a control to a different row at this width (ADR 0020 R-FIT-1 item 2).';
    default:
      return `${head} the fit check found the fault "${finding.fault}": ${finding.text}.`;
  }
}
