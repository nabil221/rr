//! Native-only composition: filesystem selection stays outside shared core/UI.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use local_stack_proof_application::RunService;
use local_stack_proof_persistence_sqlite::SqliteRunRepository;

pub fn unavailable() -> io::Error {
    io::Error::other("Desktop local storage is unavailable")
}

pub fn data_directory(default: PathBuf, override_path: Option<OsString>) -> io::Result<PathBuf> {
    let directory = override_path.map_or(default, PathBuf::from);
    if !directory.is_absolute() || directory.parent().is_none() {
        return Err(io::Error::other(
            "Desktop data directory must be an absolute local directory",
        ));
    }
    // SQLite belongs on a local filesystem, not a Windows network share/device.
    #[cfg(windows)]
    if directory.components().any(|part| matches!(part,
        std::path::Component::Prefix(prefix) if !matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))) {
        return Err(io::Error::other("Desktop data directory must be an absolute local directory"));
    }
    Ok(directory)
}

pub fn service(directory: &Path) -> io::Result<RunService> {
    std::fs::create_dir_all(directory).map_err(|_| unavailable())?;
    SqliteRunRepository::open(&directory.join("runs.sqlite3"))
        .map(SqliteRunRepository::service)
        .map_err(|_| unavailable())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_isolated_override_are_explicit_and_local() {
        let directory = tempfile::tempdir().unwrap();
        let default = directory.path().join("default");
        assert_eq!(data_directory(default.clone(), None).unwrap(), default);
        let isolated = directory.path().join("isolated");
        assert_eq!(
            data_directory(default.clone(), Some(isolated.clone().into_os_string())).unwrap(),
            isolated
        );
        for invalid in ["", "relative/path"] {
            assert!(data_directory(default.clone(), Some(invalid.into())).is_err());
        }
        #[cfg(windows)]
        assert!(data_directory(default, Some(r"\\server\share\proof".into())).is_err());
        let first = service(&isolated).unwrap();
        let run = first
            .create(local_stack_proof_domain_configuration())
            .unwrap();
        drop(first);
        assert_eq!(service(&isolated).unwrap().get(run.id()).unwrap(), run);
        assert!(!isolated.join("unrelated").exists());
    }

    fn local_stack_proof_domain_configuration() -> local_stack_proof_domain::RunConfiguration {
        local_stack_proof_domain::RunConfiguration::new(42, "north", 0.0, false).unwrap()
    }
}
