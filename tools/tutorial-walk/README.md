# Tutorial Walk

These scripts take the Studio screenshots of an artist tutorial. They start a
test SFTP server on this computer and a Studio binary with its own data, and
they operate the UI with `tools/drive-ui.mjs`. They are tools for agents and
developers. Studio does not use them.

This document is advisory. No ADR owns it.

## Files

| File | Use |
| --- | --- |
| `setup.sh WORK [SSH_PORT]` | Makes the work folder: SSH keys, the sshd configuration, test media and empty data. |
| `run.sh WORK OUT STEPS_JSON` | Starts sshd and Studio, drives one steps file, and stops the two. |
| `steps.py` | `START` (dark theme, 1280 × 800, unlock), `out()`, `fill()`, `main_text()`, `dialog_text()` and `user_key()`. |
| `example_connect.py WORK STEPS_JSON` | An example part: the SFTP form with example values, then Connect to the test server, the vault and the found feeds. |
| `media.py FOLDER` | WAV files with 16-bit and 24-bit samples, and a cover PNG. `setup.sh` runs it. |
| `crop.py` | Contact sheets of the shots, and the crops for the page. It needs Pillow. |

## Procedure

1. Read the feeds of the artist before the walk. Look for misspelled `podcast:` tags, a missing `podcast:medium` tag and enclosure redirects. A feed defect changes the tutorial.
2. Search `crates/api/ui/*.js` for each button name of the draft. This search finds most text corrections before the first screenshot.
3. Make the work folder in a location that is not in the repository:

   ```sh
   tools/tutorial-walk/setup.sh /tmp/walk
   ```

4. Put test copies of the feeds in `/tmp/walk/host`, at the same paths as on the server. Use a domain that ends in `.example`. A node address has 66 hexadecimal characters.
5. Write one part script for each part of the tutorial, from `example_connect.py`. Do each part in one run. A condition that is only on the page goes away after a restart, for example the list of found feeds after Connect.
6. Make the steps file and run it. All paths are absolute:

   ```sh
   python3 -I tools/tutorial-walk/example_connect.py /tmp/walk /tmp/walk/p1.json
   tools/tutorial-walk/run.sh /tmp/walk /tmp/walk/o1 /tmp/walk/p1.json
   ```

7. Read the log of each run. A step that returns an error stops the run. At the end, `drive-ui.mjs` runs the fit check (musicindex ADR 0065) and exits with 1 when it finds a fault. Such a fault is a UI finding, not a fault of the walk. In rc.32, the SFTP fingerprint gives a fit fault.
8. Make the contact sheets, read them, and write the crop boxes:

   ```sh
   python3 -I tools/tutorial-walk/crop.py sheets /tmp/walk/sheets /tmp/walk/o1/*.png
   python3 -I tools/tutorial-walk/crop.py crop /tmp/walk/crops.json /tmp/walk/page/img
   ```

## Limits

- A write to a `.example` domain fails, because Studio checks the public address before a write. Use the end of Review as the last screenshot of a write.
- With no file host, File hosts shows the form immediately. There is no add button to select.
- A WAV file with 32-bit integer samples is refused.
- The data folder keeps its data between runs. To start again, remove `WORK/data` and `WORK/library`.
- A step waits before its `js`, and the shot is taken after the `js`. To wait after a click, put the wait on the next step.
