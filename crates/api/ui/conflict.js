// The conflict notice model (ADR 0186). The core gives Unix times.

const localTime = (seconds) => new Date(seconds * 1000).toLocaleString();

export function conflictNotice(given) {
  const details = given || {};
  const lines = [];
  if (Number.isFinite(details.draft_started)) {
    lines.push(`Draft started ${localTime(details.draft_started)}`);
  }
  if (Number.isFinite(details.file_changed)) {
    lines.push(`File changed ${localTime(details.file_changed)}`);
  }
  return {
    heading: 'The hosted file changed after your draft started. Studio did not write.',
    times: lines,
    cause: 'The change can come from a different collection, a different Studio version, a different tool or a different person.',
  };
}
