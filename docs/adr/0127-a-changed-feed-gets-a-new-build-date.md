# ADR 0127: A Changed Feed Gets A New Build Date

- Status: Accepted
- Date: 2026-11-04
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-04 by the operator. The operator selected the exception to the writer rule and a new date on a granted write. The operator also selected a second time source as the protection from a wrong clock.
- Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F2. Research note `research_notes/studio/improving-the-state-of-feeds.md` in `../musicindex-research`. Stophammer record `docs/reviews/last-build-date-behavior-research.md`.
- Changes: [ADR 0008](0008-feed-types-and-feed-identity.md) and [ADR 0012](0012-lossless-tree-with-a-typed-model.md), for the rule that the writer changes only what the user changed. [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) items 13 and 15, for a granted write. [ADR 0126](0126-studio-refuses-a-bad-move-link.md) items 8 and 26, for the request of this record.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0057](0057-the-private-network-guard-of-the-server.md), [ADR 0077](0077-a-new-album-can-start-from-an-album.md), [ADR 0114](0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md) and Stophammer ADR 0053.

## Context

The review of the request found these facts at commit `1283bc7`:

- `crates/` has no code for `lastBuildDate`. The writer keeps the value of the source, so an edited feed keeps an old date. A new feed of Studio has no date.
- Stophammer ADR 0043 defines `lastBuildDate` as the time that a feed file was generated.
- Stophammer ADR 0053 section 3 refuses a submission when the submission and the record both have a date and the submitted date is earlier. The reason is `stale_submission`, and `force_reingest` does not skip the rule. An equal date passes. A feed with no date gets no protection (`src/api.rs:2934`).
- A self link move or a tag move does not go through that comparison. A move through a permanent redirect does (`src/api.rs:2927`).
- The write puts the approved bytes on the host, and the diff ID covers these bytes (ADR 0007). A date that the write adds would not be in the approved diff.
- Drafts save through many paths: `replace_model`, `edit_draft`, `apply_repair`, `draft_from_snapshot`, the rebase of a linked draft, and step 3 of a granted write.
- The editor saves a draft also when the user changed nothing, so a save is not a change.
- `WebFile` keeps no response header, so the core cannot read a `Date` header today.

### The Wrong Clock

The operator requires that an incorrect clock cannot make the index ignore a feed. A computer clock that is ahead gives a date in the future. The index stores it, and then refuses each later submission with an earlier date. The artist cannot repair this with a correct date, because the correct date is earlier. A later write from a different tool also gets a refusal.

The Stophammer record measured 9,840 feeds with a date on 2026-09-24. No feed had a date more than one day in the future, and no feed moved its date back. A future date is thus not a normal state in the index. A clock that is behind does not have this effect, because the date never goes back (item 9).

The research did not examine how other indexes and apps treat a date in the future. This stays open. The parser in `stophammer/stophammer-parser` reads `lastBuildDate` through `parse_date`, and its test passes for RFC 2822 with `+0000`.

## Decision

### The Exception

1. The writer changes only the elements that the user changed, with one exception: a change of a feed also sets its `lastBuildDate`. Only the core sets this element, and the user cannot edit it.
2. A read and a write with no edit keep the same bytes. A write of only file operations, with feed XML that does not change, sets no date. The corpus tests stay as they are.

### The Time Of The Date

3. A save that changes the feed gives the draft the date of its base. A save makes no request, and it does not read a clock for this element. The compare of a save ignores `lastBuildDate`.
4. The core sets the date one time, when it makes the diff of a draft that differs from its base. The date is thus in the approved bytes, and a retry of a move compares the same bytes.
5. Another diff with no other change keeps the date, so the diff ID stays the same. The revision that the editor holds stays valid.
6. An edit that the user undoes gives the bytes of the base, through item 3. The diff then has no date line.

### The Checked Time

7. The checked time is the earliest of the computer clock and the `Date` header of each server that answered. These servers are the musicindex API and the host of the public address of the feed. The time of a server is its `Date` header plus its `Age` header. A cache with no `Age` header can give a time that is too early, and that has no bad effect on the date.
8. The core needs an answer from one server or more. With no answer, the draft keeps the date of its base, and the write continues. A feed with no date stays with no date. An equal date passes in the index, so this is safe. Review says that the build date does not change.
9. The new date is the later of the checked time and the date of the base. A date never goes back.
10. A date in the base that the core cannot read counts as no date, and the new date replaces it.
11. A date in the base that is later than the checked time stays, because the index refuses an earlier date. Review gives a notice that names the date. Studio has no repair for it. Only the index operator can correct a stored value.
12. When the computer clock is more than 5 minutes later than the checked time, Review gives a notice: the clock of this computer is ahead, and Studio uses the time of a server. No clock difference stops a write.
13. The requests use the network policy `Public` in both shells, with a limit of 5 seconds for each request. They run outside the lock of the workspace. The core keeps the difference between each server and the computer clock in memory for 10 minutes, and it does not store it. A new start requests the time again.

### The Form

14. The core writes the date in RFC 2822 with the zone `+0000`, through `datetime::format_rfc2822`.
15. A changed date stays in the position of its element. A feed with no such element gets one in the position of ADR 0012 item 3.
16. Each path that saves a changed draft uses item 3. Each path that makes a diff for a write uses items 4 to 11. One core function does each of these, and a path gets no form of its own.

### Each Kind Of Write

