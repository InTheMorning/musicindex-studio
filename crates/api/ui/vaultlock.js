// The words of the time before the vault locks (ADR 0142). The core gives the
// times, so this module has no rule of its own.

export const LOCK_HINT = 'Studio closes the vault after this time with no use of a file host key. A longer time keeps the key in memory for longer.';

export function lockChoiceText(minutes) {
  if (minutes === 0) return 'Never';
  if (minutes === 60) return '1 hour';
  return minutes === 1 ? '1 minute' : `${minutes} minutes`;
}
