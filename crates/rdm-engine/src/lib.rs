//! Filesystem primitives of the rdm store: atomic file replacement, the writer lock with its
//! holder record, and the intent record. Every effect goes through the [`fs::Fs`] trait so a
//! recording implementation can log them in order. This crate defines no contract type: the
//! holder record and the intent payload are bytes, and the caller gives them their meaning.

pub mod codec;
pub mod fs;
pub mod intent;
pub mod lock;
