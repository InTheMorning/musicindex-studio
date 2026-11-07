//! Core of musicindex-studio.
//!
//! This crate holds the feed model, the RSS reader and writer, the
//! connectors, the vault and the write pipeline (ADR 0002). It contains no
//! HTTP server code and no UI code, and it depends on no other workspace
//! member. It makes HTTP requests to read remote files.

pub mod account_archive;
pub mod address;
pub mod announce;
pub mod apps;
pub mod assets;
pub mod change_set;
pub mod checked_time;
pub mod collection;
pub mod connector;
pub mod datetime;
mod digest;
pub mod discover;
pub mod download;
pub mod duration;
pub mod feed;
pub mod folder_scan;
pub mod grant;
pub mod grant_vault;
pub mod guid;
pub mod inventory;
pub mod keysend;
pub mod library;
pub mod live;
pub mod lookup;
pub mod media;
pub mod merge;
pub mod migrate;
pub mod move_link;
pub mod moves;
pub mod msp;
pub mod musicindex;
pub mod net;
pub mod nip46;
pub mod nostr_events;
pub mod people;
pub mod pipeline;
pub mod playlist;
pub mod profiles;
pub mod publisher;
pub mod remote;
pub mod review;
pub mod roles;
pub mod signed_request;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod vault;
pub mod web;
pub mod xml;
