# Value routes

A value route tells apps who gets the payments for your music.
Each recipient has a share of the payment.

Studio shows the recipients of one payment block for the album and for each track.
Studio keeps the payment values that the editor does not show.
For example, it keeps value time splits and extra payment blocks the same.

## Album recipients

1. Open the album.
2. Select **Value routes** in the outline.

The album recipients get the payments for each track that has no recipients of its own.
If the album has no recipients, select **Add recipients**.

The page has two groups: **Recipients** and **Fees**.

## Add a recipient

Select **Add a recipient**. Studio opens a new line. Fill in these fields:

| Field | What to enter |
| --- | --- |
| Name | The name of the recipient. |
| Type | **node (keysend)** or **lnaddress**. |
| Address | The public key of the Lightning node (66 hexadecimal characters), or a Lightning address such as `name@example.com`. |
| Custom key | Only for a node that many people share. This field shows only for the type **node (keysend)**. |
| Custom value | The account on that node. This field shows only for the type **node (keysend)**. |
| Split (shares) | The share of this recipient. |

Each line shows the name, a bar, the percent and the address.
The percent comes from the shares of all recipients.

To make a recipient a fee, select the check box "Fee: this recipient gets its split as a percentage before the others share the rest".
The recipient then moves to the group **Fees**, and the field is **Split (percent)**.

In an open recipient you can also use:

- **Save to your recipients**. This keeps the recipient in your list for other feeds.
- **Move up**, **Move down** and **Remove**.

## Pick a recipient that you used before

1. Select **Pick from your recipients**. The sheet "Pick recipients" opens.
2. Under **From musicindex**, search for a feed by name. Open the feed or one of its tracks and select recipients. 
3. Under **Your recipients**, select recipients from your list. Use **Filter** to find a name or an address.
4. Select **Add N recipients**.

Studio does not set a split for a recipient that you add this way.
Type the split for each new recipient.

## Recipients for one track

A track uses the album recipients by default.
To give a track its own recipients:

1. Open the track page.
2. In **Value routes**, select **Give this track its own recipients**.
3. Edit the recipients as for the album.

The recipients of the track replace the album recipients for that track only.
To go back, select **Use the album recipients**. Studio asks you to confirm.
