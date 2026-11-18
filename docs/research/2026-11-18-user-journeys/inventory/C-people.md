<!-- Recorded output of ChatGPT inventory run C at commit a64637d. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

**Run C: People inventory**

The checked commit is `a64637d38f70bde5920b49880bd7c7d308246430`. No files were changed. No repository-changing git commands were run.

All JavaScript references below are under `crates/api/ui/`, unless another path is given.

Primary sources: [peoplepages.js](../../../../crates/api/ui/peoplepages.js), [people.js](../../../../crates/api/ui/people.js), [editor.js](../../../../crates/api/ui/editor.js), [values.js](../../../../crates/api/ui/values.js), [app.js](../../../../crates/api/ui/app.js), and [shell.js](../../../../crates/api/ui/shell.js).

**Reading and counting rules**

- Each numbered action below is one step.
- One field entry counts as one step, independent of its character count.
- One menu selection counts as one step. Opening a separate action menu counts as another step.
- Automatic requests, rendering, scrolling, and waits count as zero steps.
- Counts start on the named page. They exclude earlier navigation unless stated.
- **INFERENCE:** All totals are calculated from the handlers. They are not recorded browser measurements.
- A closed disclosure adds one click before its contents can be used.
- Native browser dialogs can have browser-specific text. Those texts are marked **UNSURE**.
- Shared failures and confirmations apply wherever referenced. They are listed after the journeys.
- “No confirmation” means the relevant handler does not call `askConfirm`.
- API messages can contain server-generated text. Their exact runtime text is marked **UNSURE** where the scoped UI only renders `${error.message}`.

**1. Open People**

Entry points:

| Starting place | Exact control | Evidence |
|---|---|---|
| Wide shell, including an open feed | “People” in the left panel footer | `shell.js:276`, `app.js:3339` |
| Narrow root page | “People” in the footer | `app.js:3346`, `app.js:3352` |
| Missing change-set notice | “Open People” | `app.js:3095` |
| Navigation history | “Back” or “Forward”, when that history entry is People | `app.js:1972`, `app.js:1976`, `shell.js:752` |
| Restored session navigation | Automatic restoration of a saved `people` view | `app.js:3653` |

Main path from the wide shell:

1. Click “People” (`shell.js:276`, renderer `app.js:3339`). The People page appears (`app.js:3979`).

Automatic actions: load people, suggestions, change sets, and hidden suggestions. Then load recipients and refresh their lookups (`peoplepages.js:139`).

Branches:

- `view.name === 'people'`: selecting the current “People” destination does not navigate (`shell.js:279`, `app.js:3340`).
- `useNarrow()` uses `(max-width: 760px)` (`app.js:1844`).
- From another narrow page, “Destinations” opens the root page. Then “People” opens People (`app.js:1969`, `app.js:3352`).
- The narrow title button also opens Destinations. Its accessible text includes “, Destinations” (`app.js:1979`).
- `assetMode().server`: the page includes “The operator of this server can read the people that you keep.” (`peoplepages.js:170`, `people.js:35`).
- Local locked-vault startup shows Unlock before the shell. Server startup can show the shell with a locked vault (`app.js:5384`).
- No People-specific desktop menu or shortcut appears in the scoped UI.

Failures: shared API error notice. Initial pending text is “Loading the people…” (`peoplepages.js:227`).

End state: the People page stays open. There is no success message.

Counts:

| Branch | Steps |
|---|---:|
| Wide footer, narrow root footer, or missing-change-set notice | 1 |
| Another narrow page → Destinations → People | 2 |
| Automatic session restoration | 0 |

**2. Find a person and open the person page**

Entry points:

- “Search people, addresses and keys” on People (`peoplepages.js:174`).
- A person row, rendered as `${row.title}` (`peoplepages.js:203`).
- A publisher page’s People row, rendered as `${m.name}` (`editor.js:3918`).
- “Open” under “Already in People” in Add a Person (`peoplepages.js:359`).
- Completion of Add a Person, Check suggestion, or Add to a Person (`peoplepages.js:383`, `peoplepages.js:296`, `peoplepages.js:1610`).
- History navigation, including replacement of a missing merged record (`peoplepages.js:979`).

Main path from People:

1. Type into “Search people, addresses and keys” (`peoplepages.js:174`). The People and payee lists filter.
2. Click `${row.title}` in “People” (`peoplepages.js:203`). The person page appears (`app.js:3982`).

The row title is `displayName(person)`: the trimmed nickname, when present, otherwise `person.name` (`people.js:125`, `people.js:129`).

Search conditions:

- Search matches `person.name`, wallet addresses, and Nostr keys (`people.js:310`).
- Matching ignores case and trims the query (`people.js:296`).
- This filter does not search `nickname` or `credit_names` (`people.js:311`).
- The separate Add a Person duplicate list does search those values. See job 3.
- Search filters payees by name, address, and member addresses (`people.js:316`).
- Suggestions and hidden suggestions are passed through without this filter (`peoplepages.js:191`, `peoplepages.js:210`).
- A nonempty query forces the People and payee disclosures open (`peoplepages.js:200`, `peoplepages.js:220`).

Other paths:

- Publisher page: click `${m.name}` in its “People” section. The person page appears (`editor.js:3920`).
- Add a Person duplicate row: click “Open”. The add sheet closes, and the existing person page appears (`peoplepages.js:361`).
- A missing person with HTTP 404 redirects to its kept merge record, when recorded in the page-load map. Otherwise it redirects to People (`peoplepages.js:980`, `people.js:419`).

Failures and empty states:

- “No person matches the search.” (`peoplepages.js:208`).
- “No people yet. Check a suggestion above, or add a person.” (`people.js:32`, rendered `peoplepages.js:207`).
- “Loading the person…” while the person request is pending (`peoplepages.js:984`).
- Other request failures use `ErrorNotice` (`peoplepages.js:984`).

End state: the person page shows the saved record and its sections. No result message appears.

Counts:

| Branch | Steps |
|---|---:|
| Search and open from People | 2 |
| Open an already visible person row | 1 |
| Publisher People row → person | 1 |
| Add sheet duplicate “Open” → person | 1 |
| Closed People disclosure before selection | Add 1 |

**3. Add a person manually or from a page**

Entry point: “Add a Person” on People (`peoplepages.js:177`).

Main path from People:

1. Click “Add a Person” (`peoplepages.js:177`). The “Add a Person” sheet appears (`peoplepages.js:395`).
2. Type “Name” (`peoplepages.js:388`). The form contains the name.
3. Click “Add the Person” (`peoplepages.js:401`). Studio creates the person and opens its page (`peoplepages.js:381`).

Optional fields, each one step:

- “Picture address” (`peoplepages.js:389`).
- “Information address” (`peoplepages.js:390`).

Own-page branch:

1. Open “Your artist and label pages”, if closed (`peoplepages.js:399`, `pickers.js:206`).
2. Click `${label}` (`peoplepages.js:356`). A “Page to link” row appears (`peoplepages.js:392`).
3. Continue with “Add the Person”.

Selecting an own-page row sets `pendingLink`. That handler does not fill Name, Picture address, or Information address (`peoplepages.js:356`).

Music Index branch:

1. Click “Find on Music Index” (`peoplepages.js:398`). The shared search sheet appears.
2. Type “Name”, if the current start text is insufficient (`pickers.js:315`).
3. Click a result `${row.title || 'Untitled feed'}` (`pickers.js:270`, `peoplepages.js:751`).
4. If several choices appear, select `${option.title}` (`pickers.js:321`).

Automatic result: the add form receives name, picture address, information address, and a pending page link (`peoplepages.js:402`).

Then click “Add the Person” (`peoplepages.js:401`).

Duplicate branch:

- Name text of at least two characters enables the duplicate search.
- It matches existing names, nicknames, and credit names by case-insensitive substring (`people.js:135`).
- Matches appear under “Already in People” (`peoplepages.js:399`).
- “Open” opens the existing person. It does not create a person (`peoplepages.js:361`).
- A duplicate match does not disable creation. Only an empty trimmed name disables the main button (`people.js:143`).

Pending-link controls:

- “Remove” removes the pending page link (`peoplepages.js:394`).
- “Cancel” or “Close” exits the add sheet (`peoplepages.js:396`, `peoplepages.js:400`).

Failures:

- Shared API errors appear inside the add sheet (`peoplepages.js:397`).
- Selecting a release in this lookup yields: “A release has no values for a person. Select an artist or label page.” (`peoplepages.js:403`).
- The create request and optional link request are separate awaited calls (`peoplepages.js:381`).
- **INFERENCE:** If creation succeeds and linking fails, the error can appear after the person has already been created.

End state: the new person page opens. No success message remains. This path does not call the person page’s credit-offer sequence.

Counts:

| Branch | Steps from People |
|---|---:|
| Name only | 3 |
| Name plus both optional addresses | 5 |
| Name plus an already visible own-page row | 4 |
| Name plus own-page row in a closed disclosure | 5 |
| Name → Music Index → one result → create | 5 |
| Additional lookup query entry | Add 1 |
| Additional result choice | Add 1 |
| Open an existing duplicate after entering Name | 3 |

**4. Change the person’s name or usual role**

Entry point: the “Credits” section of a person page (`peoplepages.js:1080`).

Name path:

1. Type “Name” (`peoplepages.js:475`). The change stays pending.
2. Click “Save” (`peoplepages.js:1132`). Studio sends the pending person changes and reloads the record (`peoplepages.js:931`).

Usual-role path:

1. Pick a value from “Usual role” (`peoplepages.js:476`). The change stays pending.
2. Click “Save” (`peoplepages.js:1132`).

Other-role branch:

1. Pick “Other role…” (`roles.js:7`, `app.js:1371`).
2. Type “Other role” (`app.js:1376`).
3. Optionally pick “Group”, whose placeholder is “No group” (`app.js:1378`).
4. Click “Save”.

The free role is sent when its field receives a change event (`app.js:1377`). The usual role and group count as one pending control (`people.js:1002`).

Pending-state controls:

- “Discard” restores the saved values without a confirmation (`peoplepages.js:909`, `peoplepages.js:1131`).
- “Save” and “Discard” are disabled when there is no pending change or a save is active (`peoplepages.js:1131`).
- Toolbar text is “No changes”, `${n} change not saved`, or `${n} changes not saved` (`people.js:1025`).

Failures:

- Save errors remain in the toolbar status.
- The fallback is “Studio could not save this person” (`peoplepages.js:935`).
- Pending values remain after a save failure (`peoplepages.js:920`).

Confirmation: leaving with pending changes can show the leave confirmation described below.

End state: the person page reloads. The toolbar returns to “No changes”. No separate success notice appears.

Counts:

| Branch | Steps |
|---|---:|
| Change Name | 2 |
| Pick a listed usual role | 2 |
| Enter an Other role | 3 |
| Enter an Other role and choose Group | 4 |
| Discard pending changes | 1 |

