//! Filesystem primitives of the rdm store: atomic file replacement, the writer lock with its
//! holder record, and the intent record. Every effect goes through the [`fs::Fs`] trait so a
//! recording implementation can log them in order. This crate defines no contract type: the
//! holder record and the intent payload are bytes, and the caller gives them their meaning.

pub mod cache;
pub(crate) mod clock;
pub mod codec;
pub mod fs;
pub mod ids;
pub mod intent;
pub mod links;
pub mod lock;
pub mod order;
pub(crate) mod publish;
pub mod rev;
pub mod store;
pub mod version;
pub mod write;
