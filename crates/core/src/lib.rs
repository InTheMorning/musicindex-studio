//! Core of musicindex-studio.
//!
//! This crate holds the feed model, the RSS reader and writer, the
//! connectors, the vault and the write pipeline (ADR 0002). It contains no
//! HTTP server code and no UI code, and it depends on no other workspace
//! member. It makes HTTP requests to read remote files.

pub mod address;
pub mod announce;
pub mod apps;
pub mod collection;
pub mod connector;
mod digest;
pub mod discover;
pub mod download;
pub mod feed;
pub mod folder_scan;
pub mod guid;
pub mod inventory;
pub mod library;
pub mod live;
pub mod media;
pub mod migrate;
pub mod msp;
pub mod musicindex;
pub mod nostr_events;
pub mod pipeline;
pub mod playlist;
pub mod publisher;
pub mod remote;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod vault;
pub mod web;
pub mod xml;
