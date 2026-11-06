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