**5. Change the nickname**

Entry point: the pencil button with accessible label “Change the Nickname” beside the person heading (`peoplepages.js:1011`).

Main path:

1. Click “Change the Nickname” (`peoplepages.js:1011`). The “Change the Nickname” sheet appears (`peoplepages.js:1175`).
2. Type “Nickname” (`peoplepages.js:1178`).
3. Click “Change” (`peoplepages.js:1183`). Studio patches the person and closes the sheet (`peoplepages.js:1152`).

The hint is:

> “Only Studio shows the nickname. Studio does not put it in a feed. A new credit uses the field Name in Credits.”

Source: `peoplepages.js:1179`.

Branches:

- Empty nickname is submitted through the same path.
- Other pending person fields remain pending through `afterNicknameSave` (`peoplepages.js:1151`, `people.js:1086`).
- Enter submits the form (`peoplepages.js:1177`).
- “Cancel” and “Close” exit without the patch (`peoplepages.js:1176`, `peoplepages.js:1182`).

Failures: `ErrorNotice` inside the nickname sheet (`peoplepages.js:1181`). The sheet stays open after failure (`peoplepages.js:1171`).

End state: the displayed name changes through `displayName`. No result message appears.

Count: **3 steps**. Enter can replace the final button click.

**6. Add, remove, or select pictures and websites**

Entry points in the person’s Credits section:

- “Add a Picture Address” (`peoplepages.js:471`).
- “Add a Website” (`peoplepages.js:471`).
- “Add” on a picture or website suggestion (`peoplepages.js:431`).
- “Make Default” on a nondefault value (`peoplepages.js:464`).
- “Remove” on a saved or pending value (`peoplepages.js:460`).
- Linked-page suggestions and “Add All Suggestions”. See job 12.

Manual picture path:

1. Click “Add a Picture Address” (`peoplepages.js:471`). An inline form appears.
2. Type “Picture address” (`peoplepages.js:447`, `peoplepages.js:467`).
3. Click “Add” (`peoplepages.js:470`). The form closes, and a pending row appears.
4. Click “Save” (`peoplepages.js:1132`).

Website path: the same four actions use “Add a Website” and “Website address” (`peoplepages.js:447`, `peoplepages.js:471`).

Branches:

- Enter can replace the inline “Add” click (`peoplepages.js:468`).
- Inline “Cancel” closes the form (`peoplepages.js:469`).
- “Add” is disabled for blank trimmed text (`peoplepages.js:470`).
- Duplicate values return the unchanged model (`people.js:1063`).
- Adding the first value also sets its default field. Adding later values preserves the current default (`people.js:1064`).
- “Make Default” changes the pending default (`peoplepages.js:464`).
- “Remove” removes the pending value. Removing the default selects the first remaining value, or empty text (`people.js:1069`).
- The default row shows “Default” (`peoplepages.js:463`).
- Empty lists show “No picture yet.” or “No website yet.” (`peoplepages.js:480`, `peoplepages.js:483`).
- Invalid or unreadable pictures fall back to initials (`peoplepages.js:47`).

Suggestion path:

1. Open `${list.collapsedLine}`, if the current default field is filled (`peoplepages.js:435`).
2. Click “Add” for `${row.value}` (`peoplepages.js:431`).
3. Click “Save”.

With no current default, suggestions appear directly under “Suggestions for the picture” or “Suggestions for the information address” (`people.js:1356`, `people.js:1404`).

Sources can name a Nostr profile, a credit, or a feed (`people.js:1370`). Added rows retain their source line (`peoplepages.js:459`).

Failures: person-save failure in the toolbar. Failed detail-suggestion reads silently set `detail` to null (`peoplepages.js:872`).

Confirmation: none for Add, Remove, or Make Default. Leaving pending changes uses the leave confirmation.

End state: saved rows remain in their lists. The default appears first. No success notice remains.

Counts:

| Branch | Steps |
|---|---:|
| Manually add picture or website | 4 |
| Add a visible suggestion | 2 |
| Add a collapsed suggestion | 3 |
| Make Default | 2 |
| Remove a value | 2 |

**7. Add, edit, remove, or select a payment wallet**

Entry points:

- “Add a Wallet” in Payment (`peoplepages.js:1079`).
- “Actions” on an address, then “Edit” (`addressrow.js:18`, `people.js:15`).
- “Remove Wallet” (`peoplepages.js:1060`).
- “Make Default” in an address action menu (`people.js:14`).
- “Add to Wallet” on a profile-address suggestion (`peoplepages.js:1068`).
- “Add” on a linked-page wallet suggestion (`peoplepages.js:1074`).

Main LNURL path:

1. Click “Add a Wallet” (`peoplepages.js:1079`). The “Add a Wallet” sheet appears (`people.js:848`).
2. Type “Address” (`people.js:854`, renderer `peoplepages.js:535`).
3. Click “Add the Wallet” (`people.js:849`, renderer `peoplepages.js:554`). The sheet closes, and the person reloads.

The initial Type is “LNURL Address” (`people.js:853`).

Node branch:

1. Pick “Node Address” in “Type” (`people.js:853`).
2. Type “Address”.
3. Optionally type “Custom key” (`people.js:855`).
4. Optionally type “Account” (`people.js:856`).
5. Click “Add the Wallet”.

Only node type shows Custom key and Account (`people.js:884`). Address is the required field (`people.js:854`, `people.js:944`).

Existing-payee branch:

1. Open “Payees with no person”, if closed (`peoplepages.js:549`).
2. Click `${row.title}` (`peoplepages.js:541`). The form receives the wallet values.
3. Click “Add the Wallet”.

Music Index branch: “Find on Music Index” fills the form. It does not save the wallet (`peoplepages.js:548`, `peoplepages.js:551`).

Edit path:

1. Click “Actions” for the address (`addressrow.js:18`).
2. Pick “Edit” (`people.js:15`, handler `peoplepages.js:1056`). “Edit a Wallet” appears.
3. Type “Address”, or change another field.
4. Click “Save the Wallet” (`people.js:851`, `peoplepages.js:554`).

If the wallet has other member addresses, the sheet says:

> `This wallet also holds ${others.join(', ')}. A saved change replaces the wallet and removes ${others.length === 1 ? 'that address' : 'those addresses'} from this person. This changes no feed.`

Source: `people.js:941`.

Default-address path:

1. Click “Actions” (`addressrow.js:18`).
2. Pick “Make Default” (`people.js:14`, `peoplepages.js:1055`). The default stays pending.
3. Click “Save” (`peoplepages.js:1132`).

Removal path:

1. Click “Remove Wallet” (`peoplepages.js:1060`). Studio deletes the wallet link and reloads the person (`peoplepages.js:999`).

Profile-address branch:

- “Add to Wallet” posts the suggested address immediately, refreshes suggestions, and reloads the person (`peoplepages.js:1065`).
- “Dismiss” posts a dismissal and reloads (`peoplepages.js:1069`).
- These two handlers do not use `withLeave` (`peoplepages.js:1065`).
- Person reload resets pending form values (`peoplepages.js:903`).

Empty state:

> “No wallet yet. Add a wallet to give this person a payment address.”

Source: `peoplepages.js:1047`.

Failures:

- Wallet-sheet operations show `ErrorNotice` inside the sheet (`peoplepages.js:547`).
- Immediate removal and profile-address failures show the person-page error (`peoplepages.js:893`, `peoplepages.js:1133`).
- Recipient-list failure in Add a Wallet silently produces an empty payee list (`peoplepages.js:516`).
- Profile-address suggestion read failure silently produces no suggestions (`peoplepages.js:864`).

Confirmation: no wallet-specific confirmation. Actions using `withLeave` first ask about pending person edits.

End state: the person page shows the reloaded wallet rows. Immediate changes have no success notice.

Counts:

| Branch | Steps |
|---|---:|
| Add LNURL address manually | 3 |
| Add node address manually | 4 |
| Each optional node field | Add 1 |
| Add from visible payee row | 3 |
| Edit one wallet field | 4 |
| Change default address | 3 |
| Remove wallet, clean person page | 1 |
| Add or dismiss a profile-address suggestion | 1 |

**8. Add, edit, or remove credit identities and Nostr keys**

Credit-name entry points:

- “Add a Credit Name” (`peoplepages.js:1098`).
- “Edit” in “Credited as” (`peoplepages.js:1087`, renderer `peoplepages.js:90`).
- “Remove” in “Credited as” (`peoplepages.js:1089`, renderer `peoplepages.js:89`).

Add credit-name path:

1. Click “Add a Credit Name” (`peoplepages.js:1098`). Its sheet appears (`people.js:860`).
2. Type “Name” (`people.js:865`).
3. Click “Add the Credit Name” (`people.js:861`, `peoplepages.js:554`). The sheet closes, and the person reloads.

Optional additions:

- Type “Information address” (`people.js:866`).
- Click “Nostr key” to open “Find a Nostr profile” (`peoplepages.js:537`).
- Click “Find on Music Index” to fill credit fields (`peoplepages.js:548`, `peoplepages.js:550`).

Edit path:

1. Open “Credited as” (`peoplepages.js:1083`, title model `people.js:668`).
2. Click “Edit” (`peoplepages.js:90`, handler `peoplepages.js:1088`).
3. Type a field.
4. Click “Save the Credit Name” (`people.js:863`, `peoplepages.js:554`).

The edit note is:

> “This changes the credits that Studio counts as this person. It changes no feed.”

Source: `people.js:920`.

Removal path:

1. Open “Credited as”.
2. Click “Remove” for a credit identity (`peoplepages.js:89`).

Automatic actions: delete the credit-name link. If that row also has a key ID, delete that Nostr key link (`peoplepages.js:1090`).

A key without a credit name has its own “Remove” row (`peoplepages.js:1093`).

Add Nostr key path:

1. Click “Add a Nostr Key” (`peoplepages.js:1099`). “Find a Nostr profile” appears.
2. Type “Name, npub or NIP-05 address” (`editor.js:794`).
3. Click “Search” (`editor.js:796`).
4. Click `${profile.name || shortKey(profile.npub)}` in the results (`editor.js:817`, `editor.js:719`).

Automatic actions: the sheet closes, and Studio posts the selected key (`peoplepages.js:1142`, `peoplepages.js:859`).

Branches:

- Person-page Nostr sheets use `offerPeople=${false}`. They do not offer another person’s saved keys (`peoplepages.js:1142`, `peoplepages.js:552`).
- A signed-in session with a pubkey offers “Use My Key” (`editor.js:735`, `editor.js:798`).
- “Find on Music Index” offers a feed’s Nostr key (`editor.js:807`, `editor.js:825`).
- Existing-key sheets first show a key card, “Remove”, and “Change” (`editor.js:782`).
- “Change” reveals the search form (`editor.js:789`).
- Direct person-key rows provide Remove, but no key-edit sheet. Credit-name editing can change its key (`peoplepages.js:1093`, `peoplepages.js:552`).

