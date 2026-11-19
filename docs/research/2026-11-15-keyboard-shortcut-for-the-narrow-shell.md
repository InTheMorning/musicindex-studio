# Research: A Keyboard Shortcut For The Narrow Shell

## Summary

- Studio has no sidebar at 760 px or less. The narrow shell shows one page of a stack.
- A shortcut can open the root page, move back one page, or show a new overlay panel.
- The overlay panel conflicts with the current narrow-shell rules.
- The browser and operating-system checks do not prove that a candidate key is free.
- Recommendation: give the existing Back step a shortcut after keyboard and assistive-technology inspection.
- This research does not change a rule or Studio code.

## Option 1: Open The Root Page

### What the user sees

The shortcut selects the root page. The root page shows the destinations and the footer. The current page leaves view.

### Evidence

Studio uses the narrow layout at 760 px or below (`crates/api/ui/app.js`, commit `033c89c2910447c8af075b19c353b047ac41321b`, lines 1843–1845, read 2026-11-15). The shell has no sidebar in that layout (same commit, lines 3973–3978). The root page shows the destinations and footer (same commit, lines 3324–3335).

The narrow stack starts with the root page. Studio navigation can select a stack entry through its navigation record (`crates/api/ui/shell.js`, same commit, lines 806–828, read 2026-11-15). Inference: a shortcut can select the root entry without adding a new view.

Studio has local key handlers for segments, lists and menus. The search found no global `keydown` handler on `window` or `document`. The search covered `crates/api/ui/app.js` line 2036, `crates/api/ui/listkeys.js` lines 75 and 148, `crates/api/ui/menu.js` line 24, and `crates/api/ui/pickers.js` lines 35 and 114 on 2026-11-15. A new key handler is necessary for a global shortcut.

### Good points

- The user can open the destinations from each narrow page.
- This selection uses the page that holds the narrow destinations.

### Bad points

- The user leaves the current page and must go back through navigation.
- This behavior differs from Back when the current page is more than one level below the root.

### Rules that it changes

This selection does not add a panel or change the stack sequence. It adds a keyboard path to the root page. No current record defines that shortcut. Studio ADR 0088 item 7 and ADR 0085 item 7 also apply. musicindex ADR 0069 R-SHELL-2 also applies.

## Option 2: Move Back One Page

### What the user sees

The shortcut performs the current Back step. The user sees the prior page in the stack. From a level page, this opens the level above it.

### Evidence

Studio ADR 0088 item 7 says each level is a stack page and Back opens the level above. Studio code builds those pages in `narrowStack` and `levelPages` (`crates/api/ui/shell.js`, commit `033c89c2910447c8af075b19c353b047ac41321b`, lines 804–834, read 2026-11-15). The Back step uses browser history when its entry matches Studio navigation (`crates/api/ui/app.js`, same commit, lines 3721–3734, read 2026-11-15).

musicindex ADR 0069 R-SHELL-2 item 3 says that Back pops the stack. It also says the device Back button and browser Back button act like the back control. The record was read on 2026-11-15 from `/home/citizen/build/musicindex/docs/adr/0069-the-interface-rules-and-the-shell-of-the-three-apps.md`, musicindex commit `f4e8ca35f627fe3587f4feb73c2f85fa08fe0394`.

### Good points

- This selection follows the current navigation step and keeps its history behavior.
- It does not add a panel or a second page area.

### Bad points

- Back from a page can open different prior pages. It does not always show the root page.
- A shortcut may use the same key as a browser, operating system or assistive technology.

### Rules that it changes

This selection adds a keyboard path to the existing Back step. It does not change Studio ADR 0088 item 7, Studio ADR 0085 item 7, or musicindex ADR 0069 R-SHELL-2. musicindex ADR 0069 R-NAV-1 item 2 says Back goes to the prior screen. It was read on 2026-11-15 from the same record and commit named above.

## Option 3: Show An Overlay Panel

### What the user sees

A panel appears above the current page. The current page remains below it. A second shortcut or dismissal step would close the panel.

### Evidence

