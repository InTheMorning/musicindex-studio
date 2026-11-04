// The script of the window on Linux (ADR 0124 items 23 to 25). WebKitGTK gives a page no
// file from a drop, and it does not connect Ctrl+Z to the undo command. The
// UI stays the same as in a browser: it sees a usual file drag.
(() => {
  const FILE_DRAG = (event) => {
    const types = Array.from((event.dataTransfer && event.dataTransfer.types) || []);
    return event.isTrusted && types.includes('text/uri-list') && !types.includes('Files');
  };
  const fileEvent = (type, files) => {
    const event = new Event(type, { bubbles: true, cancelable: true, composed: true });
    Object.defineProperty(event, 'dataTransfer', {
      value: { types: ['Files'], files, dropEffect: 'copy' },
    });
    return event;
  };
  // A target cancels the `dragover` event. The page refuses each other file
  // drag with the effect `none`.
  const accepts = (target) => {
    const event = fileEvent('dragover', []);
    target.dispatchEvent(event);
    return event.defaultPrevented && event.dataTransfer.dropEffect !== 'none';
  };
  for (const type of ['dragenter', 'dragover']) {
    window.addEventListener(type, (event) => {
      if (!FILE_DRAG(event)) return;
      event.stopImmediatePropagation();
      // The cancel stops the insert of the file address in a text box.
      event.preventDefault();
      event.dataTransfer.dropEffect = accepts(event.target) ? 'copy' : 'none';
    }, true);
  }
  // The app calls this function with the drop that it got.
  window.__studioDrop = async ({ x, y, files, token }) => {
    const target = document.elementFromPoint(x, y);
    if (!target || !accepts(target)) return;
    // A target for one file refuses many files, so it needs no bytes.
    const bytes = files.length === 1 || target.closest('[data-drop-many]');
    const given = [];
    for (const { name, id } of files) {
      let parts = [];
      if (bytes && id) {
        const answer = await fetch(`/api/v1/dropped-files/${id}`, {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (!answer.ok) return;
        parts = [await answer.blob()];
      }
      given.push(new File(parts, name));
    }
    target.dispatchEvent(fileEvent('drop', given));
  };
  window.addEventListener('keydown', (event) => {
    if (event.defaultPrevented || !event.ctrlKey || event.altKey || event.metaKey) return;
    const key = event.key.toLowerCase();
    let command = null;
    if (key === 'z') command = event.shiftKey ? 'redo' : 'undo';
    if (key === 'y' && !event.shiftKey) command = 'redo';
    if (!command) return;
    const target = event.target;
    if (target.matches && (target.matches('input, textarea') || target.isContentEditable)) {
      event.preventDefault();
      document.execCommand(command);
    }
  });
})();