Failures: Nostr failures are listed in the lookup catalogue below. Link-save failures stay inside the credit-name sheet.

Confirmation: none specific to adding or deleting an identity. The pending-person leave guard applies to the entry actions.

End state: the person page reloads. “Credited as” shows the saved credit identities and keys. No success notice remains.

Counts:

| Branch | Steps |
|---|---:|
| Add credit name with Name only | 3 |
| Add optional Information address | Add 1 |
| Edit a credit identity from a closed fold | 4 |
| Remove a credit identity or standalone key from a closed fold | 2 |
| Add Nostr key through explicit search | 4 |
| Add Nostr key with “Use My Key” | 2 |
| Add Nostr key through Music Index with a typed query and one result | 4 |
| Shared lookup with multiple choices | Add 1 |

**9. Add, edit, or remove support links**

Entry points:

- “Add a Support Link” in the person’s Support links section (`peoplepages.js:1106`).
- “Edit” beside `${l.label || 'Support link'}` (`peoplepages.js:1102`).
- “Remove” beside that link (`peoplepages.js:1103`).

Main path:

1. Click “Add a Support Link” (`peoplepages.js:1106`). Its sheet appears (`people.js:872`).
2. Type “Address” (`people.js:878`).
3. Click “Add the Link” (`people.js:873`, `peoplepages.js:554`). The person reloads.

Optional: type “Label” (`people.js:877`).

Edit path:

1. Click “Edit” (`peoplepages.js:1105`).
2. Type “Address” or “Label”.
3. Click “Save the Link” (`people.js:875`).

Remove path:

1. Click “Remove” (`peoplepages.js:1103`). The link is deleted, and the person reloads.

Music Index branch:

- “Find on Music Index” fills Address from a feed website (`peoplepages.js:492`, `peoplepages.js:550`).
- The no-value text is “This feed has no website.” (`peoplepages.js:492`).

Failures: shared API errors appear inside the link sheet or on the person page.

Confirmation: no link-specific confirmation. The person leave guard applies to entry and removal actions.

End state: the reloaded link row remains, or the deleted row disappears. No result message stays.

Counts: **3** to add Address only, **4** with Label, **3** to edit one field, **1** to remove.

**10. Check suggestions and collect identities into one person**

Entry point: `${row.title}` with the trailing text “Check”, under “Suggestions from your feeds” (`peoplepages.js:191`, `peoplepages.js:195`).

Main new-person path:

1. Click the suggestion row (`peoplepages.js:195`). The sheet title is `Check: ${name}` (`people.js:2059`).
2. Click “Make the Person” (`people.js:2091`, `peoplepages.js:340`). Studio creates the person and opens its page (`peoplepages.js:298`).

The sheet starts with:

- “Person”: “A new person”, or `suggestion.person_id` when supplied (`people.js:2073`).
- “Name”, for a new person (`peoplepages.js:328`).
- “Link to an artist or label page”, with placeholder “None”, for a new person (`peoplepages.js:330`).
- Selectable groups “Credits”, “Payment addresses”, and “Nostr key” (`people.js:2030`).

Each selected identity checkbox is one additional step (`peoplepages.js:318`).

Existing-person path:

1. Open the suggestion.
2. Pick an existing name in “Person”, unless already selected (`peoplepages.js:326`).
3. Click `Add ${n} ${n === 1 ? 'Item' : 'Items'}` (`people.js:1997`, `people.js:2091`).

Studio posts the selected items and opens that person (`peoplepages.js:295`).

Initial-selection conditions:

- Credits and recipients with `same_name === false` start unchecked (`people.js:192`).
- A Nostr key with a profile-name warning starts unchecked (`people.js:194`).
- Profiles arriving later can uncheck keys with that warning (`people.js:201`).
- Items already in the selected person show `Already in ${person.name}` and have no checkbox (`people.js:2023`, `peoplepages.js:316`).
- Existing-person submission requires at least one new selected item.
- New-person submission requires a nonempty trimmed Name (`people.js:2095`).

Duplicate-related text:

> “You have a person with this name. Studio adds the selected items to it.”

Source: `people.js:2063`.

The UI receives the initial matching `person_id`; it does not calculate that match (`people.js:2073`).

Hide path:

1. Open the suggestion.
2. Click “Hide This Suggestion” (`peoplepages.js:339`). Studio posts the dismissal, closes the sheet, and reloads People (`peoplepages.js:302`).

Restore path:

1. Open “Hidden suggestions” (`peoplepages.js:210`).
2. Click “Show Again” (`peoplepages.js:218`). The suggestion becomes available again after reload.

Failures: `ErrorNotice` inside the check sheet (`peoplepages.js:336`). The UI renders server conflict messages there.

Confirmation: none.

End states:

- New or existing person page after collection.
- People page after Hide or Show Again.
- No success message remains.

Counts:

| Branch | Steps from People |
|---|---:|
| New person with initial Name and selections | 2 |
| Existing person already selected | 2 |
| Select another existing person | 3 |
| Each identity checkbox change | Add 1 |
| Change Name or select a linked page | Add 1 each |
| Closed suggestions disclosure | Add 1 |
| Hide a visible suggestion | 2 |
| Show a hidden suggestion from its closed fold | 2 |

**11. Add more matching feed identities or profile addresses**

Entry points on a person page:

- “Add” or “Add Anyway” in “More from your feeds” (`peoplepages.js:1035`).
- “Correct the Feeds” beside a warned key (`peoplepages.js:1041`).
- “Add to Wallet” or “Dismiss” for a profile address (`peoplepages.js:1065`).

More-items path:

1. Click “Add” (`peoplepages.js:1043`). Studio posts the row’s identity items, refreshes More, and reloads the person.

Warning branch:

- A key-name warning changes “Add” to “Add Anyway” (`people.js:2178`).
- “Correct the Feeds” opens the person check page. It does not add the row (`peoplepages.js:1041`).
- The warning text is `The profile name is not "${name}". Check this key.` (`people.js:462`).

Matching-source lines include:

- “The name is the name of this person”
- “The name is a credit name of this person”
- “The Nostr key is a key of this person”
- “The wallet is a wallet of this person”

Source: `people.js:2112`.

The UI receives the matching items from `/people/${id}/more` (`peoplepages.js:856`).

Automatic profile-address lookup runs when the person opens (`peoplepages.js:864`).

Its source note is:

> `From the Nostr profile ${name ? \`${name} (${short})\` : short} · keysend checked`

Source: `people.js:485`.

Failures:

- More, profile-address, and detail-suggestion read failures silently remove their suggestion lists (`peoplepages.js:856`, `peoplepages.js:864`, `peoplepages.js:872`).
- Action failures use the person-page `ErrorNotice` (`peoplepages.js:1133`).

Confirmation: More-item additions use the pending-person leave guard. Profile-address Add and Dismiss do not.

End state: the updated person page remains. No success notice stays.

Counts: **1** for Add, Add Anyway, Add to Wallet, or Dismiss. Correct the Feeds is **1** to reach Check.

**12. Find, link, and fill from Music Index**

Person-page entry points:

- “Find on Music Index” below the profile (`peoplepages.js:1017`).
- “Link to an Artist or Label Page” (`peoplepages.js:1119`).

Both set `finding` to `person.name` (`peoplepages.js:1017`, `peoplepages.js:1119`).

Main Music Index path:

1. Click either entry control. `Find on Music Index: ${name}` appears (`peoplepages.js:724`).
2. Click `${row.title || 'Untitled feed'}` (`peoplepages.js:751`). `From Music Index: ${title}` appears (`peoplepages.js:794`).
3. Click the resulting primary button: “Link”, `Link and Fill ${n} ${n === 1 ? 'Field' : 'Fields'}`, or `Fill ${n} ${n === 1 ? 'Field' : 'Fields'}` (`people.js:2392`, `peoplepages.js:817`).
4. Click “Save” on the person page (`peoplepages.js:1132`).

Automatic actions:

- Search starts on opening when the query is nonempty.
- Later query changes wait 300 ms (`peoplepages.js:701`).
- Result selection requests the page’s person values (`peoplepages.js:773`).
- Selection makes pending changes. It does not save them (`peoplepages.js:790`, `peoplepages.js:911`).

Search-edit branch:

1. Type “Name” in the lookup (`peoplepages.js:727`).
2. Wait for automatic search.
3. Select a result.

Own-collection link branch:

1. Open the search sheet.
2. Open “Your artist and label pages”, if closed (`peoplepages.js:732`).
3. Click `${label}` (`peoplepages.js:721`).
4. Click “Save”.

The own-page selection adds a pending membership link. It can also replace the pending Name through `linkName` (`peoplepages.js:722`, `people.js:2458`).

Value-selection controls:

- “Link this page” checkbox (`peoplepages.js:805`).
- “Name”, “Picture”, “Information address”, “Wallet”, and “Nostr key” rows (`people.js:2308`).
- A closed `${count} other recipients on ${title}` disclosure holds other wallet rows (`people.js:2373`, `peoplepages.js:810`).
- “Open on musicindex.org” opens an external tab (`peoplepages.js:799`).
- “Back” returns to the search and resets its query to `person.name` (`peoplepages.js:816`, `peoplepages.js:1149`).

Default and ownership conditions:

- A changed page title starts selected as Name (`people.js:2312`).
- Picture and information address start selected when the corresponding person field is empty (`people.js:2322`, `people.js:2330`).
- New wallet and key choices are not automatically selected unless already present in the pending additions (`people.js:2348`, `people.js:2366`).
- A value already held by the person is nonselectable.
- A wallet or Nostr key owned by another person is nonselectable (`people.js:2337`, `people.js:2366`).
- An own collection GUID creates a membership link. Another GUID creates a read-only page link (`people.js:1187`).
- Link candidates exclude feeds whose first publisher role is `hosting` (`people.js:1142`).

Visible link-state text:

- “Already linked” (`people.js:2413`).
- “Your page. The details of this person go to this page.” (`people.js:2412`).
- “Read only. Subsequent changes of this page show on this person.” (`people.js:2411`).
- “Only in Studio”, “In the credits of this feed”, or “Not saved yet” for collection links (`people.js:1152`).
- “Read only”, “Host not known”, `On ${host}`, or “Music Index does not know this page now” for external links (`people.js:1171`).

Save can open credit-offer sheets. See job 13.

Linked-page suggestion path:

