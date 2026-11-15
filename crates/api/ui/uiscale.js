// The UI scale of Settings (ADR 0134, ADR 0042). This module has no Preact.
// `theme.js` sets the stored level before the first paint, and `styles.css`
// gives each level its factor.

export const SCALE_KEY = 'studio-ui-scale';
export const DEFAULT_SCALE = 'medium';

export const UI_SCALES = [
  ['x-small', 'Extra small'],
  ['small', 'Small'],
  ['medium', 'Medium'],
  ['large', 'Large'],
  ['x-large', 'Extra large'],
];

// A level that is not known gives the default.
export function normalizeScale(level) {
  return UI_SCALES.some(([id]) => id === level) ? level : DEFAULT_SCALE;
}

// The zoom that the engine gives a unit of the window: the zoom of the page,
// or 1. `unit` is the height of a box of 100dvh, and `full` is the height of
// a box that fills the window. A box with no size gives the zoom of the page.
export function windowUnitZoom(unit, full, zoom) {
  if (!(unit > 0) || !(full > 0)) return zoom;
  const ratio = unit / full;
  return Math.abs(ratio - 1) < Math.abs(ratio - zoom) ? 1 : zoom;
}

// Measure the engine after each change of the level, and at the start.
// `styles.css` divides dvh and vw by --window-unit-zoom.
export function measureWindowUnits(root = document.documentElement) {
  const zoom = Number(getComputedStyle(root).getPropertyValue('--ui-zoom')) || 1;
  root.style.removeProperty('--window-unit-zoom');
  if (zoom === 1) return;
  const box = (style) => {
    const el = document.createElement('div');
    el.style.cssText = `position:fixed;top:0;left:0;width:0;visibility:hidden;pointer-events:none;${style}`;
    root.append(el);
    const height = el.getBoundingClientRect().height;
    el.remove();
    return height;
  };
  if (windowUnitZoom(box('height:100dvh'), box('bottom:0'), zoom) === 1) root.style.setProperty('--window-unit-zoom', '1');
}
