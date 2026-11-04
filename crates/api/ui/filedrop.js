// The drop of a file from the computer (ADR 0125). This module has no Preact
// and no browser state, so the Node tests examine it.

// The attribute of the root element while a file drag is over the window.
export const FILE_DRAG_ATTRIBUTE = 'data-file-drag';
// A drag that leaves the window gives a `dragleave` with no related target.
// This time is for a browser that does not send that event. WebKitGTK sends no
// `dragover` while the pointer does not move, so a short time removes the mark
// during a drag.
export const FILE_DRAG_END_MS = 5000;

export const ACCOUNT_DROP_ONE = 'Drop one account file only';
export const ACCOUNT_DROP_KIND = 'Select a .tar account file';

export function hasFiles(event) {
  return Array.from((event.dataTransfer && event.dataTransfer.types) || []).includes('Files');
}

// The handlers of one drop target. `isBusy()` tells when the target takes no
// file. `stop` keeps the drag from an element around the target, such as a
// field below a sheet.
export function dropHandlers({ isBusy, setOver, onFiles, stop = false }) {
  return {
    onDragOver: (event) => {
      if (!hasFiles(event) || isBusy()) return;
      event.preventDefault();
      if (stop) event.stopPropagation();
      setOver(true);
    },
    onDragLeave: (event) => {
      if (!event.currentTarget.contains(event.relatedTarget)) setOver(false);
    },
    onDrop: (event) => {
      if (!hasFiles(event)) return;
      event.preventDefault();
      if (stop) event.stopPropagation();
      setOver(false);
      if (!isBusy()) onFiles(Array.from(event.dataTransfer.files));
    },
  };
}

// Mark the root element during a file drag, so that each target shows
// (ADR 0125 item 3). A file drag that no target takes changes nothing: the
// browser does not open the file, and a text box does not get its address
// (ADR 0125 item 5).
export function watchFileDrags(win, root) {
  let timer = null;
  const end = () => {
    win.clearTimeout(timer);
    root.removeAttribute(FILE_DRAG_ATTRIBUTE);
  };
  const seen = (event) => {
    if (!hasFiles(event)) return;
    root.setAttribute(FILE_DRAG_ATTRIBUTE, '');
    win.clearTimeout(timer);
    timer = win.setTimeout(end, FILE_DRAG_END_MS);
  };
  const refuse = (event) => {
    if (!hasFiles(event) || event.defaultPrevented) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'none';
  };
  // A target can stop its event, so the mark listens first.
  win.addEventListener('dragenter', seen, true);
  win.addEventListener('dragover', seen, true);
  win.addEventListener('drop', end, true);
  win.addEventListener('dragend', end, true);
  win.addEventListener('dragleave', (event) => { if (!event.relatedTarget) end(); }, true);
  win.addEventListener('dragover', refuse);
  win.addEventListener('drop', refuse);
}

// The result of a drop on "Import an account".
export function accountDropResult(files) {
  if (files.length !== 1) return { error: ACCOUNT_DROP_ONE };
  if (!/\.tar$/i.test(files[0].name)) return { error: ACCOUNT_DROP_KIND };
  return { file: files[0] };
}