Studio makes the sidebar only when the layout is not narrow (`crates/api/ui/app.js`, commit `033c89c2910447c8af075b19c353b047ac41321b`, lines 3973–3978, read 2026-11-15). The narrow stack maps each level to a page (`crates/api/ui/shell.js`, same commit, lines 806–834).

musicindex ADR 0069 R-SHELL-1 item 6 says that no other second pane exists. R-SHELL-2 item 1 says the narrow shell has no menu button and no drawer. These items were read on 2026-11-15 from the record and commit named in Selection 2.

### Good points

- The user can see the destinations while the current page stays open.
- This selection resembles a standard sidebar toggle in some desktop applications.

### Bad points

- The narrow shell has no panel to open. This selection requires a new overlay and new close behavior.
- The overlay adds a new focus and dismissal mode.

### Rules that it changes

This selection changes Studio ADR 0088 item 7 by adding an overlay above the stack page. It also changes musicindex ADR 0069 R-SHELL-2 item 1, which bars a drawer, and R-SHELL-1 item 6, which bars a new pane. Studio ADR 0206 also owns the shell. Its current narrow-shell rule was read on 2026-11-15 at `docs/adr/0206-the-shell-and-the-editor-pages.md`, item 8.

## Key Combination Evidence

Each result names a reviewed shortcut list. “No documented use” means that the cited source gives no task for the key. This result does not prove that no app or extension uses it.

### Browsers

| Combination | Chrome | Firefox | Safari |
| --- | --- | --- | --- |
| Ctrl+B | no documented use [C] | Bookmarks sidebar [F] | no documented use [S] |
| Ctrl+Shift+B | Bookmarks bar [C] | Show or hide bookmarks toolbar [F] | no documented use [S] |
| Ctrl+Alt+S | no documented use [C] | no documented use [F] | no documented use [S] |
| Alt+Shift+S | no documented use [C] | no documented use [F] | no documented use [S] |
| Command+Shift+L | not applicable [C] | no documented use [F] | no documented use [S] |

Chrome and Firefox document Ctrl+B and Ctrl+Shift+B for bookmark controls. Safari documents Control-Command+1 and Control-Command+2 for its two sidebars [S]. Safari does not show Command+Shift+L for its sidebar.

### Operating Systems

| Combination | GNOME | KDE Plasma | Windows | macOS |
| --- | --- | --- | --- | --- |
| Ctrl+B | no documented use [G] | no documented use [K] | Bold text [W] | no documented use [M] |
| Ctrl+Shift+B | no documented use [G] | no documented use [K] | no documented use [W] | no documented use [M] |
| Ctrl+Alt+S | no documented use [G] | no documented use [K] | no documented use [W] | no documented use [M] |
| Alt+Shift+S | no documented use [G] | no documented use [K] | no documented use [W] | no documented use [M] |
| Command+Shift+L | not applicable [G] | not applicable [K] | not applicable [W] | no documented use [M] |

The Windows list gives Ctrl+B for bold text in text fields. macOS documents Command+B for bold text and Option-Command+S for the Finder sidebar [M]. The latter is not one of the candidates in this table.

### WebViews And Assistive Technology

| Combination | WebKitGTK | Tauri WebView2 (Windows) | Tauri WKWebView (macOS) | Tauri WebKitGTK (Linux) | NVDA | JAWS | VoiceOver | Orca |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ctrl+B | no documented use [WK] | no documented use [WV] | no documented use [TA] | no documented use [WK] | no documented use [N] | no documented use [J] | no documented use [V] | no documented use [O] |
| Ctrl+Shift+B | no documented use [WK] | no documented use [WV] | no documented use [TA] | no documented use [WK] | no documented use [N] | no documented use [J] | no documented use [V] | no documented use [O] |
| Ctrl+Alt+S | no documented use [WK] | no documented use [WV] | no documented use [TA] | no documented use [WK] | no documented use [N] | no documented use [J] | no documented use [V] | no documented use [O] |
| Alt+Shift+S | no documented use [WK] | no documented use [WV] | no documented use [TA] | no documented use [WK] | no documented use [N] | no documented use [J] | no documented use [V] | no documented use [O] |
| Command+Shift+L | not applicable [WK] | not applicable [WV] | no documented use [TA] | not applicable [WK] | no documented use [N] | no documented use [J] | no documented use [V] | no documented use [O] |

