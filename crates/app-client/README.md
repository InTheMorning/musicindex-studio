# studio-app-client

This crate lets an app connect to musicindex Studio. It does the full protocol of [ADR 0110](../../docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md): the pairing, the signed requests, the grant vault key and the four operations of a show. The crate has the AGPL-3.0 license. The crate documentation (`cargo doc --open`) has an example of each step.

## What The User Does

1. The user opens Settings, Connected apps, "Connect an App" in Studio. The user types a name.
2. Studio shows a pairing text, for example `https://studio.musicindex.org#pair=K7M2-Q9XD`. It works one time, for 15 minutes.
3. The user copies the pairing text into the app. The app reads it from the clipboard or from a prompt that does not show the text.
4. The user selects Done in Studio after the app says that it is connected.
5. For each show, the user approves a diff in Studio and selects the app in the grant. A grant names one app.

## What The App Keeps

- The private key of the app. Use `FileSigner` (a file with mode 0600 in a folder with mode 0700), or give your own `AppSigner`, for example for the key store of the operating system. Studio does not see the private key.
- The origin of Studio. After the first pairing, the app can accept the code alone, for example `K7M2-Q9XD`.
- Its live value origin: the scheme, host and port where its listeners get the live value, for example `wss://live.example.com`. To change it, the user connects the app again.

The app does not keep the pairing text. The client keeps the grant vault key in memory only. It clears the key when the show ends, when the grant ends, and after the error `NotGranted` or `VaultLocked`.

## What The App Does Not Log

- The pairing text or the code.
- The private key.
- A signed event or an `Authorization` header.
- A request body.
- The grant vault key.

The error texts of the crate hold none of these. Do not add them to your own messages.

## Rules

- Use one pairing for each copy of the app.
- Read the pairing text only with `input::prompt` or `input::from_clipboard` (feature `clipboard`). Do not read it from a command argument or an environment variable. A shell history and a process list keep them.
- Show no part of the code after the read. Show the result only.
- Use `https` for Studio. Only a loopback address can use `http`.
- Keep the clock of the computer correct. Studio refuses a request that is more than 60 seconds from its own time. The client corrects a difference of 5 minutes or less, and calls your function above 30 seconds. Show that warning to the user.

## What To Do For Each Error

| Error | What to do |
| --- | --- |
| `CodeInvalid` | Tell the user to make a new pairing text. |
| `CodeUsed` | Tell the user that a different program possibly used the pairing text. The user removes the app in Studio and makes a new pairing text. |
| `ClockWrong` | Tell the user to turn on the automatic time of the computer. |
| `SignatureInvalid` | Tell the user to connect the app again. |
| `KeyRequired` | Tell the user to connect the app again. |
| `KeyInUse` | Make a new key and pair again. |
| `LiveValueInvalid` | Examine the live value origin. To change it, the user connects the app again. |
| `NotGranted` | Stop the writes for this show. Tell the operator. |
| `VaultLocked` | Stop the writes for this show. Tell the operator. |
| `TooManyAttempts` | Wait one minute. Then send the request again. |
| `Network` | Send the request again. The client signs a new event. |
