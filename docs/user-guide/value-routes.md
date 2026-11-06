# Value routes

A value route tells apps who gets the payments for your music.
Each recipient has a share of the payment.

Studio shows the recipients of one payment block for the album and for each track.
Studio keeps the payment values that the editor does not show.
For example, it keeps value time splits and extra payment blocks the same.

## Album recipients

1. Open the album.
2. Select the tab **Value Routes**.

The album recipients get the payments for each track that has no recipients of its own.
If the album has no recipients, select **Add recipients**.

The page has two groups: **Recipients** and **Fees**.

## Add a recipient

Select **Add a recipient**. Studio opens a new line. Fill in these fields:

| Field | What to enter |
| --- | --- |
| Name | The name of the recipient. |
| Type | **node (keysend)** or **lnaddress**. |
| Address | The public key of the Node address (66 hexadecimal characters), or an LNURL address such as `name@example.com`. |
| Custom key | Only for a node that many people share. This field shows only for the type **node (keysend)**. |
| Custom value | The account on that node. This field shows only for the type **node (keysend)**. |
| Split (shares) | The share of this recipient. |

Each line shows the name, a bar, the percent and the address.
The percent comes from the shares of all recipients.

To make a recipient a fee, select the check box "Fee, paid first".
The recipient then moves to the group **Fees**, and the field is **Split (percent)**.

The rows sort by split, the biggest first. They sort when you leave the row. When you change a list of recipients, the feed gets the same sequence. A list that you do not change stays as it is in the feed.

In an open recipient you can also use:

- **Change…** to change the address.
- **Add to People**, when no person holds the address.
- **Remove**.

## Add a recipient

1. Select **Add a recipient**. The sheet **Add a Recipient** opens. The form is at the top.
2. Type the values, or select a row from **Your people** or **Payees with no person** to fill the form.
3. Select **Add the Recipient**, then type the split in the row.

To copy many recipients of a different feed, select **Copy from a Feed on musicindex** in the sheet. Search for the feed, open it or one of its tracks and select the recipients. Then select **Add N Recipients**.

## Recipients for one track

A track uses the album recipients by default.
To give a track its own recipients:

1. Open the track page.
2. In **Value routes**, select **Give This Track Its Own Recipients**.
3. Edit the recipients as for the album.

The recipients of the track replace the album recipients for that track only.
To go back, select **Use the Album Recipients**. Studio asks you to confirm.
