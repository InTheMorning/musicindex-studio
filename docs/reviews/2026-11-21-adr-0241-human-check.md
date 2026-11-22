# Human Check Of ADR 0240 And ADR 0241 On The Desktop

The operator ran the release binary of `0f2880b` on the desktop data. This file records the steps and the findings. The fixes start after the check ends.

## Steps

1. Start with one collection. Pass: the footer has People, File hosts, Settings, Lock Studio. It has no collection row (ADR 0149 item 17).
2. Make the collection "TJ" with **New Collection…** and **Make and Open**. Pass: Studio opens TJ on its Welcome page. The footer has "Collection: TJ" and "Switch Collection…".
3. Open HeyCitizen from the sheet "Switch Collection". KeePassXC asks for its connection. This is correct: the vault key of HeyCitizen is in the keyring (ADR 0235). TJ has no vault, so the switch to TJ reads no keyring.

## Findings

1. The footer has two rows for one subject. The row "Collection: {name}" opens Settings, and the row "Switch Collection…" opens the sheet. The operator wants one row. [ADR 0242](../adr/0242-the-collection-row-of-the-footer-opens-switch-collection.md) gives the change, and task 228 builds it.
