"""Example part script: connect the test SFTP server and add the found feeds.

The form first gets the values of the artist (example values for the
screenshot). Then it gets the values of the test server, and Studio connects.
The list of found feeds shows only after Connect, so this part adds the feeds
in the same run.

Usage: python3 -I tools/tutorial-walk/example_connect.py WORK STEPS_JSON
Then:  tools/tutorial-walk/run.sh WORK OUT STEPS_JSON
"""
import getpass
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from steps import PASSPHRASE, fill, main_text, out, user_key  # noqa: E402

work, steps_file = sys.argv[1], sys.argv[2]
port = (Path(work) / "sshk" / "port").read_text().strip()

out(steps_file, [
    {"js": "$go('File hosts')"},
    {"wait": 2000, "js": "$pick('SFTP server')"},
    # The values that the artist types. These are for the screenshot only.
    {"wait": 1200, "js": fill(sftp_host="example.com", sftp_user="your_user",
                              sftp_root="/home/your_user/example.com",
                              sftp_url="https://www.example.com", sftp_login="password")},
    {"wait": 800, "js": fill(sftp_password="example")},
    {"wait": 600, "js": "1", "shot": "b03-sftp-form"},
    # The values of the test server.
    {"js": fill(sftp_name="Test host", sftp_host="127.0.0.1", sftp_port=port,
                sftp_user=getpass.getuser(), sftp_root=f"{work}/host",
                sftp_url="https://www.example.example/", sftp_login="key")},
    {"wait": 800, "js": fill(sftp_key=user_key(work))},
    {"wait": 600, "js": "$click('Connect')"},
    {"wait": 5000, "js": main_text(), "shot": "b05-fingerprint"},
    {"js": "$click('Accept and Add')"},
    {"wait": 3000, "js": fill(passphrase=PASSPHRASE, confirm=PASSPHRASE), "shot": "b06-vault"},
    {"wait": 600, "js": "$click('Make the Vault')"},
    {"wait": 5000, "js": main_text(), "shot": "b07-recovery"},
    {"js": "[...document.querySelectorAll('main input[type=checkbox]')].pop().click()"},
    {"wait": 600, "js": "$click('Continue')"},
    {"wait": 8000, "js": main_text(), "shot": "b08-found"},
    {"js": "[...document.querySelectorAll('main button')].find((b) => /^Add \\d+ to Studio$/.test(b.textContent.trim())).click()"},
    {"wait": 5000, "js": main_text(), "shot": "b10-added"},
])
