# Studio v1 Task 133: A Person Keeps A List Of Pictures And Websites

Decision: [ADR 0196](../adr/0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md).

Status: Complete on 2026-11-13. The human check of ADR 0196 is open.

Part 133b: the mechanical checks passed, with 752 Node tests. The main session restored the import of `hasLinkedPages`, because the People page uses it and no test found its removal. The main session also removed the `.form` wrapper of the add form, because the field touched the edge of the group.

The walk `steps-196-page-{light,dark}.json` uses `setup-196.py`. After one **Update**, the page of Prosperous Soul showed two pictures and two websites, each with its source line. The first page value has the tag "Default" and **Remove**. The other value has **Remove** and then **Make Default**. **Make Default** moved the tag, marked the row as pending and showed "1 change not saved". At 1280 px light and 390 px dark, `$edges()` and `$overflow()` are empty on the groups and on the open form of **Add a Website**.

Part 133a: the mechanical checks passed. The main session added the skip of a shared value in a merge, because the `UNIQUE` constraint stopped the merge. The walk `setup-196.py` gives one person with two linked pages. At 1280 px light and 390 px dark, the first sheet showed two pictures and two websites, each with its host. After **Update**, the second sheet showed "The linked pages have no new values for your people." The database kept the first page value as the default and both values of each kind in `person_values`. `$edges()` and `$overflow()` are empty.

## Goal

A person has a list of pictures and a list of websites, each value with its source. `people.img` and `people.href` keep the default. A page value that is in the list gives no suggestion. Thus two linked pages no longer replace each other's values.

## Rules For The Delegate

- Read ADR 0196 first. The code locations are in `$TMPDIR/locations-196.md`, if it exists. Otherwise find them.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do only the part that the main session names.

## Part 133a: The Database, The Core And The API

1. Migration (`crates/core/src/collection.rs`, append to `MIGRATIONS`): table `person_values` with `id INTEGER PRIMARY KEY`, `person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE`, `kind TEXT NOT NULL` (`picture` or `website`), `value TEXT NOT NULL`, `source_guid TEXT`, `source_title TEXT`, `source_host TEXT`, `position INTEGER NOT NULL`, `created_at INTEGER NOT NULL`, and `UNIQUE(person_id, kind, value)`. In the same migration, insert each `people.img` and `people.href` that is not empty, with position 0 and no source. Add a migration test like the newest one.
2. Core types (`crates/core/src/people.rs`):
   - `ValueSource { feed_guid, title, host }` and `PersonValue { value: String, source: Option<ValueSource> }`, with Serialize and Deserialize.
   - `Person` gets `pictures: Vec<PersonValue>` and `websites: Vec<PersonValue>`, ordered by `position, id`. `read_person` fills them.
3. Core rules:
   - A helper adds a value to a list: it skips a value that is in the list, it appends at the next position, and it sets `img` or `href` to the value when the default is empty (ADR 0196 items 5 and 6).
   - `update_fields`: after a write, when `img` or `href` is not empty and not in its list, add it with no source (item 5).
   - `PersonSave` gets `pictures: Option<Vec<PersonValue>>` and `websites: Option<Vec<PersonValue>>`. When present, the save replaces the list in that order. Then it applies item 5 to the default. When the default is no longer in a replaced list, the first value becomes the default, or the default becomes empty (item 12).
   - `page_suggestions` (about line 3353): suggest `picture` and `website` only when the page value is not in the list (item 8). Add `host: Option<String>` to `PageSuggestion`, from the host of `values.feed_url`.
   - `PageSuggestionChanges`: replace `img` and `href` with `pictures: Vec<PersonValue>` and `websites: Vec<PersonValue>`, each with the page source. A value that two pages give shows one time, with the first page (item 14). `page_suggestion_changes_batch` keeps a person with one or more values.
   - `save_page_suggestion_changes`: add each value with the helper. It does not change a default that is not empty (item 6).
   - `merge_people`: move each value of the other person to the kept person, after its values, and skip a duplicate (item 15).
   - `detail_suggestions` and `join_details`: leave out each value that is in the list, not only the default.
   - `lookup.rs` `compose_values`: the state is `Current` when the value is in the list.
   - Export (`ExportPerson`, about line 3812): keep `img` and `href`, and add `pictures` and `websites` (item 16). The format version stays `1`, because the change only adds fields.
