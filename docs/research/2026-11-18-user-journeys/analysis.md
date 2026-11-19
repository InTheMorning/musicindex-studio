# Analysis of two journeys for a new independent artist

Date: 2026-11-18. Status: research record. This record is advisory. It states no rule and gives no design.
The operator selects the themes for mockups. A decision record or a task comes only after that selection.

## Scope

- **Journey 1. First run to a written release.** Walks J01 and J02, with the variants J04 (no artist page)
  and J06 (a payment recipient).
- **Journey 2. People: collect and merge identities.** Walk J07, and inventory C §3, §11 and §15.
- **User.** A new independent artist who uses Studio for the first time. The artist knows music files. The
  artist does not know RSS, Nostr, Lightning, podping, feeds or file hosts.

## Method

Two evaluators worked independently on the same brief. Neither saw the result of the other.

| Evaluation | Delegate | Input | Findings | Result |
|---|---|---|---|---|
| Cognitive walkthrough: four questions for each step | ChatGPT `gpt-6-astra`, effort high | Code and walk tables, no screenshots | 15 | `evaluations/cognitive-walkthrough.md` |
| Heuristic evaluation: the 10 heuristics of Nielsen | Claude `sonnet`, effort high | Screenshots and code, and the other two apps | 31 | `evaluations/heuristic-evaluation.md` |

Both evaluations also did four special checks: errors and recovery, dead ends, the words that the user must
learn, and the feedback after an action. The heuristic evaluation also compared Studio with the Music Index
website and HitIt.

The two result files are the recorded tool output. They are kept without a change, except that absolute
paths are relative. They are not written in STE.

The main session examined the code of each claim that has an effect on the rank below. The column
"Examined" gives the result.

Severity: 0 no problem, 1 cosmetic, 2 minor, 3 major, 4 the user cannot reach the goal.
A theme has the highest severity of its examined findings.

## Themes in rank order

The rank uses the severity first, then the agreement of the two evaluators, then the frequency of the step.

| Rank | Theme | Severity | Both found it | Journey |
|---|---|---|---|---|
| T1 | A written release is not a public release | 4 | Yes | 1 |
| T2 | Payment setup has no guide and no check | 3 | Yes | 1 |
| T3 | A merge is fast, has no undo and is hard to check | 3 | Yes | 2 |
| T4 | Review stops the write for a missing artist page | 3 | Yes | 1 |
| T5 | Errors do not tell the user how to recover | 3 | Yes | 1, 2 |
| T6 | People leads the user to make extra persons | 3 | Yes | 2 |
| T7 | Studio does not tell when a release is incomplete | 3 | No | 1 |
| T8 | Words from the feed standard with no explanation | 2 | Yes | 1, 2 |
| T9 | The first-run sheet does not match its steps | 2 | Yes | 1 |
| T10 | Words and status display differ from the other two apps | 2 | No | 1, 2 |

### T1. A written release is not a public release

The artist wants listeners to hear the music. Studio ends the first run with a local folder as the file host.
The public address of that folder is optional. The write succeeds, but no listener can get the music.

- The default file host in the first run is `"Folder on this computer"`, with the line
  `"The public address is optional."` (`welcome.js:92`). Shot `j01-05-where`.
- After the write, no message tells that the write succeeded. The toolbar shows `"No changes"` (O5).
- The release page then shows `"This feed has no public address yet."` No step tells the artist how to get
  a public address. Shot `j02-06-after-write`.
- The status icon of the release in the sidebar is a dashed circle with no word (`outsidecheck.js:206`).
- No link opens the release on musicindex.org (O8). HitIt has such a link (heuristic F7).

Findings: walkthrough F1 and F4. Heuristic F1, F2, F4, F6, F7 and F8.
Examined: the default file host, the optional public address and the absence of a result message are true.
Severity 4, because the artist cannot reach the goal "listeners hear my music" on the default path.

### T2. Payment setup has no guide and no check

