# Roles, Levels And Words Of The Music Structure

Status: proposal. The operator made the choices below on 2026-11-24. The roles
belong to musicindex ADR 0036, ADR 0066 and ADR 0081. Thus the shared rules go
to the orchestrator session, and the musicindex session writes the record. This
note is advisory until that record exists.

## Context

- The prototype of the levels and moves uses music words for each change
  (`.studio-dev/design-brief/prototype/structure.html`, which Git ignores). The
  operator accepted the music words. The prototype used "host" for the
  `hosting` role.
- The word "host" has three meanings at this time:
  - The `rel` role `hosting` is the party that hosts and writes the RSS
    (musicindex ADR 0036 R-ROLE-2).
  - The `rel` role `host` is a person who presents
    (musicindex ADR 0081 R-CURATOR-2 item 1).
  - A file host is the server where Studio puts the files (ADR 0092).
- Studio keeps one role set for each publisher feed. Each link of the feed
  writes that set in `rel` (ADR 0082 items 1 and 8). Stophammer keeps the
  `rel` of each link (Stophammer ADR 0101).
- The operator described the levels: at the root is the party that hosts and
  writes the RSS. Sometimes this is the artist. Sometimes it is a person with
  artist friends. That person can give the feed the name of a label or a brand.
  A label can also host the files of artists that it does not put on the label.

## Operator Choices Of 2026-11-24

1. **Roles for each link.** A link carries a subset of the roles of its feed.
   Example: the feed Northwind Records has the roles `label hosting`. The link
   to White Triangles has `rel="label hosting"`. The link to the EP of a friend
   has `rel="hosting"`, so the EP is not on the label.
2. **Levels.** Hosting, label, curator and artist make the levels of the tree.
   Producer, host, author, network and sponsor are credits. They show on a page
   and do not make a level.
3. **Screen word for `hosting`.** The screen says "Published by" and
   "Publisher". The `rel` word stays `hosting`, because other apps read it.
4. **Name of the left panel.** The panel shows the full structure of the music,
   not a list of releases. The name "Releases" must change. The new name is
   open.

## Proposed Shared Rules

These rules are proposals for the musicindex record. The musicindex session
writes the text of the record.

1. The `rel` of a link is a subset of the role set of its feed. The role set of
   a feed is the union of the `rel` of its links, plus the roles that the user
   selects for the feed.
2. A new link gets each level role of its feed. The user can remove a role
   from one link.
3. The level roles are hosting, label, curator and artist, in that rank. A
   release is below an artist. A playlist is below a curator. A page of a
   lower rank is below a page of a higher rank when a link between the two
   gives the higher role. The feed role alone does not make a level.
4. The screen word of the role `hosting` is "Publisher", and a sentence says
   "published by". The screen word of the role `host` stays "Host". No screen
   uses "host" for the role `hosting`.
5. Because "Publisher" names one role, a screen does not use "publisher feed"
   for each feed with roles. The code names can stay.

## Effects In Studio

- ADR 0082 item 8 changes: a link writes its own subset, not the full set.
- ADR 0087 item 1 reads the role of each link.
- ADR 0215 item 2 changes: a level comes from the role of a link, as rule 3
  says.
- The sheets of the prototype say "Published by" and "Change publisher…". The
  release sheet shows which roles each link gives.
- File hosts keep the words of ADR 0092.

## Open Points

- The new name of the left panel (item 4).
- The position of curator relative to label: a label can also list playlists.
- The rank of a hosting feed that is also the artist feed, for example
  `rel="artist hosting"` on each link. The tree can show one row with the two
  roles, or two levels.