4. API (`crates/api/src/people.rs`): the person JSON gives `pictures` and `websites`. `POST /people/{id}/save` accepts them. `GET` and `POST /people/linked-page-changes` use the new `PageSuggestionChanges`.
5. UI adaptation, only so the app keeps working. Do not build the groups of item 9 (part 133b):
   - `crates/api/ui/people.js`: `linkedPageChangeLines` gives one line for each value of `pictures` and `websites`, with `source` set to `From "{title}" on {host}`. The selection key stays `"{person_id}:{kind}:{value}"`. `linkedPageBody` keeps only the selected entries of `pictures`, `websites`, `keys` and `wallets`, and leaves out a person with no selected value.
   - `crates/api/ui/peoplepages.js` `LinkedPageReview`: show the `source` line below the address of each picture and website line.
   - `pageSuggestions`: the picture and website rows get `button: 'Add'` and `ariaLabel: 'Add {value}'` (ADR 0196 item 13). Their `line` is `New picture from "{title}" on {host}` and `New website from "{title}" on {host}`. Keep the current click, which sets the field.
   - Update each Node test that this changes.

### Checks Of 133a

Mechanical, in Rust tests in `crates/core/src/people.rs`, `crates/core/src/collection.rs` and `crates/api/src/tests.rs`:

- The migration puts the current `img` and `href` into `person_values`.
- Two pages with different pictures give two suggestions. After the save of the changes of the first page, only the second page gives a suggestion. After the save of the second page, no page gives a suggestion. The default stays the first value.
- A value added to an empty list becomes the default. A value added to a list with a default does not change the default.
- A save with an `img` that is not in the list adds it to the list.
- A save with `pictures` that leaves out the default makes the first value the default.
- A merge moves the values and keeps the default of the kept person.
- Each failure message names ADR 0196 and the item, and gives the fix.

Node tests: `linkedPageChangeLines` gives the source line, `linkedPageBody` keeps only the selected values, and `pageSuggestions` gives **Add** with the source line.

