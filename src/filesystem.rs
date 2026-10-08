use nusave::{MAX_SAVE_SIZE, ParseError, Save};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A save could not be loaded from disk.
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("cannot open save {}: {source}", path.display())]
    Open {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot inspect save {}: {source}", path.display())]
    Metadata {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "save {} is {actual} bytes, exceeding the {limit}-byte safety limit",
        path.display()
    )]
    TooLarge {
        path: PathBuf,
        actual: u64,
        limit: u64,
    },
    #[error("cannot read save {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid save {}: {source}", path.display())]
    Invalid {
        path: PathBuf,
        #[source]
        source: ParseError,
    },
}

/// A save could not be published atomically.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("cannot create temporary output {}: {source}", path.display())]
    CreateTemporary {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "could not reserve a temporary output next to {} after 100 attempts",
        target.display()
    )]
    TemporaryNamesExhausted { target: PathBuf },
    #[error("cannot write temporary output {}: {source}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot flush temporary output {}: {source}", path.display())]
    Flush {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "cannot copy permissions from {} to {}: {source}",
        target.display(),
        temporary.display()
    )]
    Permissions {
        target: PathBuf,
        temporary: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot sync temporary output {}: {source}", path.display())]
    Sync {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot publish temporary output {} as {}: {source}", temporary.display(), target.display())]
    Publish {
        temporary: PathBuf,
        target: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub fn read_save(path: &Path) -> Result<Save, ReadError> {
    let file = File::open(path).map_err(|source| ReadError::Open {
        path: path.to_owned(),
        source,
    })?;
    let length = file
        .metadata()
        .map_err(|source| ReadError::Metadata {
            path: path.to_owned(),
            source,
        })?
        .len();
    if length > MAX_SAVE_SIZE {
        return Err(ReadError::TooLarge {
            path: path.to_owned(),
            actual: length,
            limit: MAX_SAVE_SIZE,
        });
    }
    let mut bytes = Vec::with_capacity(length as usize);
    file.take(MAX_SAVE_SIZE + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| ReadError::Read {
            path: path.to_owned(),
            source,
        })?;
    Save::parse(bytes).map_err(|source| ReadError::Invalid {
        path: path.to_owned(),
        source,
    })
}

fn temporary_path(path: &Path, attempt: u32) -> PathBuf {
    let mut value: OsString = path.as_os_str().to_owned();
    value.push(format!(".incomplete.{}.{attempt}", std::process::id()));
    PathBuf::from(value)
}

pub fn write_atomic(path: &Path, save: &Save, replace: bool) -> Result<(), WriteError> {
    let mut opened = None;
    for attempt in 0..100 {
        let temporary = temporary_path(path, attempt);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => {
                opened = Some((temporary, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(WriteError::CreateTemporary {
                    path: temporary,
                    source,
                });
            }
        }
    }
    let (temporary, mut file) = opened.ok_or_else(|| WriteError::TemporaryNamesExhausted {
        target: path.to_owned(),
    })?;
    let result = (|| -> Result<(), WriteError> {
        file.write_all(&save.bytes)
            .map_err(|source| WriteError::Write {
                path: temporary.clone(),
                source,
            })?;
        file.flush().map_err(|source| WriteError::Flush {
            path: temporary.clone(),
            source,
        })?;
        if replace && let Ok(metadata) = fs::metadata(path) {
            file.set_permissions(metadata.permissions())
                .map_err(|source| WriteError::Permissions {
                    target: path.to_owned(),
                    temporary: temporary.clone(),
                    source,
                })?;
        }
        file.sync_all().map_err(|source| WriteError::Sync {
            path: temporary.clone(),
            source,
        })?;
        drop(file);
        publish(&temporary, path, replace).map_err(|source| WriteError::Publish {
            temporary: temporary.clone(),
            target: path.to_owned(),
            source,
        })
    })();
    let _ = fs::remove_file(&temporary);
    result
}

#[cfg(not(windows))]
fn publish(temporary: &Path, path: &Path, replace: bool) -> io::Result<()> {
    if replace {
        fs::rename(temporary, path)
    } else {
        fs::hard_link(temporary, path)
    }
}

#[cfg(windows)]
fn publish(temporary: &Path, path: &Path, replace: bool) -> io::Result<()> {
    if !replace {
        return fs::hard_link(temporary, path);
    }
    use std::os::windows::ffi::OsStrExt;
    const MOVEFILE_REPLACE_EXISTING: u32 = 1;
    const MOVEFILE_WRITE_THROUGH: u32 = 8;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let existing: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
    let new: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: both pointers reference NUL-terminated UTF-16 buffers for the duration of the call.
    let ok = unsafe {
        MoveFileExW(
            existing.as_ptr(),
            new.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