- Picture and website “Add” makes pending list additions (`peoplepages.js:1082`).
- Key “Add” makes a pending key addition (`peoplepages.js:1097`).
- Wallet “Add” opens a prefilled Add a Wallet sheet (`peoplepages.js:1074`).
- “Add All Suggestions” makes pending additions for pictures, websites, keys, and matching wallets (`peoplepages.js:1117`, `people.js:1232`).
- Other-recipient wallets remain separate suggestions (`people.js:1230`).

Counts:

| Branch | Steps from person page |
|---|---:|
| Initial search text, one result, accept initial choices, Save | 4 |
| Enter another query | Add 1 |
| Each checkbox change | Add 1 |
| Own-page link, visible row, Save | 3 |
| Own-page link in closed disclosure | 4 |
| Remove an existing page link, then Save | 2 |
| Add All Suggestions, then Save | 2 |
| Add one visible picture, website, or key suggestion, then Save | 2 |
| Linked wallet Add → accept prefilled wallet | 2 |

**UNSURE — nested-sheet return timing:** `fill()` clears `picked` but does not explicitly clear `finding` (`peoplepages.js:911`). The search sheet also clears `finding` on close (`peoplepages.js:1146`). If the search is visible again after Fill, “Close” adds one step before Save.

Failures and messages:

- “Searching…” (`people.js:2240`).
- “Reading the page on Music Index…” (`people.js:2399`).
- “No artist or label feed on Music Index has this name.” (`people.js:2233`).
- A runtime `${note.message}` can appear below search (`peoplepages.js:738`).
- Fetch failures use `ErrorNotice` in the relevant sheet (`peoplepages.js:730`, `peoplepages.js:800`).
- After a values-fetch error, Link can remain available when linking is possible (`people.js:2405`).
- “This wallet has no address, so Studio cannot save it.” (`people.js:2294`).
- “The keysend check failed” (`people.js:2298`).
- “This feed gives no Nostr key. Use “Add a Nostr Key” to search the Nostr profiles.” (`people.js:2293`).

End state: the saved person page shows links and values. No success notice remains.

**13. Add or remove a linked person’s credit in publisher feeds**

Entry condition: Save changed `members` on the person page (`peoplepages.js:926`, `peoplepages.js:939`).

This is an automatic continuation of job 12, rather than a separate initial button.

Offer conditions:

- Removed membership plus an existing feed credit creates a removal offer (`people.js:1293`).
- New membership plus a writable feed without the credit creates an add offer (`people.js:1296`).
- A new link to a read-only feed creates no add offer (`people.js:1298`).
- Removal offers come before addition offers (`people.js:1300`).

Add-credit path:

1. Pick “Role”, if no usual role is already supplied (`peoplepages.js:1224`).
2. Click “Review in 1 Feed” (`peoplepages.js:1241`). Studio creates a change set and closes this offer.
3. On the change-set review, click `Write ${n} ${n === 1 ? 'Feed' : 'Feeds'}` (`app.js:3092`, `recipients.js:416`).

The offer title is:

> `Add ${name} to the Credits of ${feed}?`

Source: `people.js:1314`, rendered `peoplepages.js:1200`.

Removal path:

1. Click “Review in 1 Feed” (`peoplepages.js:1259`).
2. Write from the change-set review.

The removal title is:

> `Remove ${name} from the Credits of ${feed}?`

Source: `people.js:1317`.

“Not Now”, “Close”, or closing the dialog advances to the next offer (`peoplepages.js:1198`, `peoplepages.js:1240`).

Final navigation:

- No made change sets: stay on the person page.
- One made change set: open its review.
- More than one: open People, where “Changes to write” lists them.

Sources: `peoplepages.js:953`, `people.js:1308`.

Failures: offer-sheet `ErrorNotice` (`peoplepages.js:1237`, `peoplepages.js:1255`). Change-set write failures follow job 20.

End state: the resulting review remains open with per-feed status. Skipped offers leave the saved person link in Studio.

Counts, excluding the earlier link-and-Save path:

| Branch | Steps |
|---|---:|
| One add offer with usual role, through write | 2 |
| One add offer needing a role | 3 |
| One removal offer, through write | 2 |
| Skip an offer | 1 |
| Several offers | One response per offer, plus each required role selection and later review/write navigation |

**14. Update all people from linked pages**

Entry point: “Update from Linked Pages” on People (`peoplepages.js:178`).

Condition: `hasLinkedPages(data.people)` means at least one person has a nonempty `pages` list (`people.js:1251`).

Main path:

1. Click “Update from Linked Pages” (`peoplepages.js:181`). Studio reads changes and opens its review sheet.
2. Click “Update” (`peoplepages.js:269`). Studio posts selected changes, closes the sheet, and reloads People (`peoplepages.js:252`).

The review lists each `${row.name}` with value checkboxes for “Picture”, “Website”, “Nostr key”, and “Wallet” (`peoplepages.js:261`, `people.js:1263`).

All offered values start selected (`peoplepages.js:245`, `people.js:1259`).

Branches:

- Each checkbox change adds one step.
- No changes: “The linked pages have no new values for your people.” (`peoplepages.js:237`).
- Update is disabled with no selection or while busy (`peoplepages.js:269`).
- “Cancel” and “Close” exit without updating (`peoplepages.js:259`, `peoplepages.js:268`).

Failures:

- The initial fetch failure uses the People-page `ErrorNotice` (`peoplepages.js:178`, `peoplepages.js:227`).
- Update failure uses the sheet’s `ErrorNotice` (`peoplepages.js:254`, `peoplepages.js:268`).

End state: People reloads. No success notice remains.

Count: **2 steps**, plus changed checkboxes.

**15. Merge person records**

Entry point: “Merge with a Person” at the bottom of a person page (`peoplepages.js:1127`).

Main path:

1. Click “Merge with a Person” (`peoplepages.js:1127`). `Merge with ${name}` appears (`people.js:2188`, `peoplepages.js:1294`).
2. Click “Merge the People” (`peoplepages.js:1311`). Studio posts the merge, closes the sheet, and reloads the kept person (`peoplepages.js:1289`, `peoplepages.js:1145`).

The first other person is selected automatically after `/people` loads (`peoplepages.js:1278`).

Selection branch:

1. Pick a different “Person to merge” (`peoplepages.js:1299`).
2. Continue with “Merge the People”.

The other-person menu excludes the current record and sorts by person name (`people.js:1984`).

The sheet shows:

- `${other.name} has` (`peoplepages.js:1301`).
- Wallets, Nostr keys, credit names, and support links (`people.js:2194`).
- “After the merge” (`peoplepages.js:1305`).
- `Name, picture and default address stay as in ${keep.name}` (`people.js:2216`).

Its hint is:

> `Studio moves each item of the selected person to ${name}, then deletes that person. Studio changes no feed.`

Source: `people.js:2189`.

Branches:

- No other person: “No other person yet. Make a second person first.” (`people.js:2190`).
- Selected record without listed links: “This person has no link.” (`peoplepages.js:1302`).
- The merge button is disabled when no other record is available (`peoplepages.js:1311`).
- “Cancel” and “Close” leave without merging (`peoplepages.js:1295`, `peoplepages.js:1310`).
- No multiselect exists. The request body is `{ other: Number(otherId) }` (`people.js:2221`).

Failures: the merge sheet’s `ErrorNotice` (`peoplepages.js:1307`).

Confirmation: the merge sheet is the action surface. There is no additional `askConfirm` call. Pending person edits can trigger the leave confirmation before opening it.

End state: the kept person page reloads. No merge-success notice remains. The deleted ID is mapped to the kept ID for later history navigation (`peoplepages.js:1290`).

Counts:

| Branch | Steps |
|---|---:|
| Merge the initially selected other record | 2 |
| Select another record first | 3 |
| Merge \(n\) records into one | **INFERENCE:** `2 × (n − 1)`, plus each required other-person selection |

Other identity-collection flows are jobs 7, 8, 10, 11, 12, and 18. They add identity links rather than invoking this record-merge endpoint.

**16. Split a person or undo a merge**

No split, unmerge, merge history, or merge undo control is rendered by `peoplepages.js`.

Evidence:

- Merge submits one other ID and keeps an in-memory redirect map (`peoplepages.js:1288`).
- The map is used only to replace navigation to missing records (`peoplepages.js:979`, `people.js:419`).
- Identity removals are separate delete actions (`peoplepages.js:999`, `peoplepages.js:1090`).
- Add a Person is a separate creation action (`peoplepages.js:376`).

There is no main path or step count for a dedicated split/undo job.

**INFERENCE:** A user can separately remove identity links, create another person, and add links to that person. The UI does not identify this sequence as a split.

The desktop Linux Ctrl+Z/Ctrl+Y handler applies only to editable text elements (`crates/desktop-app/src/linux_page.js:55`). It does not call a person-merge undo operation.

**17. Use a person in a feed credit, recipient, support link, or Nostr field**

These pickers fill feed-editing values. Their rows do not navigate to People.

**17a. Add a feed credit from a person**

Entry points:

- “Add a credit” on the feed Credits page (`editor.js:329`, containing page `editor.js:3213`).
- “Add a credit” in a track’s own credits (`editor.js:3348`).

Main path from the relevant editor section:

1. Click “Add a credit” (`editor.js:330`). “Add a Credit” appears (`editor.js:207`).
2. Click `${r.name}` under “Your people” (`editor.js:203`). The credit form fills.
3. Click “Add the Credit” (`editor.js:222`). The sheet closes, and the credit appears in the draft.

The copied values are Name, usual role and group, default picture and information address, and first available Nostr key (`people.js:1815`).

Branches:

- “Your people” can require an opening click (`editor.js:221`, `pickers.js:206`).
- A matching name or Nostr key marks the row “Already in this list” and removes its selection handler (`people.js:1822`, `pickers.js:249`).
- Track credits initially use album credits when `item.persons.length === 0` (`credits.js:5`).
- Pick “Credits for this track” before using the track picker (`editor.js:3343`). This copies album credits, or supplies a blank credit (`editor.js:3305`).
- Pick “Use the album credits” to remove track-specific credits. A confirmation appears (`editor.js:3313`).
- Playlist dispatch does not render the Credits page (`editor.js:5027`).
- Publisher and nonplaylist album dispatch can render it (`editor.js:5022`, `editor.js:5037`).

Failures:

- People loading failure uses `ErrorNotice` in the credit sheet (`editor.js:219`).
- Empty name disables “Add the Credit” (`editor.js:202`).
- Editing an existing credit to a blank name shows “A credit needs a name.” (`credits.js:26`, rendered `editor.js:315`).

End state: the credit is in the feed draft. Automatic draft save follows. The hosted feed is not yet written.

Counts:

- Existing credit section: **3**.
- Closed Your people disclosure: **4**.
- Track initially using album credits: add **1** to select “Credits for this track”.
- Publish the draft: add the Review/write continuation described below.

**17b. Add or change a recipient from a person**

Entry points:

