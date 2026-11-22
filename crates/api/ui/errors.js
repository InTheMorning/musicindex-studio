// The step of each error code and the rule for "Try Again" (ADR 0222
// items 1 and 2). The module has no Preact, so that the Node tests can
// import it.

export const ERROR_STEPS = Object.freeze({
  offline: { step: 'Make sure that Studio is still running, then try again.', needsUser: false },
  network: { step: 'Check the internet connection of this computer, then try again.', needsUser: false },
  not_reachable: { step: 'Check the internet connection of this computer, then try again.', needsUser: false },
  host_unavailable: { step: 'The file host did not answer. Check the connection, then try again.', needsUser: false },
  host_unauthorized: { step: 'The file host refused the login. Check the login of the file host in File hosts.', needsUser: true },
  vault_locked: { step: 'Unlock the vault, then try again.', needsUser: true },
  signature_required: { step: 'Connect your Nostr signer, then try again.', needsUser: true },
  stale_revision: { step: 'The page changed in a different place. Open the page again, then do the change again.', needsUser: true },
  conflict: { step: 'The page changed in a different place. Open the page again, then do the change again.', needsUser: true },
  read_only: { step: 'Studio cannot change this feed. It is read-only.', needsUser: true },
  in_change_set: { step: 'Write or discard the changes first.', needsUser: true },
  no_public_address: { step: 'Give this file host a public address in File hosts.', needsUser: true },
  address_not_public: { step: 'Give this file host a public address in File hosts.', needsUser: true },
  too_large: { step: 'The file is too large. Use a smaller file, or change the limit in Settings.', needsUser: true },
  download_limit: { step: 'The file is too large. Use a smaller file, or change the limit in Settings.', needsUser: true },
});

// A new try can fix a failure with no answer or with an error of the server.
export function canTryAgain(error) {
  return error?.status === 0 || error?.status >= 500;
}

export function errorStep(error) {
  return ERROR_STEPS[error?.code]?.step || 'Try again. If the error stays, restart Studio.';
}

export function retryHelps(error) {
  return canTryAgain(error) && !ERROR_STEPS[error?.code]?.needsUser;
}
