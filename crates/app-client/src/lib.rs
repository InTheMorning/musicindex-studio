//! `studio-app-client`: the client of a connected app for musicindex Studio.
//!
//! The crate does the full protocol of [ADR 0110]: the pairing, the signed
//! requests, the wrapper of a grant vault key and the four operations of a
//! show. It does not depend on `studio-core`. Studio tests the crate against
//! its own API in each CI run.
//!
//! # Rules for the app
//!
//! These rules come from ADR 0110 item 32.
//!
//! - Keep the private key private. Keep it out of a log, a backup that other
//!   persons can read, and a support message.
//! - Use one pairing for each copy of the app.
//! - Keep the grant vault key in memory only. The client does this. Do not
//!   copy the key out of the crate.
//! - Log no pairing text, signed event, request body or grant vault key.
//! - Keep the clock of the computer correct.
//!
//! # The steps of a show
//!
//! Each step shows one call. The steps that need a live Studio use `no_run`.
//!
//! ## 1. Keep a key
//!
//! [`FileSigner`] keeps the private key in a file with mode 0600, in a folder
//! with mode 0700. An app can give its own [`AppSigner`].
//!
//! ```
//! use studio_app_client::{AppSigner, FileSigner};
//!
//! // The signer makes a missing folder with mode 0700.
//! let folder = tempfile::tempdir()?;
//! let signer = FileSigner::load_or_create(folder.path().join("keys").join("studio.key"))?;
//! println!("public key: {}", signer.public_key().to_hex());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 2. Read the pairing text
//!
//! The user makes a pairing text in Studio. Read it with [`input::prompt`], or
//! with `input::from_clipboard` (feature `clipboard`). Then parse it. After
//! the first pairing, the app saves the origin and accepts the code alone.
//!
//! ```
//! use studio_app_client::PairingText;
//!
//! let full = PairingText::parse("https://studio.example.com#pair=K7M2-Q9XD")?;
//! let alone = PairingText::parse_with_origin("k7m2 q9xd", full.origin())?;
//! assert_eq!(full, alone);
//! // Studio needs https. Only a loopback address can use http.
//! assert!(PairingText::parse("http://studio.example.com#pair=K7M2-Q9XD").is_err());
//! # Ok::<(), studio_app_client::Error>(())
//! ```
//!
//! ## 3. Pair
//!
//! ```no_run
//! # use studio_app_client::{Client, FileSigner, PairingText};
//! # let signer = FileSigner::load_or_create("studio.key")?;
//! # let text = PairingText::parse("https://studio.example.com#pair=K7M2-Q9XD")?;
//! let client = Client::pair(&text, signer, "wss://live.example.com")?
//!     .with_clock_warning(|seconds| eprintln!("Correct the clock: it is {seconds} s off"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 4. Find the shows
//!
//! ```no_run
//! # use studio_app_client::{Client, FileSigner};
//! # let client = Client::connect("https://studio.example.com", FileSigner::load_or_create("studio.key")?)?;
//! let me = client.whoami()?;
//! println!("{} is {:?}", me.name, me.state);
//! for show in client.shows()? {
//!     println!("grant {}: {:?}", show.grant_id, show.title);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 5. Run the show
//!
//! The first write of a grant gets the wrapper and decrypts the grant vault
//! key. The client keeps the key in memory only.
//!
//! ```no_run
//! # use studio_app_client::{Client, FileSigner, ShowStatus};
//! # let client = Client::connect("https://studio.example.com", FileSigner::load_or_create("studio.key")?)?;
//! # let grant_id = 1;
//! client.set_status(grant_id, ShowStatus::Live)?;
//! client.set_live_value(grant_id, "wss://live.example.com/shows/1", "lnurlp")?;
//! client.remove_live_value(grant_id)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 6. End the show
//!
//! After `Ended`, the client clears the grant vault key.
//!
//! ```no_run
//! # use studio_app_client::{Client, FileSigner, ShowStatus};
//! # let client = Client::connect("https://studio.example.com", FileSigner::load_or_create("studio.key")?)?;
//! # let grant_id = 1;
//! client.set_status(grant_id, ShowStatus::Ended)?;
//! assert!(!client.holds_grant_key(grant_id));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Errors
//!
//! Each [`Error`] has a type. An error text holds no pairing text, event or
//! key. See the README for the action for each error.
//!
//! [ADR 0110]: https://github.com/inthemorning/musicindex-studio/blob/main/docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md

mod client;
mod clock;
mod error;
pub mod input;
mod pairing;
mod signer;
pub mod wire;

pub use client::{CLOCK_LIMIT_SECONDS, CLOCK_WARNING_SECONDS, Client, ShowStatus};
pub use error::Error;
pub use nostr;
pub use pairing::PairingText;
pub use signer::{AppSigner, FileSigner, MemorySigner};