- “Add a recipient” in a payment block (`values.js:209`).
- An existing recipient’s “Actions” → “Change…” (`values.js:195`, `people.js:22`).
- First payment-block entry: “Add recipients” (`values.js:530`).
- Track/show own-recipient entry: `Give This ${noun.charAt(0).toUpperCase() + noun.slice(1)} Its Own Recipients` (`values.js:554`).

Main add path:

1. Click “Add a recipient” (`values.js:209`). “Add a Recipient” appears (`values.js:258`).
2. Click `${r.name}` under “Your people” (`values.js:251`, `values.js:278`). The recipient form fills.
3. Click “Add the Recipient” (`values.js:279`). The sheet closes, and the draft row appears.

Afterward, the user can:

1. Open `${r.name || r.address || 'New recipient'}` (`values.js:184`).
2. Type “Split (shares)” (`values.js:191`).
3. Set “Fee”, if wanted (`values.js:192`).

Person rows require a default address and at least one wallet (`people.js:1790`). The selection copies the person’s Name and default address, with no split (`people.js:1773`).

Change-address path:

1. Open the recipient summary (`values.js:184`).
2. Click “Actions” (`addressrow.js:18`).
3. Pick “Change…” (`people.js:22`). “Change the Address” appears (`values.js:258`).
4. Select a person row.
5. Click “Use This Address” (`values.js:279`).

Branches:

- No block: click “Add recipients” first (`values.js:527`).
- Track/show with no own block: select its own-recipient button first (`values.js:551`).
- Existing recipient identities are nonselectable in the picker (`values.js:245`, `values.js:251`).
- Typed duplicate identity shows “This address is already in this list.” (`values.js:272`).
- Node recipient fields include “Custom key” and “Account” (`values.js:269`).
- “Use the Album Recipients” removes track/show-specific recipients after confirmation (`values.js:544`).
- Album/publisher/playlist Value Routes use the shared payment editor. Track and live-show payments use `TrackPayments` (`values.js:520`, `values.js:540`).

Failures:

- “Loading your people…” while loading (`values.js:274`).
- People/recipient loading failures use the sheet’s `ErrorNotice` (`values.js:273`).
- Duplicate and required-field conditions prevent submission.

End state: recipient values are in the draft. Split is edited on the resulting row. No success notice appears.

Counts:

| Branch | Steps |
|---|---:|
| Add from an existing payment block | 3 |
| First create the block | 4 |
| First give track/show its own recipients | 4 |
| Change an address from a closed recipient row | 5 |
| Fill split after adding the open new row | Add 1 |
| Open an existing row before filling split | Add 1 |
| Closed Your people disclosure | Add 1 |

**17c. Add a feed support link from a person**

Entry point: “Add a support link” on the feed Credits page (`editor.js:354`).

1. Click “Add a support link”. “Add a Support Link” appears (`editor.js:277`).
2. Click `${r.name}` under “Your people” (`editor.js:273`).
3. Click “Add the Support Link” (`editor.js:288`).

Each saved support link of each person supplies one row. A URL already in the feed is marked taken (`people.js:1848`).

Failures: loading error notice (`editor.js:285`). Empty Address disables submission (`editor.js:272`).

End state: the support link appears in the automatically saved draft.

Count: **3**, plus **1** if its disclosure is closed.

**17d. Use a saved person’s Nostr key**

Entry points:

- “Nostr” in Add a Credit (`editor.js:217`).
- “Nostr” in an open credit (`editor.js:321`).
- Feed Nostr rows using `NostrRow` (`editor.js:682`).

1. Click the Nostr row. “Find a Nostr profile” appears (`editor.js:778`).
2. Click `${r.name}` under “Your people” (`editor.js:811`).

If there is an existing key, first click “Change” (`editor.js:789`).

Branches:

- One row appears per saved key, excluding the current field’s key (`people.js:1835`).
- Query text filters these rows (`editor.js:775`).
- Credit contexts offer “Use the profile name” and “Use the profile picture” (`editor.js:801`).
- The boxes initially select missing credit values (`editor.js:752`).
- Person-page identity sheets suppress Your people (`peoplepages.js:1142`).

End state: the sheet closes, and the selected key fills the invoking form or draft (`editor.js:764`).

Count: **2**, or **3** with an existing key card. Add one per checkbox change or required disclosure opening.

**17e. Write the resulting feed draft**

The feed changes above save automatically after 700 ms (`draft.js:100`, `draft.js:275`).

Toolbar states include “Saving…”, “Not saved”, and “Saved, not written” (`editor.js:2274`).

Main continuation:

1. Click “Review” (`editor.js:2301`). “Review changes” appears (`editor.js:2135`).
2. Click “Write to the File Host” (`editor.js:2228`).

This adds **2 steps** to a draft-edit path when no other condition intervenes.

Material branches:

- No connected file host: the review offers “Connect the File Host of This File”, or “Move to a file host” and “Move to This File Host” (`editor.js:2185`).
- No host choices: “No file host is connected yet. Connect a file host in File hosts first.” (`editor.js:2192`).
- Wrong addresses block writing with “Studio cannot write while an address is not correct.” (`editor.js:2165`).
- A signer can require “Connect a Signer” or “Connect a Different Signer” (`editor.js:2213`).
- Changed draft revision offers “Load the Draft Again” (`editor.js:2142`).
- Hosted-file conflict offers “Apply My Edits to the New File” (`editor.js:2154`).
- “Examine the Changes” can refresh the review (`editor.js:2218`).

These are shared feed-write continuations. Their file-host, signer, and conflict subjourneys extend beyond the People-specific scope.

**18. Save a feed recipient to People**

Entry points:

- Existing recipient “Actions” → “Add to People” (`people.js:24`, `values.js:117`).
- “Add to People” beside a payee row inside Add a Recipient / Change the Address (`values.js:254`, `values.js:318`).

Conditions:

- The recipient must have an address.
- That identity must not already belong to People (`values.js:315`, `people.js:24`).

Existing-row path:

1. Open the recipient summary (`values.js:184`).
2. Click “Actions” (`addressrow.js:18`).
3. Pick “Add to People” (`people.js:24`).

Automatic action: POST `/recipients/saved`, then mark the identity as in People (`values.js:118`).

Picker-row path:

1. Click “Add to People” beside the payee (`values.js:318`).

Automatic action: POST the recipient, show “Added”, then hide the button after two seconds (`values.js:299`, `values.js:312`).

The backend handler saves the recipient as a person (`crates/api/src/routes.rs:7037`). The UI does not open the created person page.

Failures: `ErrorNotice` beside the address or picker action (`values.js:126`, `values.js:316`).

Confirmation: none.

End state:

- Existing address menu: Add to People disappears after success.
- Picker row: “Added” appears for two seconds. Then the action disappears.
- The invoking editor or picker remains open.

Counts: **3** from a closed recipient row, **2** from an open row, or **1** beside a picker payee.

**19. Add a payee without a person to a person**

Entry point: a `${row.title}` under “Payees with no person” on People (`peoplepages.js:220`).

Main new-person path:

1. Click the payee row (`peoplepages.js:222`). Its wallet page appears.
2. Click “Add to a Person” (`peoplepages.js:1578`, label `recipients.js:540`).
3. Click “Make the Person” (`people.js:2108`, `peoplepages.js:1629`).

Automatic actions: create the person with the payee title, add its wallet, and open the person page (`peoplepages.js:1606`).

Existing-person branch:

1. In the sheet, pick a saved person from “Person” (`peoplepages.js:1618`).
2. Click “Add the Wallet” (`people.js:2108`).

New-person Name can be changed in “Name” (`peoplepages.js:1620`).

The title is `Add to a Person: ${name}` (`people.js:1994`, `peoplepages.js:1614`).

The hint is:

> “Studio keeps this wallet with the person. Studio changes no feed.”

Source: `people.js:2099`.

Failures:

- People fetch or wallet-add failures show `ErrorNotice` in the sheet (`peoplepages.js:1625`).
- Submission waits for the People list (`peoplepages.js:1629`).
- Empty trimmed Name disables new-person submission (`people.js:2105`).

Confirmation: none.

End state: the selected or new person page opens. No result message remains.

Counts from People: **3** for a new person, **4** for an existing person, plus **1** if the payee disclosure is closed.

**20. See occurrences, check differences, and change feeds**

**20a. See where a person occurs**

Entry point: the “Where this person is” section of the person page (`peoplepages.js:1019`).

1. Click `${entry.title}` (`peoplepages.js:1027`). The disclosure shows places, payment split, and roles (`peoplepages.js:1028`).

Optional:

2. Click `${openLabel(feedOf(entry.id))}` (`peoplepages.js:1030`). The feed opens.

The label function returns `Open the ${titleCase(releaseWord(feed))}` (`people.js:1479`).

Its type selection is:

- Playlist → `playlist`.
- Nonpublisher → `album`.
- Publisher with label role → `label page`.
- Other publisher → `artist page`.

Source: `people.js:1472`.

The section exists only when `where.length > 0` (`peoplepages.js:1019`). One entry uses the shared `name="person-where"` disclosure group (`peoplepages.js:1023`).

Failures: initial person load failures use the person-page notice. There is no separate occurrence-fetch button here.

End state: the occurrence entry remains open, or the feed page appears.

Count: **1** to inspect a feed’s places, **2** to open that feed.

**20b. Check the person against collection feeds**

Entry points:

- “Check your feeds” on the person page (`peoplepages.js:1033`, `people.js:699`).
- “Correct the Feeds” on a warned More row (`peoplepages.js:1041`).

1. Click either entry. “Check your feeds” appears (`peoplepages.js:1366`).

Automatic actions: read the person and `/people/${id}/check`; resolve involved Nostr keys (`peoplepages.js:1356`).

Groups are “Needs a correction”, “Not your default address”, and “Correct” (`people.js:1410`).

Branches:

- “Correct” entries directly open the feed (`peoplepages.js:1385`).
- Other entries expand to show differences (`peoplepages.js:1317`).
- No occurrences: “No feed uses this person.” (`peoplepages.js:1373`).
- Pending state: “Checking the feeds…” (`peoplepages.js:1368`).

Count: **1** to reach the check page.

**20c. Correct all or one feed**

All-feed path from the person page:

1. Click “Check your feeds” (`peoplepages.js:1033`).
2. Click `Correct ${n} ${n === 1 ? 'Feed' : 'Feeds'}` (`people.js:1626`, `peoplepages.js:1372`). `Correct the Feeds: ${name}` appears (`people.js:1627`).
3. Click `Review in ${n} ${n === 1 ? 'Feed' : 'Feeds'}` (`peoplepages.js:1455`, `people.js:239`).
4. Click `Write ${n} ${n === 1 ? 'Feed' : 'Feeds'}` (`app.js:3092`, `recipients.js:416`).

One-feed branch:

1. Open its `${title}` difference entry (`peoplepages.js:1319`).
2. Click `Use ${name}'s Values` (`people.js:1482`, `peoplepages.js:1325`).
3. Continue with Review and Write.

Correction choices:

| Exact label | Condition/source |
|---|---|
| “Key” | At least two person keys, `people.js:1697`, renderer `peoplepages.js:1444` |
| “Nostr key” | Person has a key, `people.js:1687` |
| “Payment address” | Person has a wallet, `people.js:1688` |
| “Picture and information address” | Always available, `people.js:1689` |
| “Role” | Always available, `people.js:1690` |
| `Replace other addresses named "${person.name}"` | Person has a wallet, `people.js:1691` |

Initial choices select available key/payment, details, and role. Other payees start unselected (`people.js:1647`).

A later profile response can select a nonwarned key. If every key has a warning, it clears the initial key correction (`people.js:1658`).

Automatic preview waits 250 ms after selection changes (`peoplepages.js:1424`).

Failures and empty results:

- “No feed changes with these corrections.” (`people.js:1629`).
- “A feed with no change is not in the list.” (`people.js:1628`).
- Preview or submission errors use the correction-sheet notice (`peoplepages.js:1452`).
- Review is disabled with zero selected feed IDs (`peoplepages.js:1455`).

Counts:

| Branch from person page | Steps through write |
|---|---:|
| Correct all with initial choices | 4 |
| Correct one through its expanded difference entry | 5 |
| Each correction checkbox, key selection, or feed checkbox | Add 1 |

**20d. Use another address, rename, add to releases, or remove from feeds**

Entry points are the person’s “Change your feeds” rows (`peoplepages.js:1121`):

| Exact row label | Sheet kind | Source |
|---|---|---|
| “Use This Address” | `use_address` | `people.js:792` |
| “Rename” | `rename` | `people.js:793` |
| “Add to All Releases” | `add_to_releases` | `people.js:794` |
| “Remove from Feeds” | `remove` | `people.js:795` |

All use `OperationSheet` (`peoplepages.js:560`).

Main paths from the person page:

- **Use This Address:** click row → pick “Address” → click Review → click Write.
- **Rename:** click row → type “Name” → click Review → click Write.
- **Add to All Releases:** click row → pick “Artist or label” → click Review → click Write.
- **Remove from Feeds:** click row → click Review → click Write.

Sources: `peoplepages.js:622`, `peoplepages.js:636`, `peoplepages.js:638`, `peoplepages.js:648`, `app.js:3092`.

Branch controls:

- Rename: “Payment name” and “Credit name” start selected (`peoplepages.js:640`, `people.js:811`).
- Remove: “Payee” and “Credit” start selected (`peoplepages.js:642`, `people.js:813`).
- Add: “As a payee”, “Percentage”, “As a credit”, “Support link”, and “Link” (`peoplepages.js:625`).
- Add initially selects Credit. Payee starts selected when wallets exist, with blank Percentage (`people.js:812`).
- Selecting an artist/label fetches `/feeds/${owner}/level`; its release rows start selected (`peoplepages.js:599`, `people.js:778`).
- That release list excludes publisher feed rows (`people.js:782`).
- Every feed checkbox change adds one step (`peoplepages.js:578`).
- Review requires selected feed IDs and suitable fields (`people.js:828`).

Empty states:

- “This artist or label has no release.”
- “No feed uses this person.”

Source: `peoplepages.js:643`.

End state: these actions create drafts and open a change-set review (`peoplepages.js:615`). Writing uses the continuation below.

Counts through write:

| Operation | Main count |
|---|---:|
| Use This Address with an explicit address pick | 4 |
| Use the initially selected address | 3 |
| Rename with a new Name | 4 |
| Add to All Releases using the initial Credit selection | 4 |
| Add with entered Percentage | 5 |
| Remove from Feeds with initial selections | 3 |

**20e. Review, write, retry, or discard a change set**

Entry points:

- Completion of person operations or correction.
- Completion of one credit offer.
- `${row.title}` under “Changes to write” on People (`peoplepages.js:186`).
- History navigation.

The row title is `${changeTitle(set.operation)} · ${who}` (`people.js:400`).

Main write path from the review:

1. Click `Write ${n} ${n === 1 ? 'Feed' : 'Feeds'}` (`app.js:3092`, `recipients.js:416`).

Automatic actions: obtain each diff, sign required events, write each eligible feed, and update per-feed statuses (`app.js:2988`, `app.js:3059`).

The page title is “Review the changes” (`app.js:3090`).

Branches:

- Changed feed entries can expand to show the diff (`app.js:3113`).
- The first changed entry starts open (`recipients.js:343`).
- Unchanged outcomes are static rows (`app.js:3111`).
- One failed feed does not stop the loop (`app.js:3070`, `recipients.js:381`).
- Selecting Write again retries failed statuses (`app.js:3061`, `recipients.js:408`).
- Only changed, unblocked, unwritten feeds qualify (`recipients.js:337`).
- Unlocking triggers a reload of the change set (`app.js:3039`).

Discard path:

1. Click “Discard” (`app.js:3091`).
2. If any feed was written, confirm “Discard the Changes” (`app.js:3081`).
3. Studio deletes the change set and returns to the person, or People for a recipient-only set (`app.js:3083`, `people.js:411`).

Failures:

- No host: “Studio cannot write this feed. Connect a file host for it first.” (`recipients.js:300`).
- Locked vault: “Studio cannot write this feed while the vault is locked. Unlock Studio first.” (`recipients.js:301`).
- Other blocked reason: “Studio cannot write this feed now.” (`recipients.js:306`).
- Hosted-file conflict: “The hosted file changed after the draft started. Studio did not write this feed.” (`app.js:3005`).
- Other write failure: signer text, `${e.message}`, or “Studio did not write this feed.” (`app.js:3006`).
- Missing set notice: “Studio did not find these changes.” followed by “It is written or discarded.” It offers “Open People” (`app.js:3095`).
- Diff-read errors appear inside the expanded feed (`app.js:3118`).

End state: the review stays open. Per-feed statuses remain “Waiting”, “Writing”, “Written”, or “Not written” (`recipients.js:385`). The Write button disappears when no pending writes remain (`app.js:3092`).

Counts:

| Branch | Steps from review |
|---|---:|
| Write eligible feeds | 1 |
| Retry failed feeds | 1 per retry |
| Discard before any write | 1 |
| Discard after at least one write | 2 |
| Missing set → Open People | 1 |

**21. Remove a person**

Entry point: “Delete Person” at the bottom of the person page (`peoplepages.js:1126`).

Main path:

1. Click “Delete Person” (`peoplepages.js:1126`). The delete confirmation appears (`peoplepages.js:1005`).
2. Click “Delete the Person” (`confirms.js:65`). Studio deletes the record and opens People (`peoplepages.js:1006`).

Branches:

- Pending edits first trigger “Leave without saving?”. “Discard Changes” is another required click (`peoplepages.js:971`).
- “Cancel” or “Close” on the delete confirmation cancels deletion.
- Later history navigation to the deleted person redirects to People (`peoplepages.js:980`, `people.js:419`).

Failure: the person-page `ErrorNotice` remains if the delete request fails (`peoplepages.js:1004`, `peoplepages.js:1133`).

End state: People opens. No deletion-success message remains.

Count: **2** on a clean page, **3** with pending edits discarded first.

**22. Export People**

Entry point: “Export People” on People (`peoplepages.js:182`).

1. Click “Export People”. Studio fetches the export and starts a browser download named `studio-people.json` (`peoplepages.js:117`).

Failure: People-page `ErrorNotice` (`peoplepages.js:227`).

End state: People stays open. No persistent result message appears.

Count: **1**. Browser download prompts are **UNSURE**, because the code does not control browser download settings.

**23. Adjacent wallet operations on People’s payee page**

These controls are part of `peoplepages.js`, although they operate on a payee without a person.

Entry: open a row under “Payees with no person” (`peoplepages.js:222`).

Available rows:

- “Change Address”
- “Rename”
- “Use the LNURL Address”
- “Add to All Releases”
- “Remove from Feeds”

Sources: `recipients.js:428`, renderer `peoplepages.js:1580`.

“Use the LNURL Address” appears only when `hasNode(entry)` (`recipients.js:448`). Other than Add, actions are disabled when no uses are loaded (`peoplepages.js:1581`).

Main paths from the wallet page:

| Job | Atomic actions | Steps through write |
|---|---|---:|
| Change Address | Click row → type address → Review → Write | 4 |
| Rename | Click row → type “Name” → Review → Write | 4 |
| Use a known LNURL address | Click row → Review → Write | 3 |
| Enter an LNURL address | Click row → type address → Review → Write | 4 |
| Add to releases | Click row → pick “Artist or label” → type “Percentage” → Review → Write | 5 |
| Remove from feeds | Click row → Review → Write | 3 |

Field sources: `peoplepages.js:1520`, `peoplepages.js:1529`, `peoplepages.js:1531`, `peoplepages.js:1540`. Submission: `peoplepages.js:1554`.

Additional controls:

- “Type”: “Node Address” or “LNURL Address” (`recipients.js:453`).
- “LNURL address” or “Node address key” (`peoplepages.js:1522`).
- “Custom key” and “Account” for node type (`peoplepages.js:1524`).
- Add’s “Recipient” disclosure holds Name and address fields (`peoplepages.js:1544`).
- Individual feed checkboxes (`peoplepages.js:1550`).

LNURL suggestion branch:

- If no LNURL address is known, opening that operation requests suggestions (`peoplepages.js:1483`).
- Pending text: “Studio looks for LNURL addresses of this wallet…” (`peoplepages.js:1532`).
- A suggestion menu can show “LNURL address”; its alternative text field is “Other address” (`peoplepages.js:1533`).
- The hint includes “The address must pay the same wallet. Studio checks it.” (`peoplepages.js:1536`).

Empty/failure states:

- “No feed uses this wallet. Add to All Releases puts it in feeds.” (`peoplepages.js:1584`).
- “No feed uses this wallet.” (`peoplepages.js:1550`).
- “This artist or label has no release.” (`peoplepages.js:1550`).
- Action errors appear inside the operation sheet (`peoplepages.js:1551`).
- LNURL suggestion-request failure becomes an empty suggestion list (`peoplepages.js:1490`).

End state: a change-set review appears. Write and Discard use job 20e.

**Shared lookup journeys**

**Music Index form lookup**

Entry label: “Find on Music Index”.

Call sites:

