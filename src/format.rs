use crate::layout::{GameSave, SaveHeader};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zerocopy::FromBytes;
use zerocopy::byteorder::little_endian::U32;

pub const HEADER_SIZE: usize = 0x2028;
pub const ANDROID_GAME_SIZE: usize = 0x7e58;
pub const WINDOWS_GAME_SIZE: usize = 0x7e4c;
/// Size used when creating a new game save. New saves target the Android layout.
pub const GAME_SIZE: usize = ANDROID_GAME_SIZE;
pub const OPTIONS_SIZE: usize = 0x18;
const MAX_SAVE_SIZE: u64 = 16 * 1024 * 1024;

pub fn save_file_name(slot: u8) -> String {
    format!("SaveGame{slot}.LEGO Star Wars - The Complete Saga_SavedGame")
}

/// A byte buffer is not a supported Nu save envelope.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    #[error("save is only {actual} bytes; a save envelope needs at least {minimum} bytes")]
    TooShort { actual: usize, minimum: usize },
    #[error("invalid save magic 0x{found:08x}; expected 0x52474d48")]
    InvalidMagic { found: i32 },
    #[error("unsupported save header version {found}; expected version 1")]
    UnsupportedVersion { found: i32 },
    #[error("invalid header size {found}; expected {expected} bytes")]
    InvalidHeaderSize { found: i32, expected: usize },
    #[error("extra-data offset is negative ({found})")]
    NegativeExtraDataOffset { found: i32 },
    #[error("extra-data offset {found} cannot be represented on this platform")]
    ExtraDataOffsetOverflow { found: i32 },
    #[error(
        "payload offset {payload_offset} leaves no room for the trailing 8-byte footer in a {file_size}-byte save"
    )]
    PayloadBeyondFile {
        payload_offset: usize,
        file_size: usize,
    },
    #[error(
        "unsupported payload size {found} bytes; expected {windows_game} for a Windows game save, {android_game} for an Android game save, or {options} for Android SuperOptions"
    )]
    UnsupportedPayloadSize {
        found: usize,
        windows_game: usize,
        android_game: usize,
        options: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveKind {
    WindowsGame,
    AndroidGame,
    AndroidOptions,
}

impl SaveKind {
    pub fn description(self) -> &'static str {
        match self {
            Self::WindowsGame => "Windows PC game progress",
            Self::AndroidGame => "Android game progress",
            Self::AndroidOptions => "Android SuperOptions",
        }
    }

    pub fn is_game(self) -> bool {
        matches!(self, Self::WindowsGame | Self::AndroidGame)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HeaderMetadata {
    pub application: Option<String>,
    pub slot: Option<String>,
    pub reserved: Option<String>,
    pub timestamp: Option<String>,
}

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

#[derive(Clone, Debug)]
pub struct Save {
    pub bytes: Vec<u8>,
    pub payload: usize,
    pub payload_size: usize,
}

impl Save {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < HEADER_SIZE + 8 {
            return Err(ParseError::TooShort {
                actual: bytes.len(),
                minimum: HEADER_SIZE + 8,
            });
        }
        let (header, _) =
            SaveHeader::ref_from_prefix(&bytes).map_err(|_| ParseError::TooShort {
                actual: bytes.len(),
                minimum: HEADER_SIZE,
            })?;
        if header.magic.get() != 0x5247_4d48 {
            return Err(ParseError::InvalidMagic {
                found: header.magic.get(),
            });
        }
        if header.version.get() != 1 {
            return Err(ParseError::UnsupportedVersion {
                found: header.version.get(),
            });
        }
        if header.size.get() != HEADER_SIZE as i32 {
            return Err(ParseError::InvalidHeaderSize {
                found: header.size.get(),
                expected: HEADER_SIZE,
            });
        }
        let extra = header.extra_data_offset.get();
        if extra < 0 {
            return Err(ParseError::NegativeExtraDataOffset { found: extra });
        }
        let extra = usize::try_from(extra)
            .map_err(|_| ParseError::ExtraDataOffsetOverflow { found: extra })?;
        let payload =
            HEADER_SIZE
                .checked_add(extra)
                .ok_or(ParseError::ExtraDataOffsetOverflow {
                    found: header.extra_data_offset.get(),
                })?;
        if payload > bytes.len() - 8 {
            return Err(ParseError::PayloadBeyondFile {
                payload_offset: payload,
                file_size: bytes.len(),
            });
        }
        let payload_size = bytes.len() - payload - 8;
        if payload_size != ANDROID_GAME_SIZE
            && payload_size != WINDOWS_GAME_SIZE
            && payload_size != OPTIONS_SIZE
        {
            return Err(ParseError::UnsupportedPayloadSize {
                found: payload_size,
                windows_game: WINDOWS_GAME_SIZE,
                android_game: ANDROID_GAME_SIZE,
                options: OPTIONS_SIZE,
            });
        }
        Ok(Self {
            bytes,
            payload,
            payload_size,
        })
    }

    pub fn read(path: &Path) -> Result<Self, ReadError> {
        let mut file = File::open(path).map_err(|source| ReadError::Open {
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
        file.read_to_end(&mut bytes)
            .map_err(|source| ReadError::Read {
                path: path.to_owned(),
                source,
            })?;
        Self::parse(bytes).map_err(|source| ReadError::Invalid {
            path: path.to_owned(),
            source,
        })
    }

    pub fn fresh(options: bool) -> Self {
        let payload_size = if options { OPTIONS_SIZE } else { GAME_SIZE };
        let mut save = Self {
            bytes: vec![0; HEADER_SIZE + payload_size + 8],
            payload: HEADER_SIZE,
            payload_size,
        };
        let (header, _) = SaveHeader::mut_from_prefix(&mut save.bytes)
            .expect("new save always contains a complete header");
        header.magic.set(0x5247_4d48);
        header.version.set(1);
        header.size.set(HEADER_SIZE as i32);

        if !options {
            let game =
                GameSave::mut_from_bytes(&mut save.bytes[HEADER_SIZE..HEADER_SIZE + GAME_SIZE])
                    .expect("new game payload has the exact game layout size");
            // Deterministic NewGame state before any game assets are configured.
            game.prefix.difficulty = 5;
            game.prefix.areas[0].complete = 1;
            for episode in &mut game.prefix.episodes {
                episode.superstory_time_limit.set(3600.0);
                episode.superstory_score_target.set(100_000);
            }
            game.prefix.suit_flags.set(0x21);
            game.suffix.customizer.primary_use_saved_name = 1;
            game.suffix.customizer.secondary_use_saved_name = 1;
        }
        save.derive();
        save
    }

    pub fn validate_layout(&self) -> bool {
        match Self::parse(self.bytes.clone()) {
            Ok(parsed) => {
                parsed.payload == self.payload && parsed.payload_size == self.payload_size
            }
            Err(_) => false,
        }
    }

    pub fn checksum(&self) -> u32 {
        let words =
            <[U32]>::ref_from_bytes(&self.bytes[self.payload..self.payload + self.payload_size])
                .expect("supported payload sizes are a multiple of four");
        words
            .iter()
            .fold(0x005c_0999u32, |sum, word| sum.wrapping_add(word.get()))
    }

    pub fn checksum_valid(&self) -> bool {
        self.read_u32(self.payload + self.payload_size) == self.checksum()
    }

    pub fn derive(&mut self) {
        let checksum_offset = self.payload + self.payload_size;
        self.write_u32(checksum_offset, self.checksum());
        let slot_code = match self.kind() {
            SaveKind::AndroidGame => {
                let game = GameSave::ref_from_bytes(
                    &self.bytes[self.payload..self.payload + self.payload_size],
                )
                .expect("validated Android game payload");
                let completion = game.android_summary.completion.get();
                (completion as i16 as i32) as u32
            }
            // The PC payload has no copy of the completion value used to make
            // its slot code. Preserve the real stored value when other fields
            // are edited instead of inventing one.
            SaveKind::WindowsGame => self.read_u32(self.bytes.len() - 4),
            SaveKind::AndroidOptions => u32::MAX,
        };
        self.write_u32(self.bytes.len() - 4, slot_code);
    }

    pub fn kind(&self) -> SaveKind {
        match self.payload_size {
            WINDOWS_GAME_SIZE => SaveKind::WindowsGame,
            ANDROID_GAME_SIZE => SaveKind::AndroidGame,
            OPTIONS_SIZE => SaveKind::AndroidOptions,
            _ => unreachable!("Save can only contain a validated payload size"),
        }
    }

    pub fn slot_code(&self) -> u32 {
        self.read_u32(self.bytes.len() - 4)
    }

    pub fn header_metadata(&self) -> HeaderMetadata {
        let (header, _) = SaveHeader::ref_from_prefix(&self.bytes)
            .expect("validated save always contains a complete header");
        HeaderMetadata {
            application: utf16_prefix(&header.application_metadata),
            slot: utf16_prefix(&header.slot_metadata),
            reserved: utf16_prefix(&header.reserved_metadata),
            timestamp: utf16_prefix(&header.timestamp_metadata),
        }
    }

    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("validated word offset"),
        )
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

fn utf16_prefix(bytes: &[u8]) -> Option<String> {
    let units = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    if units.is_empty() {
        None
    } else {
        Some(String::from_utf16_lossy(&units))
    }
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
