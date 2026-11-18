# ADR 0217: A New Artist Sees The Result Of A Write, The Payment Types And An Undo Of A Merge

- Status: Accepted
- Date: 2026-11-18
- Owner: operator
- Accepted: 2026-11-18 by the operator, through the questionnaire, from drawings. The operator selected the result panel for T1. For T2, the operator asked for an explanation with a picture, a text and examples. The operator then selected the panel below the menu, and kept the words "LNURL address" and "Node address". For T3, the operator selected no first selection and an undo.
- Technical choices: the main session selected items 2, 5, 6, 9, 10, 13, 16 and 17.
- Bar: crosses modules. Item 15 needs an undo in the core, the API and the UI. An agent can also make the opposite choice by mistake: close Review after a write, or select a person at the start of a merge.
- Changes: [ADR 0203](0203-people.md) item 52. The menu "Person to merge" has no first selection, and a merge has an undo.
- Uses: [ADR 0138](0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) item 3 for the type words. [ADR 0203](0203-people.md) items 31 and 32 for the merge. [ADR 0175](0175-the-check-shows-what-music-index-holds.md) item 1 for the notice of Music Index after a write. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1 for the warning title. [ADR 0007](0007-draft-diff-write-and-announce.md) for Review.
- Source: [analysis of two journeys for a new independent artist](../research/2026-11-18-user-journeys/analysis.md), themes T1, T2 and T3, and its section "Operator choices".

## Context

Two independent evaluations examined two journeys for a new independent artist. This user knows music files and does not know feeds, file hosts, Lightning or Nostr. The analysis ranks ten themes. This record gives the operator choices for the first three.

- **T1.** The first run gives a folder on this computer as the default file host, and its public address is optional (`welcome.js:92`). After a write, Review closes, and no message tells the result (`editor.js:2059`). The artist can think that the music is public when no listener can get it.
- **T2.** The form "Add a Recipient" starts with "Node Address" and gives only a hint about the format. It accepts a Lightning address with the type "Node Address" (observation O2, `values.js:244`).
- **T3.** The merge sheet selects the first other person at the start (`peoplepages.js:1278`). A row of a credit name tells no feed (`people.js:2199`). A merge deletes the other person, and no step undoes it. The core keeps no data that can undo it.

## Decision

### T1: The Result Of A Write

1. After a write of one feed from Review succeeds, Review stays open. The result panel replaces the diff, and the footer has only **Close**.
2. The result panel has the title "Written". Its first line is "{title} is written to {host name}." with the check icon.
3. When the written feed has a public address, the panel shows the line "Public address" with the address and the Copy button.
4. When the written feed has no public address, the panel shows a warning with the title "Listeners cannot get it yet." For a folder on this computer, the text is "This folder has no public address." For each other host, the text is "This file host gives no public address." The button **Put It Online…** opens File hosts with the edit form of that host open.
5. The UI makes the panel from the feed that the write route gives back and from the host of the write. The core does not change. The notice of ADR 0175 item 1 on the feed page does not change.
6. Items 1 to 4 do not apply to these writes: a write of a change set, a write that deletes the feed file, and a write under a show grant.

### T2: The Two Payment Types

7. Each form with the menu "Type" of an address shows a panel under the menu. The panel tells the selected type with an icon, one sentence, an example and where to find the address. It changes with the selection. The menu stays a select menu.
8. The panel texts:

   | Type | Icon | Text | Example | Where to find it |
   |---|---|---|---|---|
   | "LNURL Address" | `at-sign` | "Looks like an email address. Most wallet apps give you one. Some apps call it a Lightning address." | `name@getalby.com` | "In your wallet app, under Receive." |
   | "Node Address" | `zap` | "A code of 66 letters and numbers. You have one when you run a Lightning node, or when a node service gives you one." | `031ce2f1…1d9dc0d5` | "In the settings of your node." |

9. The words of ADR 0138 item 3 stay: "LNURL Address" and "Node Address" in a menu, and "LNURL address" and "Node address" in a line. "LNURL Address" is the first item of each type menu.
10. A new recipient in the form "Add a Recipient" starts with "LNURL Address". A recipient that exists keeps its type. The sheets of a change set keep their start type (ADR 0207 items 43 and 47).
11. The form refuses an address that does not match its type. A node address is 66 hexadecimal characters. An LNURL address has the form that `keysend::lookup_url` accepts. The field tells the reason. When the address matches the other type, the field also has the button **Use LNURL Address** or **Use Node Address**.

### T3: The Merge And Its Undo

12. The menu "Person to merge" starts on "Select a person". The links and the result show after a selection. **Merge the People** is disabled until the user selects a person.
13. Each row of a credit name tells where the name is a credit: the feed title and the role, for example "Credit name · Bloodshot Lies · Host". A name with credits in more than one feed tells the count, for example "Credit name · 3 feeds". The API gives these facts from the credits of the collection.
14. After a merge, the page of the person shows a notice "{other name} is merged into {name}." with the button **Undo**. The notice stays until the user leaves the page.
15. **Undo** puts back the other person, with its ID, and each row of the two persons that the merge moved, changed or deleted. The core does it in one transaction. An undo changes no feed (ADR 0203 item 32).
16. The core keeps the undo data of the last merge only. It deletes the data at the next merge, and at each change of one of the two persons. An undo after that is refused with the text "Studio cannot undo this merge, because a person changed after it."
17. The undo data stays in the collection, so that it stays after a restart of Studio.

## Alternatives Considered

- T1: a changed first run, where each host row tells if listeners can get the music. Not selected. The result after the write stays silent with it.
- T1: the result panel and the changed first run. Not selected. The operator selected the smaller change.
- T2: one address field, where Studio finds the type from the text. Not selected. The operator asked for an explanation of the two types.
- T2: a fold "Which one do I have?" with the two types side by side, and two picture cards. Not selected. Two picture cards are a radio group in a different form.
- T2: the words "Lightning address" and "Node address", or "Lightning address" and "Lightning node" as on the website. Not selected. The operator kept the words of ADR 0138.
- T3: no first selection and a confirmation, with no undo. Not selected. The operator selected the undo, with its change in the core.

## Consequences

- A write from Review needs one more click: **Close**.
- The type panel makes each form with a type menu higher.
- A recipient that the form accepted before can be refused now, when its type is wrong.
- The collection gets a table for the undo data. This needs a schema step.
- The merge sheet needs a second request, or more facts in the list of persons, for item 13.

## How This Decision Is Checked

Mechanical:

- A UI test examines the function that makes the result panel from a feed and its host. For a feed with a public address, it gives the line of item 3. For a folder with no public address, it gives the warning of item 4.
- A UI test examines the panel texts of item 8 and the order of item 9. It also examines the start type of item 10 and the address check of item 11.
- A UI test examines that the merge form starts with no selected person, and the line of a credit name of item 13.
- A core test merges two persons, then undoes the merge. Each row of the two persons is the same as before the merge. A second core test changes one person after a merge and examines that the undo is refused.

Visual, by screenshots of the main session before each commit:

- The result panel with a public address, and with the warning.
- The type panel for each type, in the wide and the narrow layout.
- The merge sheet before and after a selection, and the notice with **Undo**.
