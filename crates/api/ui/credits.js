// The credits of a track (ADR 0064 item 8). A track with no persons uses the
// album credits. This module has no Preact and no browser state.

// Does the track use the album credits? It does when it has no persons.
export function usesAlbumCredits(item) {
  return item.persons.length === 0;
}

// The item with its own credits. They start as a copy of the album persons.
// Each copy has no `_cid` and no `key`, so that the editor gives it a new
// client ID and the core takes it as a new entry (ADR 0022). A copy with the
// key of an album person names an item person that does not exist.
export function withTrackCredits(item, albumPersons) {
  const persons = albumPersons.map(({ _cid, key, ...person }) => ({ ...person }));
  return { ...item, persons };
}

// The item that uses the album credits again.
export function withAlbumCredits(item) {
  return { ...item, persons: [] };
}
