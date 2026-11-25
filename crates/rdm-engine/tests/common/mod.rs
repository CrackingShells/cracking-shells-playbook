//! Child-process plumbing shared by the guards. The child is this test binary itself,
//! re-invoked on one test by name, with `RDM_CHILD_ROLE` and `RDM_CHILD_DIR` selecting what it does.

#![allow(dead_code)]

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// How long a parent waits for a line from a child.
pub const LIMIT: Duration = Duration::from_secs(60);

/// A numeric knob from the environment, `default` when unset or unparsable.
pub fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub const ROLE: &str = "RDM_CHILD_ROLE";
pub const DIR: &str = "RDM_CHILD_DIR";

/// The role this process was started with, if it is a child.
pub fn child_role() -> Option<(String, PathBuf)> {
    let role = std::env::var(ROLE).ok()?;
    let dir = std::env::var(DIR).ok()?;
    Some((role, PathBuf::from(dir)))
}

/// A running child with its stdout lines (those starting with `!`) arriving on a channel.
pub struct ChildProc {
    pub child: Child,
    pub lines: Receiver<String>,
    pub stdin: Option<ChildStdin>,
}

pub fn spawn_child(
    test_name: &str,
    role: &str,
    dir: &Path,
    extra_env: &[(&str, &str)],
) -> ChildProc {
    let mut cmd = Command::new(std::env::current_exe().expect("current_exe"));
    cmd.args([test_name, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE, role)
        .env(DIR, dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("spawn child");
    let stdout = child.stdout.take().expect("child stdout");
    let stdin = child.stdin.take();
    let (tx, lines) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(rest) = line.strip_prefix('!')
                && tx.send(rest.to_string()).is_err()
            {
                break;
            }
        }
    });
    ChildProc {
        child,
        lines,
        stdin,
    }
}

impl ChildProc {
    /// Wait for a line starting with `prefix`, up to `limit`; other lines are skipped.
    pub fn wait_line(&self, prefix: &str, limit: Duration) -> String {
        let deadline = Instant::now() + limit;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(line) if line.starts_with(prefix) => return line,
                Ok(_) => continue,
                Err(_) => panic!("child never printed a line starting with {prefix:?}"),
            }
        }
    }

    /// Wait for the child to exit, which it must do successfully.
    pub fn wait_ok(&mut self) {
        self.stdin = None;
        let status = self.child.wait().expect("wait for the child");
        assert!(status.success(), "the child failed: {status}");
    }

    /// Close the child's stdin, which tells a waiting child to finish.
    pub fn close_stdin(&mut self) {
        self.stdin = None;
    }

    pub fn kill_and_reap(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ChildProc {
    fn drop(&mut self) {
        self.kill_and_reap();
    }
}

/// Print a protocol line to the parent and flush.
pub fn say(line: &str) {
    use std::io::Write;
    // A leading newline: libtest has just printed "test <name> ... " on this line.
    println!("\n!{line}");
    let _ = std::io::stdout().flush();
}

pub mod campaign;