| Invoking surface | Source | No-value text |
|---|---|---|
| Add a Person | `peoplepages.js:398` | “A release has no values for a person. Select an artist or label page.” |
| Person wallet add/edit | `peoplepages.js:548` | “This feed has no payment address.” |
| Person credit-name add/edit | `peoplepages.js:550` | “Music Index has no values of this feed.” |
| Person support-link add/edit | `peoplepages.js:550` | “This feed has no website.” |
| Add a feed credit | `editor.js:220` | “Music Index has no values of this feed.” |
| Existing feed credit | `editor.js:249` | “Music Index has no values of this feed.” |
| Add feed support link | `editor.js:286` | “This feed has no website.” |
| Change recipient address | `values.js:276` | “This feed has no payment address.” |
| Nostr profile sheet | `editor.js:808` | “This feed has no Nostr key.” |

Shared main path:

1. Click “Find on Music Index”.
2. Type “Name”, unless useful start text already exists (`pickers.js:315`).
3. Click a result (`pickers.js:270`, `pickers.js:273`).
4. If several choices exist, click `${option.title}` (`pickers.js:321`).

One returned choice fills the invoking form immediately. Zero choices show the call-site no-value text (`pickers.js:305`). Multiple choices replace search content with a choice list.

“Back” returns from choices to results. “Cancel” closes the sheet (`pickers.js:324`).

Count from invoking form: **3** with typed query and one result, **2** with useful start text, **4** with multiple choices.

Search groups:

- “Verified role”
- “Verified two-way links”
- “Name only”
- “Feeds” for other feed results

Sources: `people.js:2245`, `pickers.js:272`.

Failures:

- “No feed on Music Index has this name.” (`people.js:2232`).
- `${note.message}` from the API (`pickers.js:266`).
- Request/values errors use `ErrorNotice` (`pickers.js:264`, `pickers.js:318`).

**Existing-credit field fill**

Entry: “Find on Music Index” in an open feed credit (`editor.js:249`).

After a page selection:

- Empty fields fill automatically.
- Different existing values appear under “Values from Music Index”.
- Each “Replace” applies one different value (`editor.js:235`).
- A result note appears under the lookup row (`editor.js:249`).

Sources: `editor.js:240`, `people.js:1895`.

Count: the shared lookup count, plus **1** for each Replace.

**Nostr search**

Entry surfaces are listed in jobs 8 and 17d.

Main path: open sheet → type “Name, npub or NIP-05 address” → “Search” → result (`editor.js:794`).

The search is explicit. Typing alone does not call `/nostr/profiles`; form submission does (`editor.js:768`).

Visible branches and failures:

| Condition | Exact text/control | Source |
|---|---|---|
| Existing key has no returned profile | “Studio found no profile for this key” | `editor.js:786` |
| Search has no profiles | “Studio found no profile.” | `editor.js:815` |
| Collection source failed | “Studio could not read your feeds.” | `editor.js:704` |
| Music Index source failed | “Music Index did not answer.” | `editor.js:705` |
| Relay source failed | “The search relay did not answer.” | `editor.js:706` |
| Other source failed | “A source did not answer.” | `editor.js:814` |
| Result comes from collection | “In your feeds” tag, still selectable | `editor.js:719`, `editor.js:817` |
| Signed-in pubkey exists | “Use My Key” | `editor.js:798` |
| Credit context supplied | “Use the profile name”, “Use the profile picture” | `editor.js:801` |
| Invalid key row | “Not a Nostr key” | `editor.js:697` |

Automatic profile resolution is separate. Rows call POST `/nostr/resolve` when their keys are shown (`peoplepages.js:55`). Failure leaves the short key without an error notice (`peoplepages.js:63`).

A returned profile without a name can show “No Nostr profile found” (`people.js:438`, `people.js:474`).

**Copy recipients from Music Index**

This shared sheet is reachable from the recipient picker, beside its People choices.

1. Click “Add a recipient” (`values.js:209`).
2. Click “Copy from a Feed” (`values.js:277`). “Copy from a Feed on Music Index” appears (`values.js:444`).
3. Type “Name” (`values.js:449`).
4. Click a feed result (`values.js:453`).
5. Select a recipient checkbox (`values.js:338`).
6. Click `Add ${n} ${n === 1 ? 'Recipient' : 'Recipients'}` (`values.js:489`, `people.js:1955`).

Track branch: open “Tracks” if closed, then click `${t.title}` before selecting recipients (`values.js:458`).

Navigation controls:

- “Search results” returns from a feed (`values.js:456`).
- `${feedName}` returns from a track (`values.js:468`).
- `${total} Selected` with “▼” or “▲” shows or hides selected rows (`values.js:482`).
- Selected-row checkboxes remove selections (`values.js:477`).
- “Cancel” and “Close” exit (`values.js:445`, `values.js:487`).

Empty texts:

- “This feed has no recipients.” (`values.js:463`).
- “This track has no recipients.” (`values.js:471`).

The sheet copies recipients with empty splits (`values.js:434`). It does not create person records.

Count: **6** from the editor’s add-recipient control for one feed recipient. Add one per further checkbox or navigation action.

**Confirmations**

All Studio confirmation dialogs use `ConfirmHost` (`app.js:5410`). They have “Close”. The safe button receives initial focus (`app.js:5429`).

| Trigger | Exact title | Exact body | Buttons, in rendered order | Source |
|---|---|---|---|---|
| Delete person | `Delete ${name} from People?` | “Studio deletes this person and its links. Studio changes no feed.” | “Delete the Person”, “Cancel” | `confirms.js:62` |
| Leave dirty person page or guarded identity action | “Leave without saving?” | `${name} has ${changes} that ${keys.length === 1 ? 'is' : 'are'} not saved: ${joinList([...new Set(keys.map((key) => LEAVE_NAMES[key]))])}.` | “Discard Changes”, “Keep Editing” | `confirms.js:68`, `people.js:1109` |
| Discard change set after a write | “Discard these changes?” | “Each feed that you did not write gets back its draft from before the change. A feed that you wrote does not change.” | “Discard the Changes”, “Cancel” | `confirms.js:27` |
| Replace own track credits with album credits | “Remove the credits of this track?” | “The track then uses the credits of the album.” | “Remove the Credits”, “Cancel” | `confirms.js:43` |
| Replace track/show recipients with album recipients | `Remove the recipients of this ${noun}?` | `The ${noun} then uses the recipients of the album.` | “Remove the Recipients”, “Cancel” | `confirms.js:75` |
| Discard a feed draft during the write continuation | “Discard the draft?” | “Studio discards any change that you did not write. The feed file on its file host does not change.” | “Discard the Draft”, “Cancel” | `confirms.js:31` |

Pending-person body names can include “the name”, “the usual role”, “a picture”, “a website”, “the default address”, “the artists and labels”, “a wallet”, and “a Nostr key” (`people.js:1092`).

The browser unload guard sets `event.returnValue = ''` (`peoplepages.js:973`). **UNSURE:** the exact browser warning and buttons depend on the browser.

The credit-offer sheets in job 13 also ask a question. They use “Not Now” and “Review in 1 Feed”, rather than `ConfirmHost`.

**Shared error behavior**

`ErrorNotice` renders `${title(error)}` followed by `${error.message}` (`app.js:441`).

Relevant fixed titles include:

- “No connection to Studio.” for `offline` (`app.js:476`).
- “This search cannot run.” for `query_field` (`app.js:478`).
- “Music Index gave a bad answer.” for `index_answer` (`app.js:479`).
- “Studio did not find the file.” for `not_found` (`app.js:474`).
- “Something went wrong.” for other unmapped errors (`app.js:481`).

Fetch failure creates this exact message:

> “Studio does not answer. Make sure that it is still running.”

Source: `app.js:170`.

These notices contain no dedicated retry button. The invoking action can usually be attempted again. `useAction` clears its prior error when another action starts (`app.js:620`).

**UNSURE:** exact server validation, conflict, storage, lookup, and authorization messages are not fixed in the scoped UI. The code displays the returned `${error.message}` (`app.js:175`).

Vault/session branches apply to API requests:

- First `423 vault_locked` can open the unlock sheet and retry once (`app.js:143`).
- Recent-unlock requirements can do the same (`app.js:144`).
- Cancelling unlock leaves the request error.
- Server `401 session_required` or `session_ended` invokes session-loss handling (`app.js:145`).

**Pages, sheets, dialogs, and navigation records**

All sheets below use modal native `<dialog>` elements. `useSheet` opens them with `showModal()` and closes them with `close()` (`editor.js:1921`).

**INFERENCE:** Escape normally invokes native dialog cancellation. No scoped handler disables that default. Search-field lists can consume Escape first.

| Surface | Render code | Show condition / entry | Return or exit |
|---|---|---|---|
| People page | `peoplepages.js:131`; dispatch `app.js:3979` | `view.name === 'people'` | Back/Forward; another destination |
| Person page | `peoplepages.js:834`; dispatch `app.js:3982` | `view.name === 'person'` | Back/Forward; People destination; dirty leave guard |
| Check your feeds page | `peoplepages.js:1345`; dispatch `app.js:3984` | `view.name === 'check'` | Back/Forward; feed entry; correction sheet |
| Payee wallet page | `peoplepages.js:1561`; dispatch `app.js:3986` | `view.name === 'wallet'` | Back/Forward; People; Add to a Person |
| Review the changes page | `app.js:3014` | `view.name === 'change-set'` | Back/Forward; Discard; Open People on missing set |
| Check suggestion sheet | `peoplepages.js:278` | `suggestion !== null` | Close/Escape; Hide; successful person navigation |
| Add a Person sheet | `peoplepages.js:345` | `adding` | Close, Cancel, Escape, successful navigation |
| Update from Linked Pages sheet | `peoplepages.js:240` | `linkedReview !== null` | Close, Cancel, Escape, successful Update |
| Add/Edit a Wallet sheet | `peoplepages.js:496` | `linking === 'wallets'`, or wallet `editing` | Close/Escape; successful submit |
| Add/Edit a Credit Name sheet | `peoplepages.js:496` | Corresponding `linking` or `editing.kind` | Close/Escape; successful submit |
| Add/Edit a Support Link sheet | `peoplepages.js:496` | Corresponding `linking` or `editing.kind` | Close/Escape; successful submit |
| Person-operation sheet | `peoplepages.js:560` | `sheet !== null` | Close/Escape; Review navigates |
| Find on Music Index: person sheet | `peoplepages.js:677` | `finding !== null && picked === null` | Close, Cancel, Escape; own-page selection |
| From Music Index: values sheet | `peoplepages.js:759` | `picked !== null` | Back; Escape/onClose; Link/Fill |
| Change the Nickname sheet | `peoplepages.js:1162` | `nicknameOpen` | Close, Cancel, Escape, Change |
| Add/remove linked-feed credit offer | `peoplepages.js:1192` | `offers.length > 0` | Close, Not Now, Escape, Review; next offer follows |
| Merge sheet | `peoplepages.js:1265` | `merging` | Close, Cancel, Escape, successful Merge |
| Correction sheet | `peoplepages.js:1397` | `correcting !== null` | Close/Escape; Review navigates |
| Wallet-operation sheet | `peoplepages.js:1463` | Wallet `sheet !== null` | Close/Escape; Review navigates |
| Add to a Person: payee sheet | `peoplepages.js:1593` | `personSheet` | Close, Cancel, Escape, successful navigation |
| Feed Add a Credit sheet | `editor.js:186` | `adding` in Persons | Close/Escape; Add the Credit |
| Feed Add a Support Link sheet | `editor.js:257` | `adding` in Fundings | Close/Escape; Add the Support Link |
| Find a Nostr profile sheet | `editor.js:732` | Invoking `open` state | Close, Cancel, Escape, result choice |
| Add a Recipient / Change the Address sheet | `values.js:221` | Recipient `sheet !== null` | Close/Escape; submit; Copy from a Feed |
| Copy from a Feed on Music Index sheet | `values.js:362` | `copying` | Close, Cancel, Escape, Add Recipients |
| Shared Find on Music Index sheet | `pickers.js:286` | Invoking `finding` | Cancel/Escape; result choice; Back within choices |
| Search-field floating list | `pickers.js:46` | Click or typed input with returned rows/error | Escape, blur, row insertion |
| Confirmation dialog | `app.js:5410` | `askConfirm(entry)` | Action, safe button, Close, Escape |
| Shared feed Review changes sheet | `editor.js:1963`, markup `editor.js:2133` | Toolbar `open === 'review'` | Close/Escape; write; discard |
| Shared Earlier versions sheet | `editor.js:2236` | Toolbar `open === 'versions'` | Close/Escape; Put in the Draft |

