// Set the theme before the first paint: the stored choice, or the system
// setting. The same rule as musicindex.org.
(() => {
  let stored = null;
  try { stored = localStorage.getItem('studio-theme'); } catch (_) {}
  const system = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  document.documentElement.dataset.theme = stored === 'light' || stored === 'dark' ? stored : system;
})();
