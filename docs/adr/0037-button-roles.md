# ADR 0037: Button Roles

- Status: Accepted
- Date: 2026-10-23
- Owner: operator, with the technical choices of the planning model
- Changed by: musicindex ADR 0023 R-CASE-1, on 2026-10-25. A button label is title case. [ADR 0010](0010-studio-uses-the-afterglow-direction.md#rules-that-musicindex-replaces) names the change.

## Context

In the visual check of milestone 7a, the operator found that the buttons do not show what they do. "Copy to…", "Move to…" and "Delete" had the same quiet style. That style has the same fill as the surface below it. The operator asked for the button roles of the Apple Human Interface Guidelines (HIG). An action button has the accent color, and a destructive button has red. A button does not have the same fill as the surface below it.

The HIG gives four roles: normal, primary, cancel and destructive. A primary button has the accent color and is the most likely selection of a view. A destructive button uses red, and is not the primary button. The HIG also says: "Avoid applying similar colors to button labels and content layer backgrounds."

## Decision

1. Each labeled button has one of these roles. The class of the button gives its role.

   | Role | Class | Look | Use |
   | --- | --- | --- | --- |
   | Primary | `button primary` | Accent fill, white label | The most likely action of a sheet or a page, such as "Write to the host" or "Add 2 tracks". A maximum of one in each sheet, and one in each page. |
   | Action | `button action` | Accent label on an accent tint | Each other action that adds or changes something, such as "Copy to hostB", "Move to hostB", "Reference", "Import files" or "Scan the hosts". |
   | Normal | `button` | Text label on a neutral fill | A selection with no change, such as "Cancel", "Examine the changes" or "Continue". |
   | Destructive | `button destructive` | Red label on a red tint | Each action that removes or deletes, such as "Delete", "Remove", "Remove the track" or "Discard the draft". |

2. A destructive button is not a primary button.
3. The red of a destructive button is the `--error` token of [ADR 0018](0018-passphrase-strength-and-error-color.md). The accent tint and the red tint are tokens, `--accent-tint` and `--error-tint`, with a light and a dark value.
4. The fill of each button is different from the fill of the surface below it, on each background level of [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md).
5. The class `button quiet` has no fill. It is only for an icon button, such as the close button of a sheet or the theme button. A button with a text label does not use it.
6. A `button link` stays for a link in text.

## Alternatives Considered

- Keep the quiet style for actions in a list, as before. Not chosen by the operator: the actions do not show what they do.
- A filled red destructive button. Rejected: the HIG says to avoid prominence for a destructive action, so that the user does not select it by accident.
- An accent fill for each action button. Rejected: the HIG limits the prominent style to one or two buttons in a view.

## Consequences

- Each screen changes the class of its buttons.
- A test enforces item 5.

## Invariants

- A button with a text label is not a `button quiet`.
- A destructive button is not a `button primary`.
