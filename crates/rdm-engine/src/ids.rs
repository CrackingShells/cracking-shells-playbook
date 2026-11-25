//! Minting ids (`store.md`, §Ids): a random body drawn from an injected generator, checked
//! against the ids the caller can see, drawn again on a collision.
//!
//! Node, criterion and step ids are minted by the caller, outside the lock, with [`mint_id`];
//! the store mints only the ULIDs of the amendments, comments and intents it writes
//! ([`UlidMint`]).

use rdm_schema::ids::{
    CRITERION_PREFIX, CriterionId, ID_ALPHABET, ITEM_BODY_LEN, NODE_BODY_LEN, NODE_PREFIX, NodeId,
    STEP_PREFIX, StepId, ULID_ALPHABET, ULID_LEN,
};

/// Collisions in a row after which minting gives up. A real generator never gets there; a test
/// generator that always collides must still end.
pub const MAX_COLLISIONS: usize = 32;

/// A source of random bytes.
pub trait IdRng {
    /// Fills `buf` with random bytes.
    fn fill(&mut self, buf: &mut [u8]);
}

/// The operating system's generator.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsRng;

impl IdRng for OsRng {
    fn fill(&mut self, buf: &mut [u8]) {
        // A failing system generator leaves nothing sound to continue with.
        getrandom::fill(buf).expect("the operating system's random generator");
    }
}

/// Why no id was minted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintError {
    /// [`MAX_COLLISIONS`] candidates in a row were already taken.
    Exhausted,
}

impl std::fmt::Display for MintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("no free id after 32 draws")
    }
}

impl std::error::Error for MintError {}

/// An id type that can be minted: its prefix and the length of its body.
pub trait Minted: Sized {
    const PREFIX: &'static str;
    const BODY_LEN: usize;
    /// Builds the id from a string of the right format.
    fn from_minted(s: &str) -> Self;
}

macro_rules! minted {
    ($ty:ident, $prefix:expr, $len:expr) => {
        impl Minted for $ty {
            const PREFIX: &'static str = $prefix;
            const BODY_LEN: usize = $len;
            fn from_minted(s: &str) -> Self {
                // The body is drawn from the alphabet, so the string always parses.
                Self::parse(s).expect("a minted id is well formed")
            }
        }
    };
}
minted!(NodeId, NODE_PREFIX, NODE_BODY_LEN);
minted!(CriterionId, CRITERION_PREFIX, ITEM_BODY_LEN);
minted!(StepId, STEP_PREFIX, ITEM_BODY_LEN);

/// Mints an id of type `T`. Each character takes five bits of a random byte, which is exactly
/// uniform over the 32 characters of the alphabet. `taken` says whether a candidate is already
/// an id the caller can see (the campaign as last read, the ids of the same batch).
///
/// # Errors
/// [`MintError::Exhausted`] after [`MAX_COLLISIONS`] taken candidates in a row.
pub fn mint_id<T: Minted>(
    rng: &mut dyn IdRng,
    mut taken: impl FnMut(&T) -> bool,
) -> Result<T, MintError> {
    let alphabet = ID_ALPHABET.as_bytes();
    for _ in 0..MAX_COLLISIONS {
        let mut bytes = vec![0u8; T::BODY_LEN];
        rng.fill(&mut bytes);
        let mut s = String::from(T::PREFIX);
        s.extend(
            bytes
                .iter()
                .map(|b| char::from(alphabet[usize::from(b & 31)])),
        );
        let candidate = T::from_minted(&s);
        if !taken(&candidate) {
            return Ok(candidate);
        }
    }
    Err(MintError::Exhausted)
}

/// Mints ULIDs that increase strictly within one process, so records written in sequence order
/// by id (amendments and comments order by id, oldest first).
#[derive(Debug, Default)]
pub struct UlidMint {
    last: u128,
}

impl UlidMint {
    /// A ULID for the instant `unix_ms`, with 80 random bits from `rng`. Strictly greater than
    /// the previous one it minted.
    pub fn mint(&mut self, unix_ms: u64, rng: &mut dyn IdRng) -> String {
        let mut random = [0u8; 10];
        rng.fill(&mut random);
        let low = random
            .iter()
            .fold(0u128, |acc, b| (acc << 8) | u128::from(*b));
        let mut value = (u128::from(unix_ms & 0xFFFF_FFFF_FFFF) << 80) | low;
        if value <= self.last {
            value = self.last + 1;
        }
        self.last = value;
        let alphabet = ULID_ALPHABET.as_bytes();
        (0..ULID_LEN)
            .map(|i| {
                // 26 characters of 5 bits are 130 bits; the first holds the top 3 of 128.
                let shift = 5 * (ULID_LEN - 1 - i);
                char::from(alphabet[usize::try_from((value >> shift) & 31).unwrap_or(0)])
            })
            .collect()
    }
}
