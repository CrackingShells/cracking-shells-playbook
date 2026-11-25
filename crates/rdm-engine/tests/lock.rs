//! Guards of the writer lock, with real child processes and real `fs4` locks.

mod common;

use common::{LIMIT, child_role, env_u64, say, spawn_child};
use rdm_engine::lock::{LockError, WriterLock};
use std::path::Path;
use std::time::{Duration, Instant};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn holder_file(root: &Path) -> std::path::PathBuf {
    root.join(".rdm").join("holder.json")
}

/// The child of `second_writer_gets_locked_with_holder`: try for 300 ms and say what happened.
fn contender_child(root: &Path) {
    match WriterLock::acquire(root, Duration::from_millis(300), b"contender") {
        Ok(_lock) => say("acquired"),
        Err(LockError::Busy {
            holder: Some(bytes),
        }) => say(&format!("busy {}", hex(&bytes))),
        Err(LockError::Busy { holder: None }) => say("busy none"),
        Err(LockError::Io(err)) => say(&format!("io {err}")),
    }
}

#[test]
fn second_writer_gets_locked_with_holder() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "contender");
        return contender_child(&dir);
    }
    let root = tempfile::tempdir().unwrap();
    let record = br#"{"pid":4242,"host":"h","since":"t"}"#;
    let lock = WriterLock::acquire(root.path(), Duration::from_secs(5), record).unwrap();

    let child = spawn_child(
        "second_writer_gets_locked_with_holder",
        "contender",
        root.path(),
        &[],
    );
    let said = child.wait_line("", LIMIT);
    assert_eq!(
        said,
        format!("busy {}", hex(record)),
        "the second process must be refused with the holder's record"
    );
    assert_eq!(std::fs::read(holder_file(root.path())).unwrap(), record);

    lock.release();
    assert!(
        !holder_file(root.path()).exists(),
        "the holder record is removed before release"
    );
    drop(child);
    // Once released, another process acquires at once.
    let child = spawn_child(
        "second_writer_gets_locked_with_holder",
        "contender",
        root.path(),
        &[],
    );
    assert_eq!(child.wait_line("", LIMIT), "acquired");
}

/// The child of `lock_survives_holder_kill`: take the lock, say so, then wait to be killed.
fn holder_child(root: &Path) {
    let _lock = WriterLock::acquire(root, Duration::from_secs(10), b"the-killed-holder").unwrap();
    say("locked");
    std::thread::sleep(Duration::from_secs(120)); // bounded; the parent kills this process
}

#[test]
fn lock_survives_holder_kill() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "holder");
        return holder_child(&dir);
    }
    let root = tempfile::tempdir().unwrap();
    let mut child = spawn_child("lock_survives_holder_kill", "holder", root.path(), &[]);
    child.wait_line("locked", LIMIT);
    // The child really holds it.
    match WriterLock::acquire(root.path(), Duration::from_millis(100), b"parent") {
        Err(LockError::Busy {
            holder: Some(bytes),
        }) => assert_eq!(bytes, b"the-killed-holder"),
        other => panic!("expected Busy while the child lives, got {other:?}"),
    }
    // SIGKILL (TerminateProcess on Windows): no destructor runs, so the record stays behind.
    child.kill_and_reap();
    assert_eq!(
        std::fs::read(holder_file(root.path())).unwrap(),
        b"the-killed-holder"
    );
    // Windows releases a dead process's lock after a short delay: the timeout covers it.
    let lock = WriterLock::acquire(root.path(), Duration::from_secs(20), b"the-next-holder")
        .expect("the lock must be free once its holder was killed");
    assert_eq!(
        std::fs::read(holder_file(root.path())).unwrap(),
        b"the-next-holder"
    );
    drop(lock);
}

