# Examples Of A Default Among Many Addresses

Date: 2026-11-06.

Status: advisory. This note states no rule. It gives the examples for [finding 21 of the local use findings](../reviews/2026-11-06-local-use-findings.md).

## The Question

The page of a person has a select menu "Default address", above the rows of the same addresses. The operator does not like this control, and asks how other products let a user select one default.

## Method

Each row below comes from a help page of the product, and the date of each read is 2026-11-06. These pages describe the controls in words. Most of them do not state how the current default shows, so this note makes no claim about a badge or a mark.

## Examples

| Product | Object | Control | Words on the page | Source |
| --- | --- | --- | --- | --- |
| Gmail | Send-from addresses | A link on the row of each address | "Make default", to the right of the address | [Google](https://support.google.com/mail/answer/22370?hl=en) |
| Amazon | Delivery addresses | An action below each address | "Set as Default" | [Amazon](https://digprjsurvey.amazon.com/csad/help/node/GQT5HV6YYGNDSFNW) |
| Wingspan | Payout methods | An action next to each method, or a menu of three dots on the row | "Default", "Set as default" | [Wingspan](https://intercom.help/wingspan-support/en/articles/6398473-payout-methods) |
| Google Wallet | Payment cards | An action on the details of one card | "Make default for tap to pay" | [Google](https://support.google.com/pay/answer/7644064?hl=en) |
| Apple Wallet | Payment cards | Touch and hold a card, then move it to the front. Also an action at checkout | "Set as Default Card" | [Apple](https://support.apple.com/en-us/118219) |
| GitHub | Email addresses | One drop-down menu, then Save | "Primary email address" | [GitHub](https://docs.github.com/en/enterprise-server@3.16/account-and-profile/how-tos/email-preferences/changing-your-primary-email-address) |

Apple and Google state that the first card that the user adds becomes the default. [Stripe](https://docs.stripe.com/payments/cards/overview) states the same for the first card of a customer.

## What The Examples Show

- Four of the six products put the action on the object: Gmail, Amazon, Wingspan and Google Wallet.
- Apple gives the position in a stack the meaning of the default. A drag has no keyboard path, and Studio needs one ([ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md)).
- Only GitHub has a drop-down menu like the one in Studio. It also has a Save step, as the page of a person has ([ADR 0116](../adr/0116-the-page-of-a-person-has-save-and-discard.md)).

## Limits

- The help pages of BILL and River refused the request, so this note does not use them.
- A Stripe portal has payment methods, but its documents do not name a control for the default. Help pages of other products that use it name "Make default", and this note does not use them as evidence for Stripe.
- No page is a Lightning wallet. A search found no Lightning wallet with a default among many addresses of one user.
- The Amazon page is on the host `digprjsurvey.amazon.com`. The same page has links to the account pages of Amazon.
- The GitHub page is for Enterprise Server 3.16. The page for github.com gives a different path, with an add and a delete.