The person page’s section order is Profile, linked pages, Credits, occurrences, check, More, Payment, Support links, Change your feeds, and deletion/merge (`pages.js:56`).

**Navigation routes**

People navigation uses view records, rather than a People-specific URL parser:

| View record | Renderer |
|---|---|
| `{ name: 'people', sort, reversed }` | `app.js:3976` |
| `{ name: 'person', id }` | `app.js:3981` |
| `{ name: 'check', id }` | `app.js:3983` |
| `{ name: 'wallet', recipient }` | `app.js:3985` |
| `{ name: 'change-set', id }` | `app.js:3987` |
| `{ name: 'feed', id, page: 'credits' }` | `editor.js:5037` |
| `{ name: 'feed', id, page: 'recipients' }` | `editor.js:5039` |
| `{ name: 'feed', id, page: 'track', track }` | `editor.js:5029` |
| `{ name: 'feed', id, page: 'live', track }` | `editor.js:5033` |

`readNav` restores session storage or starts at Releases (`app.js:3653`). No direct People/person URL entry is parsed there.

The external page link is:

`https://musicindex.org/#feed=${encodeURIComponent(guid)}` (`people.js:2473`).

Narrow navigation stacks are:

- Root → People → Person, Wallet, or Change set.
- Root → People → Person → Check.

Source: `shell.js:809`.

Wide Back/Forward follows actual navigation history. Narrow layout also supplies Destinations and title-button navigation (`app.js:1963`).

**API routes invoked by this scope**

`api()` prefixes these paths with `/api/v1` (`app.js:196`).

| Method and route | Calling code |
|---|---|
| GET `/people` | `peoplepages.js:142`; editor/picker call sites |
| POST `/people` | `peoplepages.js:381`, `peoplepages.js:1608` |
| GET `/people/export` | `peoplepages.js:118` |
| GET `/people/suggestions` | `peoplepages.js:142` |
| GET `/people/suggestions/hidden` | `peoplepages.js:142` |
| POST `/people/suggestions/show` | `peoplepages.js:216` |
| POST `/people/suggestions/dismiss` | `peoplepages.js:303` |
| POST `/people/suggestions/make` | `peoplepages.js:298` |
| GET/POST `/people/linked-page-changes` | `peoplepages.js:179`, `peoplepages.js:252` |
| GET `/people/${id}` | `peoplepages.js:660` |
| PATCH `/people/${id}` | `peoplepages.js:1152` |
| DELETE `/people/${id}` | `peoplepages.js:1006` |
| POST `/people/${id}/save` | `peoplepages.js:931`; add plan `people.js:172` |
| POST `/people/${id}/items` | `peoplepages.js:295`, `peoplepages.js:1043` |
| POST `/people/${id}/keys` | `peoplepages.js:859` |
| DELETE `/people/${id}/keys/${linkId}` | Generic remove `peoplepages.js:999`; paired removal `peoplepages.js:1091` |
| POST `/people/${id}/wallets` | `peoplepages.js:527`, `peoplepages.js:1609` |
| PUT `/people/${id}/wallets/${walletId}` | `peoplepages.js:527` |
| DELETE `/people/${id}/wallets/${linkId}` | `peoplepages.js:999` |
| POST/PUT/DELETE credit-name routes | `peoplepages.js:527`, `peoplepages.js:1090` |
| POST/PUT/DELETE support-link routes | `peoplepages.js:527`, `peoplepages.js:999` |
| GET `/people/${id}/more` | `peoplepages.js:856` |
| GET/POST `/people/${id}/profile-addresses` | `peoplepages.js:864`, `peoplepages.js:1066` |
| POST `/people/${id}/profile-addresses/dismiss` | `peoplepages.js:1071` |
| GET `/people/${id}/detail-suggestions` | `peoplepages.js:872` |
| GET `/people/${id}/page-suggestions` | `peoplepages.js:881` |
| POST `/people/${id}/merge` | `peoplepages.js:1289` |
| GET `/people/${id}/check` | `peoplepages.js:1356` |
| GET `/people/${id}/musicindex?q=...` | `peoplepages.js:718`, `people.js:2227` |
| GET `/people/${id}/musicindex/${encodeURIComponent(row.feed_guid)}` | `peoplepages.js:773` |
| POST `/people/${id}/changes/preview` | `peoplepages.js:608`, `peoplepages.js:1425` |
| POST `/people/${id}/changes` | `peoplepages.js:615`, `peoplepages.js:1217`, `peoplepages.js:1434` |
| GET `/recipients` | `peoplepages.js:153`, `values.js:238` |
| POST `/recipients/saved` | `values.js:118`, `values.js:299` |
| POST `/recipients/uses` | `peoplepages.js:1568` |
| POST `/recipients/lnaddress-suggestions` | `peoplepages.js:1489` |
| GET `/feeds/${id}/members` | `peoplepages.js:945` |
| GET `/feeds/${owner}/level` | `peoplepages.js:599`, `peoplepages.js:1510` |
| GET `/change-sets` | `peoplepages.js:142` |
| POST `/change-sets` | `peoplepages.js:1517` |
| GET/DELETE `/change-sets/${id}` | `app.js:3025`, `app.js:3083` |
| GET `/feeds/${feedId}/diff` | `app.js:3051`, `app.js:2990` |
| POST `/feeds/${feedId}/write` | `app.js:2995` |
| POST `/nostr/resolve` | `peoplepages.js:62` |
| GET `/nostr/profiles?q=${...}${suffix}` | `editor.js:758`, `editor.js:772` |
| GET `/musicindex/search?q=...` | `pickers.js:296` |
| GET `/musicindex/values/${guid}` with optional `?credits=true` | `pickers.js:305` |
| GET `/musicindex/fields?q=...&cursor=...` with optional scope | `pickers.js:61` |
| GET `/musicindex/artist-names?from=...` | `pickers.js:73` |
| GET `/musicindex/feeds/${guid}/recipients` | `values.js:375` |
| GET `/musicindex/feeds/${guid}/tracks/${trackGuid}/recipients` | `values.js:381` |
| GET/PUT `/settings/announcements` | `editor.js:1831` |

The credit-name and support-link route forms are `/people/${personId}/${kind}` and `/people/${personId}/${kind}/${edit.walletId ?? edit.linkId}` (`peoplepages.js:527`).

**Visible controls not consumed by the main paths above**

| Control | Behavior | Evidence |
|---|---|---|
| Sort menu: “A to Z”, “Z to A”, “Newest”, “Oldest” | Changes People order; replaces its navigation record | `people.js:335`, `shell.js:593`, `app.js:3380`, `app.js:3980` |
| `${g.letter}` under “Letters” | Scrolls to the first person of that letter | `app.js:3393` |
| “People”, “Suggestions from your feeds”, “Hidden suggestions”, “Payees with no person” disclosure summaries | Open or close the corresponding list | `peoplepages.js:191`, `peoplepages.js:200`, `peoplepages.js:210`, `peoplepages.js:220` |
| Generic list `${title}` with `${rows.length}` | Opens/closes a picker disclosure | `pickers.js:206` |
| “Copy the Nostr key” accessible icon button | Copies full key; exposes “Copied” for two seconds | `peoplepages.js:98`, `app.js:1927` |
| “Copy the address” accessible icon button | Copies full address | `addressrow.js:17`, `app.js:1938` |
| Address-menu “Copy” | Copies address; silently ignores clipboard failure | `people.js:16`, `peoplepages.js:1057` |
| Person profile `${m.title}` link button | Opens linked collection feed | `peoplepages.js:1013` |
| Search field-list `${row.label}` | Inserts the supplied query token/range | `pickers.js:143`, `pickers.js:84` |
| “Other role…” / “Other role” / “Group” | Supplies a free role and optional group | `app.js:1371` |
| “Tell Podcast Index and send a podping” | Saves announcement setting on change-set review | `editor.js:1841` |
| “Send a podping” | Saves announcement choice on feed Review | `editor.js:1855`, `announce.js:20` |
| “Versions” | Opens “Earlier versions” | `editor.js:2300` |
| “Put in the Draft” | Restores a feed snapshot to its draft; does not undo a person merge | `editor.js:2245`, `editor.js:2265` |
| Feed-credit fields “Name”, “Role”, “Picture address”, “Information address”, “Nostr” | Edit the selected draft credit | `editor.js:315` |
| Feed-support fields “Text”, “Address” | Edit the selected draft support link | `editor.js:348` |
| Recipient “Remove” | Removes the draft recipient row | `values.js:196` |
| Recipient “Fee” | Changes the draft fee flag | `values.js:192` |
| “Back”, “Forward”, “Destinations”, and narrow `${title}` button | Shell history or root navigation | `app.js:1969` |

Keyboard controls:

- Arrow Up/Down moves between rows.
- Home/End moves to the first/last row.
- Enter opens a page row or toggles an expandable row.
- Arrow Right opens an expandable row.
- Arrow Left closes an expandable row.

Sources: `listkeys.js:73`, callers `pickers.js:35`, `peoplepages.js:1025`.

Search-field Escape closes its floating field list first. Without that list, the code permits the search field’s normal Escape action (`pickers.js:115`). Field-list selection and insertion use the returned core ranges (`pickers.js:84`).

No People-specific global keyboard shortcut is registered in the scoped files.