# ADR 0150: Studio Shows A Change From Outside When A Page Opens

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, from a mockup and a questionnaire. The operator selected a check box for the setting.
- Technical choices: items 4, 6, 11, 12 and 13 are choices of the planning model.
- Changed by: [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) item 5: the check runs for each feed after the feed list loads and at each open of Releases.
- Source: [open items after `0.1.0-rc.19`, the request to detect a feed move, and the feed that two collections hold](../reviews/2026-11-06-local-use-findings.md)
- Changed by: [ADR 0184](0184-after-a-move-studio-shows-the-podping-of-the-move.md). Item 5: after a move by Studio, the notice gives the podping of the move.

## Context

Studio found a change from outside only at "Read Again" or as a refused write. The version marker keeps a write safe, but the user saw nothing before that time. A feed that a different tool moved looked normal. On 2026-11-06, one feed was in two collections that disagree, and musicindex followed a third address.

## Decision

1. The Releases page checks each feed of the page when it opens. A feed page checks its feed when it opens. No worker runs in the background.
2. The check compares the hosted file with the last file that Studio read. The result "Changed outside Studio" shows on the tile and in a notice on the feed page, with "Read Again".
3. The last read can name no new address, or a different one, in `itunes:new-feed-url`. A new value in the hosted file then gives "Moved outside Studio", with the `--error` token. The notice gives the address, and says that a write can put the feed back.
4. The notice of a move has "Show the Difference", which opens the comparison of ADR 0129. It also has "Keep This File Host", which hides the notice until the hosted file changes again.
5. The check asks musicindex for the feed address of each feed page. A different address gives a notice on the feed page, with both addresses, and no mark on the tile. With no answer, or with no entry for this feed, the page shows nothing.
6. The check reads only public addresses, so it needs no vault. A feed with no public address has no check.
7. A failed read shows nothing, and it does not change an earlier result. A check never blocks a page.
8. The check writes nothing and changes no draft. A change still goes through "Read Again" or the merge of ADR 0129.
9. The core makes the comparison and gives the result. The UI shows it (ADR 0065).
10. Settings has the check box "Check for outside changes", which starts on. With the box off, a page sends no request for this check.
11. Each read has a time limit, and the Releases page reads a maximum of 4 feeds at one time. On a server, the network policy of ADR 0057 applies.
12. A feed with no record takes the bytes of its first check as its start, so that check gives no result. Each later read or write of Studio records its bytes.
13. After a write, the bytes from before that write are not a change, because a public address can give them for a short time.

## Alternatives Considered

- A check each hour. Not chosen by the operator: it needs a worker, and it adds load on a server.
- A check of the hosted file only. Not chosen by the operator: it misses a split, where the hosted file looks correct and the index reads a different copy.
- No setting. Not chosen by the operator: a user on a slow connection must be able to stop the requests.

## Consequences

- Each open of Releases sends one read for each feed, and each host can see these reads in its log.
- Each open of a feed page sends one request to musicindex with the feed GUID.
- A file host that changes the bytes of a file can give a false "Changed outside Studio". The comparison uses the file bytes, so a change of a header does not count.
