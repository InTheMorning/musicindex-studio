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

1. Select **Add a recipient**. The sheet **Add a Recipient** opens.
2. Fill in the form. You can also select a row from **Your people** or **Payees with no person**, and the form gets its values. A long list starts closed: select its heading to open it.
3. Select **Add the Recipient**, then type the split in the row.

The form has these fields:

| Field | What to enter |
| --- | --- |
| Type | **Node Address** or **LNURL Address**. |
| Address | The public key of the Node address (66 hexadecimal characters), or an LNURL address such as `name@example.com`. |
| Name | The name of the recipient. |
| Custom key | Only for a node that many people share. This field shows only for the type **Node Address**. |
| Account | The account on that node. This field shows only for the type **Node Address**. |

To copy many recipients of a different feed, select **Copy from a Feed** below the form. Type the name of the feed, open it or one of its tracks and select the recipients. Each row shows the share in that feed, and the new recipient gets no split. Then select **Add N Recipients**.

## An open recipient

Each line shows the name, a bar, the percent and the address.
The percent comes from the shares of all recipients.

To make a recipient a fee, select the check box "Fee, paid first".
The recipient then moves to the group **Fees**, and the field is **Split (percent)**.

The rows sort by split, the biggest first. They sort when you leave the row. When you change a list of recipients, the feed gets the same sequence. A list that you do not change stays as it is in the feed.

An open recipient also has:

- **Change…** to change the address. **Find on Music Index** in that sheet fills the form from a feed.
- **Add to People**, when no person holds the address.
- **Remove**.

## Recipients for one track

A track uses the album recipients by default.
To give a track its own recipients:

1. Open the track page.
2. In **Value routes**, select **Give This Track Its Own Recipients**.
3. Edit the recipients as for the album.

The recipients of the track replace the album recipients for that track only.
To go back, select **Use the Album Recipients**. Studio asks you to confirm.
