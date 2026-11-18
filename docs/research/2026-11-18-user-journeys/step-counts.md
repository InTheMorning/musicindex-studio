# Step counts

This table gives the step count of each journey of the previous study (A1 to A25), at commit `a64637d`.
The previous counts come from `02-as-is.md` of the study at Studio commit `563cce6`.
The step method is in `README.md`.

Each current count has a source:

- **Shot**: a walk in `walks.md` did the steps.
- **Code**: the count comes from an inventory, with the section number. It is an inference from the code.

| ID | Journey | Start | Previous | Now | Source | Difference |
|---|---|---|---|---|---|---|
| A1 | Unlock the vault with the passphrase | Unlock sheet | 2 | 2 | Code, A §3 | Same. 3 from the `"Unlock"` button in the footer. |
| A2 | First run to one feed, address known | Empty data | 6 | 6 | Code, A §1 and §2 | Same: library 2, Welcome discovery 4. |
| A3 | First run to one feed, found by name | Empty data | 8 | 8 | Code, A §1 and §2 | Same: library 2, Welcome search 6. |
| A4 | Connect a local folder as a host | Shell | 3 or 4 | 3 or 4 | Code, A §6 | Same. 4 when a host exists. |
| A5 | Make a new release | Artist page | 5 | 3 | Shot, J02 | 2 fewer. The artist comes from the page. The path is made from the title. |
| A6 | Import audio files into a release | Album page | 3 | 4 | Code, B §5 | The previous count did not count the native file selection. With the same rule, the count is 3. |
| A7 | Add one track from typed facts | Album page | 6 | 6 | Code, B §6 | Same. 5 with no title. |
| A8 | Change one field and write | Release list | 4 | 4, or 8 to 11 | Shot, J04 | 4 when the release has an artist page. With no artist page, Review stops the write. The link adds 4 steps, and the write of the artist page adds 3. |
| A9 | Add a recipient with a split, then write | Album page | 7 | 7, or 8 | Shot, J06 and Code, B §12 | Same. 8 when the feed has no payment block. The name is optional now. |
| A10 | Select a saved recipient, then write | Album page | 6 | 7 or 8 | Code, B §12 | 1 or 2 more. The split is a separate step. The saved list is a closed fold. |
| A11 | Copy a recipient from Music Index, then write | Album page | 9 | 9 | Code, B §13 | Same for one recipient: 6 + n, then Review and write. |
| A12 | Write with no change | Any | Not possible | Not possible | Code, B §16 | Same. Review says `"The draft is the same as the hosted file."` |
| A13 | Discard the draft | Album page | 3 | 3 | Code, B §18 | Same. |
| A14 | Put an earlier version back, then write | Album page | 4 or 5 | 4 or 5 | Code, B §18 | Same. |
| A15 | Remove a feed from the collection | Album page | 3 | 2 | Code, B §25, and Shot, J04 | 1 fewer. The button is at the end of the album page, not in `"Feed facts"`. |
| A16 | Read a feed again | Feed page | 1 | 1 | Code, A and B §21 | Same. |
| A17 | Find a feed by name in the collection | Shell | No search | No search | Code, A §2 | Same. Releases has sort and view controls, and no search or filter. |
| A18 | Apply a changed file of the library folder | Album page | 2, or 4 with a write | 2 to 4 | Code, B §24 | Now a scan on the album page: 2, and 3 with one action. |
| A19 | Download the files of a release | Album page | 2 | 2 | Code, B §22 | Same. |
| A20 | Stop downloads | Files page | 1 | 1 | Code, B §22 | Same. 2 from the album page. |
| A21 | Recover from a hosted file that changed | Review | 6, plus each edit again | k + 5 | Code, B §19 | Now a merge. The user selects a value for each of the k conflicting fields. No edit is typed again. |
| A22 | Correct an address before the write | Review | 5 | 4 | Code, B §20 | 1 fewer when the field is on the open page. |
| A23 | Read the release on musicindex.org after a write | Browser | 3 (estimate) | 3 (estimate) | Code, B §17 | Same. The result notice shows the index address as text, not as a link (`app.js:4917`). |
| A24 | Connect the folder of a read-only feed | Read-only page | 3 or 4 | 3 or 4 | Code, A §6, and O1 | The notice has a button now, but the button has no effect. See `observations.md`, O1. |
| A25 | Lock Studio | Any | 1 | 1 | Code, A §3 | Same. |

## Journeys that the previous study did not count

| Journey | Start | Now | Source |
|---|---|---|---|
| Library folder at the first start | Empty data | 2, or 3 with a typed folder | Shot, J01 |
| Welcome to the artist page editor | Welcome | 4 | Shot, J01 |
| First run to a written album with one track | Empty data | 14 | Shot, J01 and J02 |
| Link an album to an artist page | Album page | 2 | Shot, J04 |
| Open a release in the narrow layout | Release list | 2 | Shot, J05 |
| Make an artist, label or hosting page | Release list | 5 | Code, A §7 |
| Add a person in People by hand | People | See inventory C §3 | Code, C §3 |
| Two suggestions to two persons, then a merge | People | 10 | Shot, J07 |
| Add a suggestion to a person that exists | People | 6 | Code, `people.js:2091` |

The count 14 is the sum of J01 (2 + 4) and J02 (8).
