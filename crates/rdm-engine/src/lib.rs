//! The rdm store engine: a campaign directory read into the [`rdm_schema`] model and written
//! back through ordered, crash-safe writes (`__design__/rdm/store.md`).
//!
//! The one public handle is [`cache::CachedCampaign`]: it opens a campaign, applies the seven
//! writes under the writer lock, recovers an interrupted one, and its `read` revalidates the
//! decoded files and returns a [`store::Store`], the state a cold open would give.
//!
//! - Records: [`codec`] (the canonical text of a stored record and its read-back through the
//!   [`version`] chain), [`rev`] (revisions), [`order`] (order keys), [`ids`] (minting),
//!   [`links`] (the reverse link index).
//! - Reads and writes: [`store`] (the state and its options), [`cache`] (the entries and the
//!   handle), [`mod@write`] (the write algorithm and recovery).
//! - Primitives under the writes: [`fs`] (the `Fs` trait, `atomic_replace`), [`lock`] (the writer
//!   lock and its holder record) and [`intent`] (the redo record), with `layout`
//!   owning every path under `.rdm/`. Every effect goes through
//!   [`fs::Fs`], so a recording implementation can log them in order.

pub mod cache;
pub(crate) mod clock;
pub mod codec;
pub mod fs;
pub mod ids;
pub mod intent;
pub(crate) mod layout;
pub mod links;
pub mod lock;
pub mod order;
pub(crate) mod publish;
pub mod rev;
pub mod store;
pub mod version;
pub mod write;
