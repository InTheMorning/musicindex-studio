// The connected apps: the lines of each app row, the pairing notice and the
// grant control of the review sheet (ADR 0110 items 7, 8, 19 and 37). The
// core gives each state and each value. This module only chooses the text
// (ADR 0065). It has no Preact and no browser state (ADR 0042).

const plural = (count, one, many) => `${count} ${count === 1 ? one : many}`;

// The lines of one app row (screens 7 and 8). `app` is an object of
// `GET /api/v1/apps`. `timeText(seconds)` gives a time in the language of
// the user. `clockText(seconds)` gives the time of the day only.
export function appRowLines(app, timeText, clockText) {
  if (app.state === 'token') {
    return {
      key: 'Token from before key pairs',
      status: 'Studio no longer accepts tokens.',
      address: null,
      connectAgain: true,
    };
  }
  if (app.state === 'waiting') {
    return {
      key: 'No key yet',
      status: `Waiting for the app. The pairing text works until ${clockText(app.code_expires_at)}.`,
      address: null,
      connectAgain: false,
    };
  }
  const shows = app.shows || [];
  const parts = [shows.length === 0 ? 'No show yet' : plural(shows.length, 'show', 'shows')];
  if (app.last_used_at) {
    parts.push(`Last used ${timeText(app.last_used_at)}${app.last_address ? ` from ${app.last_address}` : ''}`);
  } else if (shows.length > 0) {
    parts.push('Not used yet');
  }
  return {
    key: app.short_key || 'No key yet',
    status: parts.join(' · '),
    address: app.live_value_origin || null,
    connectAgain: false,
  };
}

// What the page does with the result of `GET /api/v1/apps/{id}` while the
// notice is open. `polled` is the app object, or `null` after a 404.
export function pairingPhase(polled) {
  if (polled === null) return 'ended';
  return polled.state === 'waiting' ? 'waiting' : 'connected';
}

// The notice of a pairing (screens 6b and 6c). `made` holds the name and
// the pairing text of the app that Studio just made. The pairing text is in
// the result only while the app waits. In each other phase `text` is null,
// and the page must drop the text from its state.
export function pairingNotice(made, phase) {
  if (phase === 'connected') {
    return {
      title: `${made.name} is connected`,
      hint: 'Compare the key with the key that the app shows. If the app did not connect, select Remove.',
      text: null,
      copy: false,
    };
  }
  if (phase === 'ended') {
    return {
      title: `The pairing text of “${made.name}” ended`,
      hint: 'The 15 minutes ended, or the pairing text was removed. Select Connect an App to make a new pairing text.',
      text: null,
      copy: false,
    };
  }
  return {
    title: `The pairing text of “${made.name}”`,
    hint: 'Select Copy, then give the text to the app in the next 15 minutes, for example with musicindex-live-publisher studio connect. Select Done after the app says that it is connected. Done before that ends the pairing text.',
    text: made.pairingText,
    copy: true,
  };
}

// Done while the app waits ends the pairing text (ADR 0110 item 8). After
// the pairing, or after the end, Done only closes the notice.
export function doneRemovesApp(phase) {
  return phase === 'waiting';
}

// The control of the review sheet that gives a show to an app (screens 1 to
// 3). `apps` is `diff.apps`, the active apps. `grantApps` is
// `diff.grant_apps`, the app of each granted show. `signerNote` is the note
// of a file host with My signer, or null. The result has the kind of the
// control: `none` (a disabled menu), `check` (one app) or `select` (two or
// more apps). `appId` is the app of the grant, or null.
export function grantControl({ apps, grantApps, guid, signerNote, showTitle }) {
  const appId = grantApps && grantApps[guid] !== undefined ? grantApps[guid] : null;
  const granted = appId !== null && appId !== undefined;
  const blocked = Boolean(signerNote) && !granted;
  const notes = [];
  const suffix = showTitle === undefined ? '' : `: ${showTitle || 'Untitled show'}`;
  if (apps.length === 0) {
    return {
      kind: 'none',
      label: `Connected app for this show${suffix}`,
      options: [{ value: '', text: 'No app' }],
      value: '',
      disabled: true,
      notes: ['No app is connected. Connect an app in Settings.'],
      appId: null,
    };
  }
  if (signerNote) notes.push(signerNote);
  if (apps.length === 1) {
    const [app] = apps;
    return {
      kind: 'check',
      label: `Let ${app.name} start and end this show, and set its live value${suffix}`,
      checked: granted,
      disabled: blocked,
      notes: [`Live value address: ${app.live_value_origin}`, ...notes],
      appId: app.id,
    };
  }
  const names = apps.map((a) => a.name);
  const options = [{ value: '', text: 'No app' }, ...apps.map((a) => ({
    value: String(a.id),
    text: names.filter((n) => n === a.name).length > 1 && a.short_key ? `${a.name} · ${a.short_key}` : a.name,
  }))];
  return {
    kind: 'select',
    label: `Connected app for this show${suffix}`,
    options,
    value: granted ? String(appId) : '',
    disabled: blocked,
    notes: ['This app can start and end this show, and set its live value. No other app can.', ...notes],
    appId: granted ? appId : null,
  };
}

// The notice at the top of each editor page of a feed whose active grant a
// draft with changes blocks (screens 9 and 10, ADR 0110 item 42).
// `blocked` is `grant_blocked_by_draft` of the feed state. `startText` is
// the start time of the show in the language of the user, or null.
export function draftBlockedNotice(blocked, startText) {
  const app = blocked.app_name || 'The app';
  const show = blocked.show_title || 'the show';
  return {
    title: `${app} cannot start ${show} while this draft has changes`,
    text: `Write or discard the draft before ${startText || 'the show starts'}. Select Review to write it.`,
  };
}
