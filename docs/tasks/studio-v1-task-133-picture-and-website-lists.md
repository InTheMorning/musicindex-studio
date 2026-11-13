# Studio v1 Task 133: A Person Keeps A List Of Pictures And Websites

Decision: [ADR 0196](../adr/0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md).

Status: Open. Part 133a is complete on 2026-11-13. Part 133b is open.

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

The main session writes this part after the commit of 133a.

## Stop And Ask When

- A reader of `img` or `href` needs the list and not the default.
- The migration cannot keep the existing data.
- A test of an earlier ADR asserts a rule that ADR 0196 changes, and the change is not in the list above. Report it. Do not delete it without the main session.

## Report

- The changed functions, with file and line.
- The test names.
- The output of the four checks.
