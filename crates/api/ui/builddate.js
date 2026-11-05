// The texts of the build date in Review (ADR 0127 items 8, 11, 12 and 23, ADR
// 0042). This module has no Preact and no browser state. The Node tests in
// `ui-tests/builddate.test.mjs` examine it. The core owns the checked time and
// each state (ADR 0065). The UI calculates no clock difference.

export const NO_SERVER_TITLE = 'The build date does not change.';
export const NO_SERVER_TEXT = 'Studio could not check the time with a server. A later write sets the date.';
export const LATER_BASE_TITLE = 'The build date of this feed is in the future.';
export const LATER_BASE_TEXT = 'An index refuses an earlier date, so Studio keeps this date. The date is:';
export const CLOCK_AHEAD_TITLE = 'The clock of this computer is ahead.';
export const CLOCK_AHEAD_TEXT = 'Studio uses the time of a server for the build date.';

// The notices for the state `build_date` of a diff, in their sequence. A
// state with no notice gives an empty list. No notice stops a write.
export function buildDateNotices(state) {
  if (!state) return [];
  const notices = [];
  if (state.clock_ahead_seconds != null) notices.push({ title: CLOCK_AHEAD_TITLE, text: CLOCK_AHEAD_TEXT });
  if (state.no_server) notices.push({ title: NO_SERVER_TITLE, text: NO_SERVER_TEXT });
  if (state.later_base != null) notices.push({ title: LATER_BASE_TITLE, text: LATER_BASE_TEXT, value: state.later_base });
  return notices;
}
