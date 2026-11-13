# ADR 0196: A Person Keeps A List Of Pictures And Websites, With A Default

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-12 by the operator. The operator found the defect in the human check of ADR 0194. The operator asked for competing values with a default, as the wallets have. The operator selected "List, default stays" through the questionnaire, from a drawing.
- Technical choices: items 2, 3, 6, 9, 10, 12, 13 and 14 are choices of the planning model.
- Changes: [ADR 0194](0194-the-nickname-saves-at-once-and-the-update-sheet-selects-each-value.md) items 6 and 7, the button **Use**. [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md) items 1 and 3, the page suggestion of the picture and the website. [ADR 0170](0170-update-people-from-their-linked-pages.md) items 2 and 3, the first page gives the picture and the website. [ADR 0159](0159-a-person-can-link-to-a-page-on-music-index.md), a value that differs shows with "Use" as a pending change.
- Source: the human check of ADR 0194 on 2026-11-12.

## Context

A person has one picture address and one information address (website). A person can link to many pages on Music Index (ADR 0159). Two pages of one artist can give different values. The person "Prosperous Soul" links to a page on Fountain and to a page on Wavlake, and each page has its own picture and website.

`page_suggestions` (`crates/core/src/people.rs`) suggests each page value that differs from the saved value. One **Update** in "Update from Linked Pages" saves the values of one page. Then the other page differs, and the next sheet suggests its values. Each **Update** replaces the values of the last **Update**. The sheet also does not show the page that gives a value, or the value that it replaces.

## Decision

### The Lists

1. A person has a list of pictures and a list of websites. One value of each list is the default. Studio puts the default into a new credit. Studio shows it on the page of the person and in People. Each other place that uses the picture or the website of a person today uses the default.
2. The default stays in `people.img` and `people.href`. A new table `person_values` holds each value of the two lists: `person_id`, `kind` (`picture` or `website`), `value`, `position`, `created_at`, and the source of item 3. The value is unique for one person and one kind.
3. Each value keeps its source. A value from a linked page keeps the feed GUID, the title and the host of the feed address of that page. Each other value has no page source.
4. A migration puts each current `img` and `href` that is not empty into `person_values`, with no page source.
5. The core keeps the default in its list. When a save writes an `img` or an `href` that is not in the list, the core adds it to the list, with no page source.

### The Default Stays

6. Some values come from a linked page, from "Find on Music Index", from a credit suggestion or from "Add All Suggestions". Each of these values goes into the list. It does not change the default. When the list is empty, the new value becomes the default.
7. Only **Make Default** changes the default. **Make Default** is a pending change of the page of a person, as for a wallet (ADR 0116).
8. A page value that is in the list gives no suggestion and no line in "Update from Linked Pages". Thus two pages no longer replace the values of each other.

### The Page Of A Person

9. The page of a person shows a group "Pictures" and a group "Websites" in the place of the fields "Picture address" and "Information address". Each row shows the value, and a picture row also shows a small picture. The second line of a row gives the source: `From "{title}" on {host}` for a page value, and `Added in Studio` for a value with no page source.
10. The default row shows the tag "Default", as a default wallet does. Each row has **Remove**. Each other row also has **Make Default**. **Remove** comes first (ADR 0193). A row has no **Edit**. To change a value, the user adds a new value and removes the old value.
11. Below each group, the button **Add a Picture Address** or **Add a Website** opens a field. The new value follows item 6.
12. **Remove** of the default makes the first remaining row the default. A remove and a **Make Default** are pending changes until **Save**.
13. The page suggestion of a picture and of a website shows **Add** and the source line of item 9. Its accessible name is "Add {value}". **Add** follows item 6.

### The Update Sheet

14. "Update from Linked Pages" shows each new value with the source line of item 9. One page gives at most one picture and one website. A value that two pages give shows one time, with the first page as its source. **Update** adds each selected value to its list and follows item 6.

### Other Places

15. "Merge with a Person" moves each value of the other person into the lists of the kept person, after the values of the kept person. The default of the kept person does not change.
16. The export of the people gives the default in `img` and `href`, as today, and adds `pictures` and `websites`, each with the values and their sources.

## Alternatives Considered

- The newest value becomes the default. Not chosen by the operator: two pages then change the default each time.
- One value, with a select menu for each page in the update sheet. Not chosen by the operator: the person cannot keep the other values.
- A list with no default column, and a default flag on a row. Not chosen: each reader of `img` and `href` must change, and the credits and the change sets carry a risk.

## Consequences

- Two linked pages with different pictures no longer replace each other.
- Each value shows where it came from.
- The page of a person has two more groups of rows.

## How This Decision Is Checked

Mechanical:

- A Rust test: the migration puts the current `img` and `href` into `person_values`.
- A Rust test: two pages with different pictures give two suggestions. After one **Update**, the other page value gives a suggestion, and the first value does not. After the second **Update**, no page gives a suggestion. The default stays the first saved value.
- A Rust test: a value added to an empty list becomes the default. A value added to a list with a default does not change the default.
- A Rust test: a save with `img` not in the list adds it to the list.
- A Rust test: a merge moves the values and keeps the default of the kept person.
- A Node test: the pending state of the page gives the lists, **Make Default** and **Remove** as pending keys, and the save body sends them.
- A Node test: the page suggestion gives **Add**, and a value in the list gives no suggestion.
- The four commit checks pass.

Visual, by the planning model, with screenshots at 1280 px and 390 px in both themes:

- The page of a person with two pictures from two pages and one website: the source lines, the tag "Default", **Remove** and **Make Default**. `$edges()` and `$overflow()` are empty.
- "Update from Linked Pages" with the source line of each value.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, open "Update from Linked Pages" with Prosperous Soul and select **Update** two times. The second sheet does not show the values of the first sheet again. The page of Prosperous Soul lists the values of both pages with their sources, and the default did not change.
