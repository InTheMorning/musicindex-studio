# Task 041 Review Checklist, 2026-10-27

This record holds the orchestration of [task 041](../tasks/studio-v1-task-041-msp-fields.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) owns the rules.

## Sequence

The packets run one at a time, because packets 041b to 041e all change `crates/api/ui/editor.js`:

1. Task 041a.
2. Task 041b.
3. Task 041c.
4. Task 041d.
5. Task 041e.

A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end.

## Packets

| Packet | Status | Review |
|---|---|---|
| 041a | Complete | Pass on 2026-10-27. The four checks pass. The planning model removed one wrong code comment about the write sequence. |
| 041b | Complete | Pass on 2026-10-27. The four checks pass. The UI has no feed rule. The visual check is open. |
| 041c | Complete | Pass on 2026-10-27. The four checks pass. The planning model changed the noun of the field to "the lyrics file", as the other nouns are. |
| 041d | Complete | Pass on 2026-10-27. The four checks pass. The planning model corrected the lyrics subject and the season number. The visual check is open. |
| 041e | Complete | Pass on 2026-10-27. The four checks pass. The planning model corrected the fill of the image facts and the form of the purpose tokens. The visual check is open. |

## Decisions

### Before The Packets

- The first version of the packets put the form rule of the explicit value in the UI. It also said that the core does not change a value. No decision gave that rule. The operator decided on 2026-10-27 that the core applies the form rule at the draft save. ADR 0064 item 4 and packet 041a give this rule.
- The operator decided on 2026-10-27 that a lyrics file has its own kind in the picker. ADR 0064 item 9 gives this rule, and packet 041c adds the kind.
- The operator decided on 2026-10-27 that the page of ADR 0064 item 7 has the name "Feed settings". The album page, the playlist page and the publisher page each have a row that opens it.
- The packets are five: the core elements, the channel fields, the lyrics kind, the track fields and the artwork list.
- The operator accepted ADR 0064 on 2026-10-27, after task 042.
- The operator decided on 2026-10-27 that the work of ADR 0065 comes before task 041. ADR 0065 moves the feed rules of the UI to the core. The operator accepted ADR 0065 on 2026-10-27. Task 042 does its work.
- Packets 041b, 041d and 041e have the constraint "No feed rule in the UI (ADR 0065)". The review of each one examines each new UI function that changes the model.
- The near ratio uses a list of 19 ratios and a tolerance of 2 %. The behavior agrees with MSP 2.0. No code comes from MSP 2.0.

### Packet 041a

- Packet 041a first said that the sequence of the write calls gives the position of a new element with no hosted position. `Children` has no such function. ADR 0031 does not use a fixed sequence of elements. The coding model stopped on the escalation condition. The planning model removed the rule on 2026-10-27. A new element with no hosted position goes before the first element of a type that must come after it (ADR 0031 item 3).
- The operator decided on 2026-10-27 that the planning model commits each packet after its review passes.

### Packet 041b

- The publisher page has no details section. The coding model stopped on the escalation condition. The operator decided on 2026-10-27 that the publisher page gets a new section directly after its first section. The heading is the role label and the word "details". The section has the row "Feed settings".

### Packet 041c

- The upload route accepts a lyrics file, so the import sheet of the album refuses a lyrics file in the UI. It removes the uploaded file and shows the error of an unsupported file.
- Two tests used a `.txt` file as a file that is not media. They use a `.bin` file after this packet, because a `.txt` file is a lyrics file (ADR 0064 item 9).
- The coding model saw two failures of `testing::sshd::tests` in parallel runs, each in a different test. A second run passed each time. The two runs of the planning model passed. The cause is not known.

### Packet 041d

- Before the packet started, the planning model found two problems in the packet. `Persons` writes the fixed address path `persons`, and `PickedImageField` always shows an image. `Persons` gets an optional property `path`. The lyrics entry uses a new component `PickedFileField` with no image. The packet also lists `crates/api/src/tests.rs` for its API test.
- `replacedAddresses` of `picker.js` gave the track image for a lyrics field. The coding model gave the picker a subject with no image, so the replaced lyrics file stayed on the host. The planning model added a `lyrics` branch that gives the address of the transcript entry, with a Node test. The picker gets the entry as its subject.
- The season number is a text in the core model, not an optional text. A season with only a name sent `null` for the number, and the draft save gave an error. The UI sends an empty text for an empty number. No test covers this function, because it is in `editor.js`. The visual check examines it.
- When the album has no persons, "Credits for this track" adds one blank person. Without it, the empty list selects "Use the album credits" again.

### Packet 041e

- Before the packet started, the planning model added the picker field `artwork` to the packet. `replacedAddresses` reads `subject.image` for an image field, and an artwork entry keeps its address in `href`. After this change, the packet lets the coding model change `picker.js` for this field. The packet also gives the JSON names of the image facts, and it lists `crates/api/src/tests.rs`.
- The purpose check boxes showed the token `publisher` as a plain label. musicindex ADR 0033 R-WORD-4 item 1 does not let interface text use that word. The operator decided on 2026-10-27 that the check boxes are a technical view (R-WORD-4 item 2). Each token shows as an RSS token in code form, also in the summary line. The name of the remove action is "this image".
- A loaded fact with no value wrote `null` over a value that the user typed. The planning model changed the fill, so a fact with no value does not change the entry.
- `readImageFacts` loads an image through `/images`, as `useImage` of `app.js` does. `useImage` can give the result of the address before the change for one render, so the entry does not use it.
