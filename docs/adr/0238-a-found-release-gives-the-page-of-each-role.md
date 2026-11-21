# ADR 0238: A Found Release Gives The Page Of Each Role

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator. The operator selected item 1 and the artist values of item 4 from drawings in the questionnaire. The operator then asked in the chat that the Label and Hosting forms also make their pages from each kind of result. The operator stated that the hosting role is a user who hosts other artists and labels, not a hosting service.
- Technical choices: the main session selected items 2, 3, 5 to 9, and the label and hosting values of item 4. The operator did not receive them as a question.
- Bar: crosses modules. It changes the copy preview, the make of a feed, the link of a page and the UI. Without this record, an agent can keep ADR 0233 items 6 to 9 for a release in a page form.
- Changes: [ADR 0233](0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) items 6, 7, 8 and 9, for a found feed that is not a page, in the Artist, Label and Hosting forms.

## Context

On 2026-11-21, the operator imported a release from Music Index. Review refused the write with "No artist page". The operator opened the Artist form and selected that release in "Find on Music Index". Studio made a release, not an artist page.

ADR 0233 item 2 shows a result with no kind in each form, so each page form shows releases. A release selection then follows ADR 0233 items 6 to 9 (`crates/api/ui/app.js`, `AddFindSheet`):

- A release on a connected file host ("Yours") is imported as it is, as a release.
- A release in the collection ("In Studio") opens.
- A different release is copied: its title becomes the page name, and its tracks go into the new page.

No step reads the artist, the label or the host of the release. The role rows of the search are publisher feeds only (`crates/core/src/lookup.rs`). When Music Index has no page for that artist, label or host, the search has no row for it.

## Decision

In this record, a page form is the Artist, Label or Hosting form. A found feed is a feed result of the search that is not a publisher feed: a release, a show or a playlist.

1. In a page form, a selected found feed makes a new page of the role of the form. The page gets the values of that feed. Studio does not import, open or copy the found feed, also when the result shows "Yours" or "In Studio".
2. Studio reads the feed file of the found feed, as ADR 0233 item 8 does. The found feed can name pages in `podcast:publisher`, and Music Index can give a page for it (`publisher_feed_guid`). The page of Music Index is an artist page when its title is the artist of the feed. Each page with the role of the form, or with no role, shows first in a list with the back row **‹ Results**. A selection of that page follows ADR 0233 items 6 to 12. The last row is "Make a new {role} page from {title}".
3. A result that is a publisher feed keeps ADR 0233 items 6 to 12. The new page gets the role of the form.
4. The new page gets these start values from the found feed. The user can edit each value before the make.

   | Form | Name | Website | Payment recipients |
   | --- | --- | --- | --- |
   | Artist | The artist name (`itunes:author`) | `link` | The recipients of the channel |
   | Label | Empty | None | None |
   | Hosting | Empty | None | None |

   The hosting role is a person who hosts the feeds of other artists and labels, not a hosting service. `itunes:owner` is frequently the hosting service, for example "Wavlake", so it gives no start value. A feed does not name its label. The new page gets no picture, no description and no tracks.
5. The form shows the line "Takes the {role} of {title}. Links {title} to this page." With payment recipients, it also shows "The payments go to the recipients of {title}. Examine them on **Value Routes** before the write." When the name is empty, the Label form shows "{title} does not name its label. Type the name." The Hosting form shows "Type the name of the person who hosts {title}." Review shows each copied value before the first write.
6. The new page links the found feed. When the found feed is in the collection, Studio links the two drafts (ADR 0041 items 6 to 8). For a found feed that is not in the collection, the new page gets a `podcast:remoteItem` of that feed. It has the GUID, the address, the title and the medium of the feed.
7. When Studio cannot read the feed file, the start values come from Music Index. The Artist form gets the artist name. The other forms get no value. The form says: "Studio could not read the feed file of {title}. Only the name is copied." When Music Index gives the GUID and the address of the feed, item 6 applies.
8. The form that "Make an Artist Page…" of Review opens follows items 1 to 7.
9. The Show and Playlist forms keep ADR 0233 items 6 to 12 for each result.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0233 items 6 and 7 | Item 1: in a page form, "Yours" and "In Studio" do not import or open a found feed. |
| ADR 0233 items 8 and 9 | Items 2 to 7: in a page form, a found feed gives the start values of its role, and no tracks. |

## Consequences

- A release with no artist, label or hosting page on Music Index can start each of these pages in one selection.
- A found feed in a page form does not do the same thing as in the Release form. The line of item 5 tells the user before the make.
- A feed does not name its label or its host. The Label and Hosting forms ask for the name, and keep the link of item 6.