The artist wants to get paid. The payment page has the name `"Value Routes"` (`shell.js:226`). The form asks
for a `"Node Address"` or an `"LNURL Address"` and does not tell the artist how to get one.

- The form accepts a Lightning address with the type `"Node Address"` (O2). Review and the Check page show
  no finding.
- An empty `"Split (shares)"` shows as 100.0 % (O16). The form does not tell how shares become percentages.
- The Simplified diff in Review shows `"Payments"` with no detail (O6). The artist cannot check the address
  or the share before the write.

Findings: walkthrough F3, F8, F9 and F10. Heuristic F12, F13, F27, F28 and F29.
Examined: O2 and O16 come from walk J06. The form checks only an empty address and a duplicate
(`values.js:247`, `values.js:279`).

### T3. A merge is fast, has no undo and is hard to check

A merge moves each item of the other person and deletes that person. The sheet asks for one click.

- The sheet selects the first other person in alphabetical order before the user selects
  (`peoplepages.js:1278`). With many persons, this can be the wrong person.
- `confirms.js` has no confirmation for a merge. A deletion of a person has a confirmation.
- No step undoes a merge. `"Discard"` applies only to field changes on the page.
- The rows of the sheet can have the same text, for example 4 rows `"The Doerfels"` `"Credit name"` (O17).
- After the merge, no message tells the result (O18).

Findings: walkthrough F12, F13 and F14. Heuristic F19, F20, F21 and F22.
Examined: the first selection, the absence of a merge confirmation and the row text (`people.js:2199`) are
true. An undo is not in the code.

### T4. Review stops the write for a missing artist page

A new release with no linked artist page cannot be written. The Check page shows the same condition only as
a `"Notice"` (O3). The repair links the page, and that link also changes the draft of the artist page, which
the artist must write separately (O4).

- When no artist page exists, `"Link a Page"` shows `"Studio has no other artist or label page to link to."`
  The page has no control to make one. **Inference** from `editor.js:2531`. No walk did this branch.
- The write button is not there while the refusal shows (`editor.js:2127`, `editor.js:2228`). The artist
  sees no disabled button and no reason at the button.

Findings: walkthrough F6. Heuristic F10 and F11.

### T5. Errors do not tell the user how to recover

- A write error shows `"Something went wrong."` and the server text (O7, `app.js:441`).
- The import shows an error for a file, and the user must remove the row and select the file again. There is
  no retry on the row (walkthrough, table of errors).
- When a suggestion list of a person page fails to load, the list is empty and shows no error
  (`peoplepages.js:856`). The user cannot tell "nothing found" from "the load failed".
- The button `"Connect a File Host…"` of the read-only notice has no effect (O1).

Findings: walkthrough F7 and F15. Heuristic F15, F17 and F26.
Examined: the silent empty list is true (`() => setMore([])`).

### T6. People leads the user to make extra persons

The purpose of People is to join the identities of one person. The suggestion note says to check each one and
then make a person. The menu `"Person"` of the check sheet starts on `"A new person"` when no person matches.
The path that adds a suggestion to a person that exists has fewer steps, but the page does not tell about it
(O19).

- Studio suggests a pair when the credits or the value blocks share a Nostr key, a name or a wallet
  (`people.js:70`). Two names with different wallets and no key show as two suggestions with no link between
  them. The user can merge them from the person page.
- The sidebar groups and the counts on the People page have no explanation (heuristic F25).

Findings: walkthrough F11. Heuristic F23, F24 and F25.
Examined: heuristic F23 is only partly correct. It says that two names never show as a pair. The evidence
`same_wallet` makes a pair of two names with the same wallet. The severity of F23 is 2 here, not 3.

### T7. Studio does not tell when a release is incomplete

The Check page has no finding for a release with no tracks, no cover or no payment recipient. The finding
kinds are in `crates/core/src/feed_check.rs:20`. A new release opens on the album page, and the write button
shows before the import (O11).

