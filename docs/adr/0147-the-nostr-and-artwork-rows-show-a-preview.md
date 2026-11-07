# ADR 0147: The Nostr And Artwork Rows Show A Preview

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, from a mockup and a questionnaire.
- Technical choices: items 4 and 5 are choices of the planning model.
- Source: [findings of local use, finding 8](../reviews/2026-11-06-local-use-findings.md)

## Context

A row with a Nostr key showed only the short key. The row "Artwork" showed only a text. A feed stores only the key, so a name and a picture need a different source. `POST /nostr/resolve` gives them from the relays and stores nothing. The page of a person already sends this request when it opens.

## Decision

1. The preview is 32 px, before the label of the row. A person picture is round, and an image has small corners.
2. The row "Artwork" shows the image of its page. A track with no image of its own shows the cover of its album. A page with no image, or an image that does not load, shows no preview.
3. Each row with a Nostr key asks for the profile of its key through `POST /nostr/resolve`, when the row shows. The rows are "Nostr profile" of a feed, "Nostr" of a credit and the key row of a form of a person.
4. A row shows the short key until the answer comes. A row with a failed request or with no profile stays in this form. A row with a key that is not correct sends no request.
5. A name of the feed or the form comes before the profile name. A profile with a name and no picture shows the initials.
6. Studio stores no result of this request.

## Alternatives Considered

- A preview of 44 px at the end of the row. Not chosen by the operator: the rows are taller, and the name has little space at 390 px.
- A name and a picture from the People list only, with no request. Not chosen by the operator: a key with no person would look the same as before.
- A name and a picture that the collection keeps when the user selects a profile. Not chosen by the operator: the collection gets data that can become old.

## Consequences

- Each page open with a key tells the relays of the lookup list about that key. On a server, the server sends the request.
- The preview comes after the answer, so the label of a Nostr row can move when the answer comes.
- An album page shows the cover in its head and in the row "Artwork".