The Tauri guide names WebView2 on Windows, WKWebView on macOS and WebKitGTK on Linux [TA]. WebKitGTK docs describe a web content view, not browser chrome [WK]. Tauri uses these platform webviews [TA]. WebView2 lets the host handle accelerator keys [WV]. The sources do not give embedded webviews the browser bookmarks UI or these browser shortcuts. The host or web page can accept a key.

The screen-reader key records give many commands with their own modifier keys. They do not name these candidates. This result records the sources checked. It does not show that the combinations are safe in all screen-reader modes.

Sources, read 2026-11-15:

- [C] Google Chrome: https://support.google.com/chrome/answer/157179.
- [F] Mozilla Firefox shortcuts: https://support.mozilla.org/en-US/kb/keyboard-shortcuts-perform-firefox-tasks-quickly. The direct page returned a client challenge. The Google Translate mirror loaded the page: https://support-mozilla-org.translate.goog/en-US/kb/keyboard-shortcuts-perform-firefox-tasks-quickly?_x_tr_sl=auto&_x_tr_tl=en&_x_tr_hl=en.
- [S] Apple Safari: https://support.apple.com/guide/safari/keyboard-shortcuts-and-gestures-cpsh003/mac.
- [G] GNOME: https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html.
- [K] KDE: https://userbase.kde.org/System_Settings/Shortcuts.
- [W] Windows shortcuts: https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows.
- [M] macOS shortcuts: https://support.apple.com/en-us/102650.
- [WK] WebKitGTK WebView docs: https://webkitgtk.org/reference/webkit2gtk/stable/class.WebView.html.
- [TA] Tauri v2 webview versions: https://v2.tauri.app/reference/webview-versions/.
- [WV] WebView2 accelerator keys: https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controller#add_acceleratorkeypressed.
- [N] NV Access: https://www.nvaccess.org/files/nvda/documentation/userGuide.html.
- [J] Freedom Scientific: https://support.freedomscientific.com/Content/Documents/Manuals/JAWS/Keystrokes.pdf.
- [V] Apple VoiceOver: https://support.apple.com/guide/voiceover/welcome/mac.
- [O] GNOME Orca: https://help.gnome.org/orca/commands.html.

The listed pages loaded on 2026-11-15. The Firefox and Obsidian pages did not open directly. The Firefox mirror opened the source page. WebView2 docs describe the host accelerator key callback, not browser shortcut defaults.

### Sidebar Shortcuts In Other Apps

| App | Documented shortcut | Source |
| --- | --- | --- |
| Visual Studio Code | Ctrl+B on Windows and Linux, and Command+B on macOS toggles the primary side bar. | https://code.visualstudio.com/docs/configure/keybindings |
| Finder | Option-Command+S shows or hides the sidebar. | https://support.apple.com/guide/mac-help/keyboard-shortcuts-mchlp2262/mac |
| Messaging app | No sidebar toggle appears. Arrow keys resize the left sidebar. | https://slack.com/help/articles/201374536-Slack-keyboard-shortcuts |
| Notion | No sidebar shortcut appears in the reviewed keyboard-shortcut page. | https://www.notion.com/help/keyboard-shortcuts |
| Obsidian | No shortcut list loaded at the documented Help URL. | https://help.obsidian.md/keyboard-shortcuts |
| GitHub | No sidebar shortcut appears in the reviewed keyboard-shortcut list. | https://docs.github.com/en/get-started/accessibility/keyboard-shortcuts |

The reviewed app pages give one other sidebar toggle: Finder’s Option-Command+S. One app documents arrow keys to resize its left sidebar, but gives no toggle. The Notion, Obsidian and GitHub pages give no sidebar toggle.

### Screen Readers And WCAG

For a shortcut with printable characters only, WCAG 2.1.4 requires a safeguard. The user can disable it, remap it to include a non-printable key, or limit it to a focused component. Source: W3C, “Understanding Success Criterion 2.1.4: Character Key Shortcuts,” https://www.w3.org/WAI/WCAG22/Understanding/character-key-shortcuts.html, read 2026-11-15.

