// The result of a write to one feed (ADR 0217 items 2 to 4).
export function writeResult(feed, connector) {
  const result = {
    title: 'Written',
    line: `${feed.title} is written to ${connector.name}.`,
  };
  if (feed.public_url) return { ...result, address: feed.public_url };
  return {
    ...result,
    warning: {
      title: 'Listeners cannot get it yet.',
      text: connector.kind === 'local'
        ? 'This folder has no public address.'
        : 'This file host gives no public address.',
    },
  };
}
