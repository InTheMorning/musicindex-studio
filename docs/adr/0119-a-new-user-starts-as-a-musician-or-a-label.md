# ADR 0119: A New User Starts As A Musician Or A Label

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screens 2 to 6 of the mockups in `.studio-dev/mockup-start/site/`.
- Amended: 2026-11-02 by the operator: "My own server" is the first row of item 6. Item 11 comes from the visual check of task 083.
- Changed by: [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) items 1 to 4, step 2.
- Changes: [ADR 0072](0072-add-a-feed-opens-from-releases.md), the empty Releases page. [ADR 0020](0020-first-run-asks-one-question.md) items 1 and 2, the question of the first-run wizard (item 11). [ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) does not change for a user with feeds.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the first write. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) for the file host forms. [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) and [ADR 0083](0083-hosting-is-a-first-class-role.md) for the roles. [ADR 0107](0107-a-user-can-export-and-import-an-account.md) for the import.

## Context

- On 2026-11-02 a new user with no feed saw the Releases page with "No releases yet.", the button "Add a Feed" and the button "Import an Account".
- "Add a Feed" opens the Add feeds page (ADR 0073). With no file host, that page asks for a file host first. Its rows use the word "feed", and "Artist or label" is one row.
- The operator wants a new user to start with an artist page or a label page. A musician and a label need different paths. The operator also wants the first steps to use no word "feed".
- A new artist page or label page is a draft. ADR 0007 says that no write occurs without a diff that the user has seen.

## Decision

1. A user with no feed in the collection sees the page "Welcome to Studio" in the place of the empty Releases page. A user with one feed or more sees Releases, as before.
2. The Welcome page has the text "Studio publishes your music so that every music app can play it. What do you want to start with?" It has two groups of large rows, each with an icon, a title, one line and `›`:
    - "I make music": "Start with your artist page. Your albums go on it." It is the primary row.
    - "I run a label": "Start with your label page. Your artists and releases go on it."
    - In a second group, "My music is online already": "Find your artist or albums, and bring them into Studio." It opens the form "By address or name" of ADR 0073.
3. Below the groups, the line "Moving from a different server? Import an Account" opens the import of ADR 0107, where the shell permits it.
4. "I make music" and "I run a label" open a sheet with three steps, and a step line at its top:
    - Musician: "Your name", "Where it lives", "Your first album".
    - Label: "Label name", "Where it lives", "Your first artist".
5. Step 1 asks for the name ("Artist name" or "Label name") and an optional picture. The musician sheet has the hint "A band is one artist. Add the members later, in People."
6. Step 2, "Where it lives", has the text "Studio does not keep your music. It puts your {artist page or label page} and your songs on a file host that you choose. You can move them later." It shows one row for each file host type. "My own server" is first, because a folder on this computer lets a user try Studio before the user selects a hosting provider. Then come "Nostr site (nsite)" and "S3 or Cloudflare R2". A Blossom host holds media only (ADR 0048 item 2), so it is not a row. The lines are in the task file. A row opens the form of that file host (ADR 0045). With one connected file host, step 2 does not show. With two or more, step 2 is a select menu.
7. After step 2, the core makes the publisher feed as a draft, with the role `artist` or `label`. The sheet uses the route of the form that makes a feed. Studio writes nothing. The feed file gets its start value from the name (ADR 0073 item 8).
8. Then the page of the new feed opens. The toolbar shows the draft status and "Review", as on each editor page. The page has one large row at its top:
    - Musician: "Add your first album", with "Drop the songs in. Studio reads their names and order, and makes the album page." and the button "Add an Album".
    - Label: the rows "Add an artist", "Add a release" and "Find an artist that is online".
9. "Add an Album" opens "Add tracks" of ADR 0073 for a new album of this artist. The first write of the artist page and the album goes through Review (ADR 0007).
10. The first steps use the words "artist page", "label page", "album", "release" and "file host". They do not use the word "feed". The Add feeds page, the settings and the details keep the word "feed".
11. The Welcome page replaces the question of the first-run wizard, "Do you already have a feed?", and its page "Welcome" (ADR 0020 items 1 and 2). A new user goes from the library folder, or from the sign-up on a server, to the Welcome page. "My music is online already" uses the same search as the answer "Yes" of ADR 0020, and ADR 0020 items 3 to 15 do not change. Below the groups, the line "Moving from MSP 2.0? Import the Backups" opens the MSP 2.0 import.

## Alternatives Considered

- Two rows only, with "My music is online already" as a small link. Not chosen by the operator.
- The file host at the first publish, after the album. Not chosen by the operator: step 2 after the name is clear, and the draft then has a host.
- One row "Artist or label", as ADR 0073 has. Not chosen: a musician and a label need different next steps.

## Consequences

- The empty Releases page of a user with no feed changes. The Add feeds page does not change.
- The Welcome page and its sheet are new UI, with a model module and Node tests.
- The page of a new publisher feed gets the row of item 8 while the feed has no release.
- A task file gives the work.

## Non-goals

- A change of the Add feeds page for a user with feeds.
- A new connector or a new file host form.
- A write with no review.

## Acceptance Criteria

Mechanical:

- A Node test: the model gives the Welcome page for an empty feed list, and Releases for a list with one feed.
- A Node test: the steps of each path, and the step 2 rule of item 6 for zero, one and two connected file hosts.
- A Node test: the texts of items 2, 5, 6 and 8, and no word "feed" in them.
- A Node test: the body of the form that makes the feed has the role of the path.
- The four commit checks pass.

Visual: screens 2 to 6 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator signs up on the server as a new user, selects "I make music", gives a name, connects a file host, adds an album and writes the two feeds through Review.
