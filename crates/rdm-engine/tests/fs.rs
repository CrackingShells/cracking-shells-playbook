//! Guards of `atomic_replace`: a reader in another process, and a target held open.

mod common;

use common::{child_role, say, spawn_child};
use rdm_engine::fs::{Durability, RealFs, atomic_replace};
use std::path::Path;
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(60);

/// Version `i` of the file: one repeated byte, with a length that depends on the byte, so a
/// truncated, empty or mixed read is recognisable whatever the point it was torn at.
fn version(i: usize) -> Vec<u8> {
    let k = i % 7;
    vec![b'a' + k as u8; 1000 + k * 50_000]
}

fn is_whole_version(bytes: &[u8]) -> bool {
    match bytes.first() {
        Some(&c) if (b'a'..b'a' + 7).contains(&c) => {
            let k = (c - b'a') as usize;
            bytes.len() == 1000 + k * 50_000 && bytes.iter().all(|&b| b == c)
        }
        _ => false,
    }
}

fn reader_child(dir: &Path) {
    let target = dir.join("node.json");
    let stop = dir.join("stop");
    let (mut reads, mut bad) = (0u64, 0u64);
    let mut first_bad = String::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(50);
    loop {
        match std::fs::read(&target) {
            Ok(bytes) if is_whole_version(&bytes) => {}
            Ok(bytes) => {
                bad += 1;
                if first_bad.is_empty() {
                    first_bad = format!("a read of {} bytes", bytes.len());
                }
            }
            Err(e) => {
                bad += 1;
                if first_bad.is_empty() {
                    first_bad = format!("a failed read: {e}");
                }
            }
        }
        reads += 1;
        if reads % 25 == 0 {
            say("tick");
            if stop.exists() || std::time::Instant::now() > deadline {
                break;
            }
        }
    }
    say(&format!("result reads={reads} bad={bad} first={first_bad}"));
}

#[test]
fn reader_sees_old_or_new_never_partial() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "reader");
        return reader_child(&dir);
    }
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("node.json");
    let tmp = dir.path().join(".rdm").join("tmp");
    atomic_replace(&RealFs, &target, &tmp, &version(0), Durability::Fsync).unwrap();

    let mut child = spawn_child(
        "reader_sees_old_or_new_never_partial",
        "reader",
        dir.path(),
        &[],
    );
    // Replace until the reader has made progress during the replaces, bounded in time.
    let started = std::time::Instant::now();
    let (mut i, mut ticks) = (1usize, 0usize);
    while (i < 300 || ticks < 4) && started.elapsed() < Duration::from_secs(20) {
        atomic_replace(&RealFs, &target, &tmp, &version(i), Durability::Fsync).unwrap();
        while child.lines.try_recv().is_ok() {
            ticks += 1;
        }
        i += 1;
    }
    std::fs::write(dir.path().join("stop"), b"").unwrap();
    let result = child.wait_line("result", LIMIT);
    assert!(child.child.wait().unwrap().success());
    assert!(
        result.contains(" bad=0 "),
        "the reader saw a torn file: {result}"
    );
    assert!(
        ticks >= 1,
        "the reader made no progress while files were replaced: {result}"
    );
    assert!(is_whole_version(&std::fs::read(&target).unwrap()));
    // No temporary file is left behind.
    assert_eq!(std::fs::read_dir(&tmp).unwrap().count(), 0);
}

/// How long the child keeps the target open. Short enough to be inside the replace's retry
/// window (about 555 ms); the replace must wait it out.
const HOLD_SHORT: Duration = Duration::from_millis(150);

/// Open `target` the way a foreign program that denies deletion does (no `FILE_SHARE_DELETE`
/// on Windows; on Unix any open file will do), hold it for the time in `RDM_HOLD_MS`, release.
fn holder_child(dir: &Path) {
    let target = dir.join("node.json");
    let hold: u64 = std::env::var("RDM_HOLD_MS").unwrap().parse().unwrap();
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        opts.share_mode(1); // FILE_SHARE_READ only
    }
    let file = opts.open(&target).unwrap();
    say("holding");
    std::thread::sleep(Duration::from_millis(hold));
    drop(file);
    say("released");
}

#[test]
fn replace_while_target_open() {
    // On macOS and Linux an open file never blocks a rename, so this passes trivially there.
    // It matters on the Windows CI runner, where the child's handle denies deletion and the
    // replace succeeds only through the retry.
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "holder");
        return holder_child(&dir);
    }
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("node.json");
    let tmp = dir.path().join(".rdm").join("tmp");
    atomic_replace(&RealFs, &target, &tmp, b"old", Durability::Fsync).unwrap();

    let hold = HOLD_SHORT.as_millis().to_string();
    let mut child = spawn_child(
        "replace_while_target_open",
        "holder",
        dir.path(),
        &[("RDM_HOLD_MS", &hold)],
    );
    child.wait_line("holding", LIMIT);
    atomic_replace(&RealFs, &target, &tmp, b"new", Durability::Fsync)
        .expect("replace while the target is open");
    assert_eq!(std::fs::read(&target).unwrap(), b"new");
    child.wait_line("released", LIMIT);
    assert!(child.child.wait().unwrap().success());
    assert_eq!(std::fs::read_dir(&tmp).unwrap().count(), 0);
}

/// Windows only: a target held longer than the retry window fails cleanly, leaves the old
/// bytes, and leaves no temporary file.
#[cfg(windows)]
#[test]
fn replace_fails_cleanly_when_the_target_stays_open() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "holder");
        return holder_child(&dir);
    }
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("node.json");
    let tmp = dir.path().join(".rdm").join("tmp");
    atomic_replace(&RealFs, &target, &tmp, b"old", Durability::Fsync).unwrap();
    let mut child = spawn_child(
        "replace_fails_cleanly_when_the_target_stays_open",
        "holder",
        dir.path(),
        &[("RDM_HOLD_MS", "3000")],
    );
    child.wait_line("holding", LIMIT);
    let err = atomic_replace(&RealFs, &target, &tmp, b"new", Durability::Fsync).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Other
        ),
        "{err}"
    );
    assert!(
        err.to_string().contains("os error"),
        "the error names the code: {err}"
    );
    assert_eq!(std::fs::read(&target).unwrap(), b"old");
    assert_eq!(std::fs::read_dir(&tmp).unwrap().count(), 0);
    child.wait_line("released", LIMIT);
}
