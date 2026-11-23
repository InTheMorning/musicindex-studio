"""Helpers to write the steps file of a tutorial walk for tools/drive-ui.mjs.

A part script imports this module and calls out():

    import sys; sys.path.insert(0, "tools/tutorial-walk")
    from steps import START, out, main_text, dialog_text
    out("/abs/work/p2.json", [{"js": "$go('File hosts')"}, ...])

The page helpers ($go, $click, $tab, $fill, $pick and the others) are in
tools/drive-ui.mjs. A step waits before its js, and the shot is taken after
the js. To wait after a click, put the wait on the next step.
"""
import json
from pathlib import Path

PASSPHRASE = "test passphrase 123"

# Dark theme, 1280 x 800, and the unlock of the vault when the vault exists.
START = [
    {"js": "localStorage.setItem('studio-theme','dark');document.documentElement.dataset.theme='dark';window.confirm=()=>true"},
    {"size": [1280, 800]},
    {"wait": 1500, "js": "location.reload()"},
    {"wait": 5000, "js": f"document.getElementById('unlock') ? $fill('unlock', {json.dumps(PASSPHRASE)}) : 'no vault'"},
    {"wait": 500, "js": "(()=>{const b=[...document.querySelectorAll('button')].find(b=>/^Unlock/.test(b.textContent.trim()));if(b){b.click();return 'unlocked'}return 'no unlock'})()"},
    {"wait": 2500, "js": "1"},
]


def main_text(limit=1800):
    """A js expression that prints the start of the main area to the log."""
    return f"(document.querySelector('main')||document.body).innerText.slice(0,{limit})"


def dialog_text(limit=1500):
    """A js expression that prints the start of the open sheet to the log."""
    return f"(document.querySelector('dialog[open]')||{{innerText:'no dialog'}}).innerText.slice(0,{limit})"


def fill(**fields):
    """One js expression that fills fields by id. An underscore in a name is a hyphen."""
    calls = ";".join(f"$fill({json.dumps(k.replace('_', '-'))},{json.dumps(v)})" for k, v in fields.items())
    return f"(()=>{{{calls};return 1}})()"


def user_key(work):
    """The private key of the test SFTP user, for the field sftp-key."""
    return (Path(work) / "sshk" / "user").read_text()


def out(path, steps, start=True):
    """Write the steps file. START comes first unless start is False."""
    Path(path).write_text(json.dumps((START if start else []) + steps))
    print("steps:", path, len(steps), "steps")
