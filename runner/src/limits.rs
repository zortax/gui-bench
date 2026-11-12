//! The open file limit of the runner. One process runs every cell, and each fakeland session
//! keeps a few file descriptors after it ends (smithay's seat outlives the session with its
//! keymap memfd). A systemd service starts with a soft limit of 1024, which a full matrix
//! reaches after about 270 cells, so the runner raises the soft limit to the hard limit.

use rustix::process::{Resource, Rlimit, getrlimit, setrlimit};

/// Raises the soft limit of open files to the hard limit.
pub fn raise_open_files() {
    let limit = getrlimit(Resource::Nofile);
    let raised = Rlimit {
        current: limit.maximum,
        maximum: limit.maximum,
    };
    if let Err(e) = setrlimit(Resource::Nofile, raised) {
        eprintln!("the open file limit stays at {:?}: {e}", limit.current);
    }
}
