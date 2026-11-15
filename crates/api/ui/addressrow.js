// The shared address row of People and the value editor. A node key shows in
// the short form of musicindex ADR 0076 R-ADDRESS-1, and the hover text and
// Copy give the full key. An LNURL address shows in full.
import { html } from './vendor/htm-preact-standalone.module.js';
import { memberLine } from './people.js';
import { MenuControl } from './menu.js';
import { shortKey } from './nostrkey.js';

export function AddressRow({ member, count, note, tag, mark, pending, action, menuItems, menuLabel, onSelect, Icon, CopyButtonComponent }) {
  const { label, address, account } = member.address ? memberLine(member, true) : { label: 'No address', address: null, account: null };
  return html`<div class=${`link-row static address-row${pending ? ' pending' : ''}`}><span class="text"><span class="label">${label}${!menuItems && tag}</span>
      ${address && html`<span class="summary mono address-text" data-fit-allow-break title=${address}>${label === 'LNURL address' ? address : shortKey(address)}</span>`}
      ${account && html`<span class="summary">Account ${account}</span>`}
      ${note && html`<span class="summary">${note}</span>`}
      ${menuItems && html`<span class="summary address-status">${tag}${count}</span>`}</span>
    ${!menuItems && count && html`<span class="count">${count}</span>`}${!menuItems && mark}
    ${!menuItems && member.address && html`<${CopyButtonComponent} text=${member.address} label="the address" />`}${!menuItems && action}
    ${menuItems && html`<${MenuControl} row=${true} Icon=${Icon} buttonLabel="Actions" accessibleName=${menuLabel}
      menuLabel=${menuLabel} items=${menuItems} onSelect=${onSelect} />`}</div>`;
}