#[test]
fn garbage_holder_record_does_not_break_locking() {
    let garbage: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("binary", vec![0xff, 0xfe, 0x00, 0x80, b'{', 0x00]),
        ("truncated json", br#"{"pid": 12, "host": "a"#.to_vec()),
        ("json of another shape", br#"["pid", 1]"#.to_vec()),
        ("large", vec![0xaa; 1 << 20]),
    ];
    for (what, bytes) in &garbage {
        // Garbage left by a crashed writer or a person: the lock is still taken, and replaced it.
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".rdm")).unwrap();
        std::fs::write(holder_file(root.path()), bytes).unwrap();
        let lock = WriterLock::acquire(root.path(), Duration::from_secs(5), b"fresh")
            .unwrap_or_else(|e| panic!("{what}: a garbage record blocked the lock: {e:?}"));
        assert_eq!(
            std::fs::read(holder_file(root.path())).unwrap(),
            b"fresh",
            "{what}"
        );

        // Garbage appearing while the lock is held is only reported, to a contender, as bytes.
        std::fs::write(holder_file(root.path()), bytes).unwrap();
        match WriterLock::acquire(root.path(), Duration::from_millis(50), b"other") {
            Err(LockError::Busy { holder }) => {
                assert_eq!(holder.as_deref(), Some(bytes.as_slice()), "{what}")
            }
            other => panic!("{what}: expected Busy, got {other:?}"),
        }
        // A missing record is an unknown holder, still Busy.
        std::fs::remove_file(holder_file(root.path())).unwrap();
        match WriterLock::acquire(root.path(), Duration::from_millis(50), b"other") {
            Err(LockError::Busy { holder: None }) => {}
            other => panic!("{what}: expected Busy with no holder, got {other:?}"),
        }
        // Release with the record gone or garbage never fails, and the lock is free after.
        drop(lock);
        let again = WriterLock::acquire(root.path(), Duration::from_secs(5), b"again")
            .unwrap_or_else(|e| panic!("{what}: the lock was not released: {e:?}"));
        drop(again);
        assert!(!holder_file(root.path()).exists(), "{what}");
    }
}

/// The child of the two waiting guards: take the lock, say so, keep it for `RDM_HOLD_MS`
/// (forever if unset, until killed), then release.
fn timed_holder_child(root: &Path) {
    let lock = WriterLock::acquire(root, Duration::from_secs(10), b"timed-holder").unwrap();
    say("locked");
    match env_u64("RDM_HOLD_MS", u64::MAX) {
        u64::MAX => std::thread::sleep(Duration::from_secs(120)), // bounded; the parent kills it
        ms => {
            std::thread::sleep(Duration::from_millis(ms));
            lock.release();
            say("released");
        }
    }
}

#[test]
fn a_writer_waits_for_a_holder_that_releases_inside_the_timeout() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "timed");
        return timed_holder_child(&dir);
    }
    let root = tempfile::tempdir().unwrap();
    let mut child = spawn_child(
        "a_writer_waits_for_a_holder_that_releases_inside_the_timeout",
        "timed",
        root.path(),
        &[("RDM_HOLD_MS", "200")],
    );
    child.wait_line("locked", LIMIT);
    let started = Instant::now();
    let lock = WriterLock::acquire(root.path(), Duration::from_secs(30), b"waiter")
        .expect("the waiter must get the lock once the holder releases");
    assert!(
        started.elapsed() >= Duration::from_millis(100),
        "it did not wait for the holder"
    );
    drop(lock);
    assert!(child.child.wait().unwrap().success());
}

#[test]
fn a_writer_gives_up_with_busy_at_the_timeout_and_not_before_or_long_after() {
    if let Some((role, dir)) = child_role() {
        assert_eq!(role, "timed");
        return timed_holder_child(&dir);
    }
    let root = tempfile::tempdir().unwrap();
    let child = spawn_child(
        "a_writer_gives_up_with_busy_at_the_timeout_and_not_before_or_long_after",
        "timed",
        root.path(),
        &[],
    );
    child.wait_line("locked", LIMIT);
    let timeout = Duration::from_secs(1);
    let started = Instant::now();
    let result = WriterLock::acquire(root.path(), timeout, b"waiter");
    let waited = started.elapsed();
    assert!(matches!(result, Err(LockError::Busy { .. })), "{result:?}");
    assert!(
        waited >= timeout.mul_f64(0.9),
        "gave up after {waited:?}, before the timeout"
    );
    assert!(
        waited < Duration::from_secs(15),
        "gave up after {waited:?}, long after the timeout"
    );
}
