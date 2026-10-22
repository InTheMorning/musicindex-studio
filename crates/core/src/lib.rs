//! Core of musicindex-studio.
//!
//! This crate holds the feed model, the RSS reader and writer, the
//! connectors, the vault and the write pipeline (ADR 0002). It contains no
//! HTTP code and no UI code, and it depends on no other workspace member.

pub mod collection;
pub mod connector;
mod digest;
pub mod feed;
pub mod guid;
pub mod pipeline;
pub mod vault;
pub mod xml;
