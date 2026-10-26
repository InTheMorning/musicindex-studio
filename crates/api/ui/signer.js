// The connection of a NIP-46 signer to the workspace, inside a sheet or a
// form (ADR 0056 items 13 and 17). The API holds the connection and the
// relays. The component shows the QR code, the link and the bunker field,
// and it asks for the state each 1.5 seconds. It works in both shells.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { api, Field, ErrorNotice, useAction } from './app.js';
import { isBunkerAddress, qrSource, nip46ErrorText } from './account.js';
import { signerView } from './signing.js';

const POLL_MS = 1500;

export function SignerConnect({ onConnected, onCancel }) {
  const [link, setLink] = useState(null);
  const [bunker, setBunker] = useState('');
  const [phase, setPhase] = useState('starting');
  const [failure, setFailure] = useState(null);
  const start = useAction();
  const paste = useAction();
  const alive = useRef(true);
  // True after the signer connected. The cleanup then keeps the signer.
  const done = useRef(false);

  const finish = (pubkey) => {
    done.current = true;
    onConnected(pubkey);
  };

  // Ask for the state until the signer answers or the connection fails.
  const watch = async () => {
    while (alive.current && !done.current) {
      await new Promise((resolve) => { setTimeout(resolve, POLL_MS); });
      if (!alive.current || done.current) return;
      const view = signerView(await api('/signer'));
      if (view.phase === 'connected') { finish(view.pubkey); return; }
      if (view.phase === 'failed') {
        setPhase('failed');
        setFailure(view.error);
        return;
      }
      if (view.phase === 'idle') return;
    }
  };
  const begin = () => start.run(async () => {
    setPhase('starting');
    setFailure(null);
    const made = await api('/signer/connect', { method: 'POST', body: {} });
    setLink(made);
    setPhase('waiting');
    await watch();
  });

  useEffect(() => {
    alive.current = true;
    begin();
    return () => {
      alive.current = false;
      // Stop a wait that nobody uses. A connected signer stays.
      if (!done.current) api('/signer', { method: 'DELETE' }).catch(() => {});
    };
  }, []);

  const connectBunker = (event) => {
    event.preventDefault();
    paste.run(async () => {
      setPhase('connecting');
      try {
        const answer = await api('/signer/connect', { method: 'POST', body: { bunker: bunker.trim() } });
        const view = signerView(answer);
        if (view.phase !== 'connected') throw new Error('The signer did not connect.');
        finish(view.pubkey);
      } catch (e) {
        setPhase('failed');
        throw e;
      }
    });
  };

  const stop = () => {
    alive.current = false;
    api('/signer', { method: 'DELETE' }).catch(() => {});
    done.current = true;
    if (onCancel) onCancel();
  };

  const waiting = phase === 'waiting' || phase === 'starting';
  return html`<div class="signer-connect">
    <p>Scan the QR code with your signer app. On a phone, open the link instead. Or paste a bunker address from your signer.</p>
    ${link && html`<img class="qr" alt="QR code of the signer link" src=${qrSource(link.qr_svg)} width="240" height="240" />`}
    ${link && html`<div class="actions"><a class="button action" href=${link.uri}>Open in the signer app</a></div>`}
    <form class="form" onSubmit=${connectBunker}>
      <${Field} id="signer-bunker" label="Bunker address" autocomplete="off" spellcheck="false"
        placeholder="bunker://…" value=${bunker} onInput=${(e) => setBunker(e.target.value)} />
      <div class="actions"><button class="button action" type="submit" disabled=${paste.busy || !isBunkerAddress(bunker)}>Connect</button></div>
    </form>
    ${phase === 'waiting' && html`<p class="hint" role="status">Waiting for the signer…</p>`}
    ${phase === 'connecting' && html`<p class="hint" role="status">Connecting to the signer…</p>`}
    ${failure && html`<div class="notice error" role="alert"><strong>${nip46ErrorText(failure)}</strong></div>`}
    <${ErrorNotice} error=${paste.error || start.error} />
    <div class="actions">
      <button class="button" type="button" onClick=${stop}>${waiting ? 'Stop' : 'Close'}</button>
      ${phase === 'failed' && html`<button class="button action" type="button" onClick=${begin}>Try again</button>`}
    </div>
  </div>`;
}
