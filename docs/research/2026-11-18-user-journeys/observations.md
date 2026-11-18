# Observations

This page gives facts for the subsequent analysis. It gives no design proposal and no priority.
Each item has an evidence mark from `README.md`. The walk IDs refer to `walks.md`.

## Defects

**O1. The button of the read-only notice has no effect.** Code.

The outside notice `followed_move` shows a button with the label `"Connect a File Host…"` when the feed has no
writable host. Its handler calls `nav && nav.go({ name: 'hosts' })` (`app.js:4916`). The `nav` value comes
from `NavContext` (`app.js:4837`). That value has no member `go` (`app.js:4013`).
**Inference:** a click causes a `TypeError` and the page does not change. No walk reached this notice.

**O2. A recipient address is not examined for its type.** Shot J06, Code.

The sheet `"Add a Recipient"` accepted `artist@getalby.com` with the type `"Node Address"`. The hint of the
field asks for 66 hexadecimal characters. The sheet examines only that the address is not empty and is not
in the list (`values.js:247`, `values.js:279`). Review and the Check page showed no finding.

## The write and Review

**O3. A missing artist page has two different weights.** Shot J04, J06.

The Check page shows the condition as `"Notice"` `"No artist page"`. Review shows it as an error, and the
write button is not there (`editor.js:2127`, `editor.js:2228`).

**O4. A link to an artist page changes two drafts.** Shot J04.

After `"Link to This Page"`, the artist page draft also changes. The notice tells the user to open that draft
and write it. The album write and the artist page write are two separate journeys.

**O5. No notice tells that a write succeeded.** Shot J02.

After the write, Review closes. The toolbar changes from `"Saved, not written"` to `"No changes"`.

**O6. The Simplified diff shows rows with no detail and internal values.** Shot J02, J06.

J06 showed a row `"Payments"` with no detail above a row `"Payments"` `"Added"`.
J02 showed `"Podcast GUID"` with the value `pending-60384c23-…`.

**O7. A write failure shows the server text.** Shot J04.

The title was `"Something went wrong."` The text was the server message, in lower case, with a proxy
error. The cause in the walk was the test data. The form of the message is a fact of the UI.

**O8. The index result shows addresses as text.** Code.

The result notice after a write shows the Music Index address as text, not as a link (`app.js:4917`).

## The first run

**O9. The first-run sheet shows three stages and closes after two.** Shot J01.

The stage list shows `"Your name"`, `"Where it lives"` and `"Your first album"`. The third stage is a card in
the artist page editor.

**O10. The library folder is a text field.** Shot J01, Code.

The first-run library form has no folder picker (`app.js:1786`).

**O11. A new release opens on the album page, not on the import.** Shot J02.

The user must click `"Add tracks"` after `"Make the Release"`. An album with no tracks can be written.

**O12. The kind label changes after an import.** Shot J02.

The new release had `"Kind"` `"From the tracks"`. It showed `"ALBUM"` with no tracks and `"SINGLE"` with one
track. The first page tab changed from `"Album"` to `"Single"`.

## Navigation

**O13. Releases has no search and no filter.** Code, inventory A §2.

The release list has sort, view and add controls only (`app.js:3408`).

**O14. In the narrow layout, the fixed destinations are at the end of a long list.** Shot J05.

`"People"`, `"File hosts"` and `"Settings"` are below all the feed groups of the `"Destinations"` page.

**O15. In the narrow layout, a release needs two taps to open.** Shot J05.

The first tap opens the outline. The second tap opens the editor.

## Values

**O16. An empty split shows as 100 %.** Shot J06.

After `"Add the Recipient"`, `"Split (shares)"` was empty and the bar showed 100.0 %.

## People

**O17. Rows of the merge sheet can have the same text.** Shot J07, Code.

The sheet `"Merge with Ben Doerfel"` showed 4 rows `"The Doerfels"` `"Credit name"`. A row does not tell
the feed or the role of the credit (`people.js:2199`).

**O18. No notice tells that a merge succeeded.** Shot J07.

After `"Merge the People"`, the sheet closes and the person page shows the merged items.

**O19. Two paths join identities.** Shot J07, Code.

A user can make two persons and then merge them. A user can also add a suggestion to a person that exists,
with the menu `"Person"` of the check sheet (`people.js:2091`). The second path has fewer steps. The People page does not
tell which path to use.

## Controls that no journey uses

The inventories list the controls that no main journey reaches:

- Inventory A, section 16.
- Inventory B, section 32.
- Inventory C, the end of the file.

The walks did not examine these lists.