17. A roll back is a change, so it gets a new date. The snapshot keeps its own bytes.
18. A new feed and a copy get a date at their first diff. A copy keeps no date of its source (ADR 0077).
19. A repair of a draft and a change set get a date in the same way as each other change. A Discard of a change set puts back each draft from before the change.
20. A feed move that changes the feed XML gets a date. The pointer file has the date of its draft. A later write on the new host gives the feed a later date than the pointer.
21. A granted write gets a new date. Step 3 of the write uses the kept difference of item 13. With no kept difference, the core sends the requests with their limit of 5 seconds. A request that fails does not stop the write, and item 8 applies. The date is the one change that a grant covers outside its live item, so the check of ADR 0054 item 15 ignores `lastBuildDate`.
22. A read-only feed gets a date when the user changes its draft, because Studio then writes it to a new host.

### The Review

23. The diff shows the date as each other change. The field view of ADR 0114 names it "Build date". The core gives each notice of this record, and the UI calculates no clock difference (ADR 0065).
24. The user has no control for this element in Review. Discard removes it with the rest of the draft.

## Alternatives Considered

- Keep the rule with no exception, and offer the date as a repair. Not chosen by the operator: the date then stays old in each feed that the user does not repair.
- Use only the computer clock. Rejected by the operator: one date from a clock that is ahead can make the index refuse each later write.
- Stop the write until the user corrects the clock. Not chosen by the operator: a wrong clock then stops all work, and the check still needs a server time.
- Add one second to the date of the base, with no clock. Not chosen by the operator: the order is correct, but the date is not a real time, and a feed with no date has no start value.
- Ask a time server through NTP. Rejected: this adds a protocol and a kind of connection that some networks block. The `Date` header comes from a client that Studio already has.
- Set the date in the write. Rejected: the bytes on the host then differ from the approved diff (ADR 0007), and a retry of a move finds different bytes.
- Set the date at each save. Rejected: a save then needs a request, and the editor saves with no edit.
- Set a new date at each diff. Rejected: each new look at Review gives a new diff ID, and an approval can become stale with no edit.
- Correct a date in the future to the checked time. Rejected: the index refuses an earlier date.
- Leave the date out of a granted write. Not chosen by the operator: a live change then looks like no change to a reader of the date.

## Consequences

- Each diff of a changed feed has one more line, when a server answers.
- Review sends a maximum of two more requests in 10 minutes. These requests tell the musicindex API and the file host that Studio is running.
- A diff can now change a draft. The editor must keep a valid revision after a diff, and the task needs a test for this.
- `WebFile` or a new call must give the `Date` header. `grant::covered` needs a change, so that a granted write passes its own check.
- Tests that compare the bytes of an edit need a fixed clock or a compare that ignores this element.
- A computer that is offline writes to a local folder with no new date. The next write with an answer sets it.
- A server clock and a computer clock that are both ahead by the same amount still give a date in the future. Studio cannot detect this.
- A date that a different tool wrote can still be in the future. Item 11 shows it, and Studio cannot repair it.
- F3 of the request can compare the date of the last write with the date in the index. That comparison has no decision yet.

## Invariants

- A read and a write with no edit give the same bytes.
- The bytes that the host gets are the bytes of the approved diff, or of a diff that a grant covers.
- Studio never writes a date that is later than the checked time, unless the base already has that date.
- With no server answer, Studio writes no new date.
- A later change of a feed never has an earlier date than the change before it.
- A clock difference never stops a write.
- The feed GUID and each item GUID do not change (ADR 0008).

## Non-goals

- A correction of the computer clock, or advice on how to correct it.
- The feed check (F1) and the index view (F3).
- A date on an item. `pubDate` stays as ADR 0093 gives it.
- A repair of a feed that the user does not change. Such a feed keeps its date until its next change.

## Acceptance Criteria

Mechanical:

- A core test: a read and a write with no edit give the same bytes, and a save with no edit gives no date.
- A core test: a save makes no request. A diff of a changed draft gives a new date with that line, and a second diff gives the same diff ID.
- A core test: after a diff, the next save of the editor with its revision passes.
- A core test: an edit that the user undoes gives the bytes of the base.
- A core test: with a computer clock 2 years ahead and a correct server, the date is the server time. The report has the notice of item 12.
- A core test: with a server 2 years ahead and a correct computer clock, the date is the computer time.
- A core test: with no server answer and a clock 2 years ahead, the draft keeps the date of its base, and the write completes.
- A core test: with a clock that is behind the base, the date does not go back.
- A core test: a date in the base that is later than the checked time stays, with the notice of item 11. A date that the core cannot read gets a new date.
- A core test: for each kind of write of items 17 to 21, no new date is earlier than the date before it. This is the condition of Stophammer ADR 0053 section 3.
- A core test: a feed with no element gets one element, and a feed with one element keeps its position.
- A core test: a granted write passes its check, and it changes only the live item and this element. A server that does not answer does not stop it.
- A core test: a retry of a stopped move finds the same bytes.
- An API test: the requests run with the lock of the workspace free.
- A Node test: the field view has a text for the date, and the model module has a text for each notice.

Visual, by the planning model:

- Review shows the date line, and the notices of items 8, 11 and 12, at 1280 px and at 390 px with touch.

Human, by the operator:

- After a write to a real file host, the index gives the new `last_build_date` for that feed.
- A write with the computer clock set ahead puts the correct date on the host, and the index accepts the next write.
- A roll back of a feed on a real file host reaches the index.