Findings: heuristic F14. The walkthrough did not find it.
Examined: the list of finding kinds has no such kind.

### T8. Words from the feed standard with no explanation

The walkthrough lists 36 terms that the artist meets on the two journeys. 23 of them have no explanation at
the place where they show first. Examples:

- First run: `"Nostr site (nsite)"`, `"Blossom servers"`, `"S3 or Cloudflare R2"`, `"SFTP server"`,
  `"storage bucket"` (`welcome.js:93`).
- Editor: `"The itunes:author of the feed."` (`editor.js:2670`), `"Podcast GUID"` in Review (O6),
  `"Send a podping"` (`announce.js:20`).
- Payments: `"Value Routes"`, `"Node Address"`, `"LNURL Address"`, `"Split (shares)"`, `"Custom key"`.
- People: `"Payees with no person"`, `"value blocks"`.

Findings: walkthrough F2 and F5, and its vocabulary table. Heuristic F5, F9, F16 and F18.

### T9. The first-run sheet does not match its steps

The sheet shows three stages and closes after two (O9). The second `"Continue"` makes the artist page, but
the label does not tell that. The library result can show counts of zero. The first run asks "where" two
times: the library folder and the file host.

Findings: walkthrough F2. Heuristic F3 and F4.

### T10. Words and status display differ from the other two apps

- Studio says `"Node address"` and `"LNURL address"`. The website says `"Lightning node"` and
  `"Lightning address"` (`../musicindex/search.html:5286`). HitIt uses its own words.
- Studio says `"Split (shares)"`. HitIt says `"Shares"`.
- Studio shows a write error as raw server text. **Inference:** this differs from the notice rules in
  musicindex ADR 0023 and ADR 0026. The main session did not examine these ADRs.

Findings: heuristic F27 to F31. Only the heuristic evaluation did this check.

## The five themes with the most effect on this user

1. T1: the artist writes a release and no listener can get it.
2. T2: the artist can enter a wrong payment address and see no problem.
3. T3: a wrong merge has no undo.
4. T4: the write stops for a page that the artist has not heard of.
5. T6: the artist makes extra persons and then must merge them.

Both evaluators put T1, T2 and T3 in their own first five.

## Limits

- No real user did these journeys. Each finding is a prediction by an evaluator, or a fact of the code.
- The walks used no network. The steps with a remote host, a relay or Music Index are code reading only.
- No walk used server mode, the narrow layout on these journeys or a touch device.

## Operator choices

Operator choices of 2026-11-18. [ADR 0217](../../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) owns them.

- T1: after each write, a result panel tells where the release went and if listeners can get it. When they
  cannot, the panel gives one next step. The first run does not change.
- T2: the type choice of a payment address stays. Studio explains each type to the user with a picture, a
  text and examples, so that the user knows which type to select. The menu `"Type"` stays a select menu.
  Under it, a panel shows an icon, one sentence, an example and where to find the address. The panel changes
  with the selection. The words of ADR 0138 stay: `"LNURL address"` and `"Node address"`. `"LNURL Address"`
  is the first item and the default.
- T3: the merge sheet selects no person at the start. Each row tells its feed. After the merge, a message
  with an undo shows. This needs an undo in the core.

Operator choices of 2026-11-19. [ADR 0218](../../adr/0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) owns them.

- T4: the refusal panel in Review links the page. With no page, the panel makes an artist page. The write of
  the artist page stays separate, and the result panel tells it.
- T6: the menu "Person" of a suggestion starts on "Select a person". The note of the fold tells the user to add
  a name to a person that exists.

More operator choices of 2026-11-19. [ADR 0219](../../adr/0219-a-release-checklist-and-a-check-of-the-address-type.md) owns them.

- T7: the page of a release shows a checklist of what it still needs. The same rows are findings on the Check
  tab.
- O2: an address in the format of the other type gives a finding, and its button changes the type in the
  draft.
- The payment form keeps the menu "Type". The menu shows a different address field for each type.