The W3C explanation says screen-reader key handling varies by platform and screen reader. It describes single-letter browse commands on Windows and modifier-key commands on macOS and Linux. It names no modifier combination that is free on all systems. Examine each candidate with assistive technology before release.

The five candidates in the tables use modifier keys. The character-key safeguard of WCAG 2.1.4 does not apply to them. The criterion does not show that a candidate avoids keys used by a browser, operating system, app or screen reader.

## Answers To The Eight Questions

1. The shortcut can select the root page, perform Back to show the prior stack page, or open a new overlay panel. The first two select current pages. The overlay needs new interface behavior.
2. The root-page and Back choices add a keyboard path. The overlay changes Studio ADR 0088 item 7, musicindex ADR 0069 R-SHELL-2 item 1, and R-SHELL-1 item 6. Studio and musicindex own their records.
3. The tables give documented uses or “no documented use” for each candidate in each source. Chrome and Firefox use Ctrl+B and Ctrl+Shift+B for bookmark controls. Windows uses Ctrl+B for bold text. The reviewed screen-reader sources name no candidate key. No result proves that a key is free in all apps or modes.
4. Visual Studio Code uses Ctrl+B on Windows and Linux, and Command+B on macOS, to toggle its primary side bar. Finder uses Option-Command+S. One app page lists keys to resize its left sidebar, but no toggle. The reviewed Notion, Obsidian and GitHub pages give no documented sidebar shortcut.
5. musicindex ADR 0069 R-SHELL-2 gives the shared narrow stack. musicindex ADR 0045 R-SYSTEM-1 item 1 says the apps share tokens, button roles, shapes, words and page structure. The reviewed HitIt ADRs show no shortcut for this panel. HitIt ADR 0083 says keyboard shortcuts are optional and the app is not a keyboard-driven power tool. The record was read on 2026-11-15 from `/home/citizen/build/HitIt/docs/adr/0083-design-language.md`, HitIt commit `e62c0f6f10951ccd76bf35e95651de3b95d5e107`.
6. WCAG 2.1.4 needs a safeguard only for a shortcut with printable characters alone. Each candidate includes modifiers. Each selection needs keyboard access and a check for key conflicts.
7. A tooltip on Back, a shortcut label in Settings, or the two methods can teach the shortcut. Studio records and UI source searched for this behavior give no method.
8. Studio and musicindex records searched for a wide-width rule give no answer. At a wide width, the left panel shows. Inference: a shortcut that only opens the panel has no effect that the user can see there.

## Recommendation And Conditions

Give the current Back step a shortcut at a narrow width. This selection follows Studio ADR 0088 and musicindex ADR 0069 R-SHELL-2. It follows the current stack and adds no drawer.

Examine key combinations in Chromium, Firefox, Safari, WebKitGTK, and Tauri on Linux, macOS, and Windows. Examine supported screen readers in browse and navigation modes. The current sources do not show that a key combination is free.

Before Studio code changes, decide if the shortcut applies at wide widths. If it applies only at a narrow width, name that condition with the shortcut. If the shortcut uses printable characters alone, provide a WCAG 2.1.4 safeguard.

## Items This Research Could Not Find

- If a Back shortcut applies at wide widths. Search: Studio ADR 0088 item 7, Studio ADR 0085 item 7, Studio ADR 0206 and musicindex ADR 0069 R-SHELL-2, read 2026-11-15.
- How Studio teaches the shortcut. Search: Studio UI labels and Settings page, `crates/api/ui/`, and Studio ADR 0088 and ADR 0206, read 2026-11-15.
- A shortcut for the same panel in musicindex.org or HitIt. Search: musicindex ADR 0069 and 0045, HitIt ADR 0083, and HitIt UI shortcut labels, read 2026-11-15.
- An Obsidian shortcut list. Search: the official Help URL returned a short page without its shortcut list, https://help.obsidian.md/keyboard-shortcuts, read 2026-11-15.
