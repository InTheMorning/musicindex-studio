// A menu button and its menu (ADR 0153 and ADR 0210). They use the sort
// menu styles and Escape behavior. With `row`, the button has two forms
// (ADR 0210 item 3): the styles show the ellipsis icon in the narrow layout
// and the text in the wide layout. `Icon` is the icon component of the
// shell, so that this module does not import the shell.
import { html, useState, useRef } from './vendor/htm-preact-standalone.module.js';

export function MenuControl({ buttonLabel, accessibleName, menuLabel, items, onSelect, row = false, Icon }) {
  const [open, setOpen] = useState(false);
  const button = useRef(null);
  const pick = (entry) => {
    setOpen(false);
    onSelect(entry);
    if (button.current) button.current.focus();
  };
  // Escape closes the menu and keeps the focus on its button.
  const onKey = (event) => {
    if (event.key === 'Escape' && open) {
      event.stopPropagation();
      setOpen(false);
      if (button.current) button.current.focus();
    }
  };
  return html`<div class=${row ? 'sort row-menu-wrap' : 'sort'} onKeyDown=${onKey}>
    <button class=${row ? 'button row-menu' : 'button'} type="button" ref=${button} aria-haspopup="menu"
      aria-label=${accessibleName} aria-expanded=${open ? 'true' : 'false'} onClick=${() => setOpen(!open)}>
      ${row ? html`<span class="row-menu-icon"><${Icon} name="ellipsis" /></span><span class="row-menu-text">${buttonLabel} ⌄</span>` : html`${buttonLabel} ⌄`}</button>
    ${open && html`<ul class="sort-menu" role="menu" aria-label=${menuLabel}>
      ${items.map((entry) => html`${entry.separatorBefore && html`<li class="menu-separator" role="separator"></li>`}
        <li><button class=${entry.destructive ? 'menu-destructive' : ''} type="button" role=${entry.role || 'menuitem'}
          aria-checked=${entry.checked === undefined ? undefined : (entry.checked ? 'true' : 'false')}
          onClick=${() => pick(entry)}>${entry.mark !== undefined && html`<span class="sort-mark" aria-hidden="true">${entry.mark}</span>`}${entry.label}</button></li>`)}
    </ul>`}
  </div>`;
}
