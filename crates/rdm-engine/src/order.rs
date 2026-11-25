//! Order keys (`store.md`, §Order key): fractional indexing over the digits `0-9A-Za-z`.
//!
//! A key is a non-empty string of digits that does not end in `0`, read as a base-62 fraction.
//! Keys compare bytewise. [`between`] gives a key strictly between two bounds; [`spread`] gives
//! `n` evenly spaced keys at once, for a rebalance or a bulk create.

const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// A key longer than this makes an insert rebalance its siblings.
pub const MAX_KEY_LEN: usize = 24;

/// Why no key lies between two bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// The lower bound is not below the upper bound.
    NotAscending,
    /// A bound is empty, ends in `0`, or holds a character outside the digits.
    BadKey,
}

fn value(c: u8) -> Option<usize> {
    DIGITS.iter().position(|d| *d == c)
}

fn well_formed(key: &str) -> bool {
    !key.is_empty() && !key.ends_with('0') && key.bytes().all(|c| value(c).is_some())
}

/// A key strictly between `lower` and `upper`; `None` stands for 0 as a lower bound and for 1 as
/// an upper bound.
///
/// # Errors
/// [`KeyError::BadKey`] for a malformed bound, [`KeyError::NotAscending`] if `lower >= upper`.
pub fn between(lower: Option<&str>, upper: Option<&str>) -> Result<String, KeyError> {
    if lower.is_some_and(|k| !well_formed(k)) || upper.is_some_and(|k| !well_formed(k)) {
        return Err(KeyError::BadKey);
    }
    if let (Some(a), Some(b)) = (lower, upper)
        && a >= b
    {
        return Err(KeyError::NotAscending);
    }
    Ok(midpoint(
        lower.unwrap_or("").as_bytes(),
        upper.map(str::as_bytes),
    ))
}

fn midpoint(a: &[u8], b: Option<&[u8]>) -> String {
    if let Some(b) = b {
        // Strip the common prefix, `a` read as padded with `0`.
        let mut n = 0;
        while n < b.len() && a.get(n).copied().unwrap_or(b'0') == b[n] {
            n += 1;
        }
        if n > 0 {
            let rest_a = a.get(n..).unwrap_or(&[]);
            let mut out = String::from_utf8_lossy(&b[..n]).into_owned();
            out.push_str(&midpoint(rest_a, Some(&b[n..])));
            return out;
        }
    }
    let digit_a = a.first().and_then(|c| value(*c)).unwrap_or(0);
    let digit_b = b
        .and_then(|b| b.first())
        .and_then(|c| value(*c))
        .unwrap_or(62);
    if digit_b - digit_a > 1 {
        // Rounded down.
        return char::from(DIGITS[(digit_a + digit_b) / 2]).to_string();
    }
    if let Some(b) = b
        && b.len() > 1
    {
        // Adjacent digits and `b` has a tail: its first digit alone lies between.
        return char::from(b[0]).to_string();
    }
    let mut out = char::from(DIGITS[digit_a]).to_string();
    out.push_str(&midpoint(a.get(1..).unwrap_or(&[]), None));
    out
}

/// `n` evenly spaced keys in ascending order, as short as the count allows: 2 characters hold
/// 1 000 and more.
#[must_use]
pub fn spread(n: usize) -> Vec<String> {
    let mut len = 1u32;
    while 62u128.pow(len) < 2 * (n as u128 + 1) {
        len += 1;
    }
    let span = 62u128.pow(len);
    (1..=n)
        .map(|i| {
            let mut v = span * i as u128 / (n as u128 + 1);
            let mut digits = vec![b'0'; len as usize];
            for slot in digits.iter_mut().rev() {
                *slot = DIGITS[(v % 62) as usize];
                v /= 62;
            }
            while digits.last() == Some(&b'0') {
                digits.pop();
            }
            String::from_utf8_lossy(&digits).into_owned()
        })
        .collect()
}