The four checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`. For each changed file in `crates/api/ui/`, copy it to `$TMPDIR/x.mjs` and run `tools/node --check $TMPDIR/x.mjs`.

## Part 133b: The Page Of A Person

Part 133a is commit `ad88d13`. Read its diff (`git show ad88d13 -- crates/api/ui`) first. This part changes only `crates/api/ui/people.js`, `crates/api/ui/peoplepages.js`, the CSS of `crates/api/ui/` if a row needs it, and the Node tests in `crates/api/ui-tests/`. Do not change the core or the API.

1. Pending state (`people.js`, ADR 0196 items 7 and 12):
   - `savedValues` gives `pictures` and `websites`: copies of `person.pictures` and `person.websites` (`{ value, source }`), or `[]`. `img` and `href` stay the defaults.
   - `CONTROLS` gets `pictures` and `websites`. `pendingKeys` gives one `picture` key for each value that is added or removed, and one `website` key for each website that is added or removed. A different order of the same values gives no key. **Make Default** changes `img` or `href`, and the existing `img` or `href` key shows it.
   - `isPending(saved, values, 'pictures')` and `'websites'` are true when the two lists differ.
   - `saveBody`: when a `picture` key exists, `body.pictures = values.pictures`. The same for `website` and `body.websites`. Each entry is `{ value, source }`, as the core reads it. The existing `fields` part sends the default.
   - New exported helpers, each returns new values and does not change its input:
     - `addPersonValue(values, kind, value, source = null)`, `kind` is `'picture'` or `'website'`. It trims the value. It ignores an empty value and a value that is in the list. It appends `{ value, source }`. When the default (`img` or `href`) is empty, the value becomes the default (item 6).
     - `removePersonValue(values, kind, value)`. It removes the value. When the value is the default, the first remaining value becomes the default, or the default becomes `''` (item 12).
     - `personValueLine(source)`: `From "{title}" on {host}` for a source with a host, `From "{title}"` for a source with no host, and `Added in Studio` for no source (item 9). Use `Untitled feed` for an empty title.
2. Page suggestions (item 13): `pageSuggestions` leaves out a picture or a website that is in `values.pictures` or `values.websites`. It no longer compares with `values.img` or `values.href`. Give each row `source: { feed_guid, title, host }` from the suggestion.
3. Other paths that set a picture or a website on the page of a person follow item 6. Each one calls `addPersonValue` and does not set `img` or `href` direct:
   - The **Add** of a page suggestion, with the source of the row.
   - `addAllPageSuggestions`: each picture and each website row, with its source. Thus "Add All Suggestions" adds all of them, not only the first one.
   - The **Add** of a credit suggestion (`suggestions(key)` in `PersonFields`), with no source.
   - The `fill` of the page (Find on Music Index on the page of a person, `fillPatch`): when the patch has `img` or `href`, add it with no source, and keep the line in `sources`.
   - `detailList` callers: pass the list, so that a value that is in the list gives no credit suggestion. The core already leaves out these values; the UI must also leave out a pending value.
4. The groups (`peoplepages.js`, items 9 to 11):
   - In `PersonFields`, remove the fields `person-img` "Picture address" and `person-href` "Information address". In their place put a group with the heading "Pictures" and a group with the heading "Websites". Use the same heading form as the other groups of the section. Keep the page suggestion rows and the credit suggestions below their group, as now.
   - One row for each value, in list order, with the default row first. A picture row starts with `PersonArt` of the value. The first line is the value (`summary mono address-text`). The second line is `personValueLine(source)`.
   - The default row shows the tag of a default wallet: `<span class="default-tag"><${Icon} name="check" />Default</span>`.
   - Buttons, in this order: **Remove** (`button destructive`, aria-label `Remove {value}`), then **Make Default** (`button action`, aria-label `Make Default: {value}`) on each row that is not the default. No **Edit**. Remove has no confirmation, because it is a pending change until **Save**.
   - A row that is not saved, and the default row when the default is pending, gets the class `pending`, as `AddressRow` does.
   - An empty list shows `<p class="empty">No picture yet.</p>` or `No website yet.`.
   - Below each group, an `AddRow` **Add a Picture Address** or **Add a Website**. It opens an inline form in the group: a `Field` (id `person-add-picture` or `person-add-website`, label "Picture address" or "Website address"), then **Cancel** and **Add**. **Add** calls `addPersonValue` with no source and closes the form. **Add** is disabled for an empty value. Enter in the field does the same as **Add**.
5. Keep each other reader of `img` and `href` on the page: the header picture, the credits and the nickname save use the default.

### Checks Of 133b

Mechanical, Node tests in `crates/api/ui-tests/` (each failure message starts with "ADR 0196 item N" and gives the fix):

- `savedValues` gives the lists. `addPersonValue` appends, skips a duplicate, and sets the default only when it is empty. `removePersonValue` of the default makes the first remaining value the default.
- `pendingKeys` gives one `picture` key for an add and one for a remove. **Make Default** gives the `img` key. A new order alone gives no key.
- `saveBody` sends `pictures` with each `{ value, source }` and `fields.img` with the default.
- `pageSuggestions` gives no row for a value in the list, and gives a row for a page value that differs from the default but is not in the list.
- `addAllPageSuggestions` adds two pictures from two pages and keeps the default.
- `personValueLine` gives the three forms.
- A scan of `peoplepages.js`: no field with id `person-img` or `person-href`, and the Remove button comes before the Make Default button in the value row.
- Update each existing test that this changes. Report each one, with the reason.

The four checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`. For each changed file in `crates/api/ui/`, copy it to `$TMPDIR/x.mjs` and run `tools/node --check $TMPDIR/x.mjs`.

Visual, by the main session: screenshots of the page of Prosperous Soul (walk `setup-196.py`) at 1280 px and 390 px in both themes, after one **Update** of the sheet. `$edges()` and `$overflow()` are empty.

## Stop And Ask When

- A reader of `img` or `href` needs the list and not the default.
- The migration cannot keep the existing data.
- A test of an earlier ADR asserts a rule that ADR 0196 changes, and the change is not in the list above. Report it. Do not delete it without the main session.

## Report

- The changed functions, with file and line.
- The test names.
- The output of the four checks.
