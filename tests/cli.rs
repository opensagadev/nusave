use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const HEADER: usize = 0x2028;
const PAYLOAD: usize = 0x7e58;
const WINDOWS_PAYLOAD: usize = 0x7e4c;
const SAVE_NAME: &str = "SaveGame0.LEGO Star Wars - The Complete Saga_SavedGame";
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("nusave-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn save(&self) -> PathBuf {
        self.0.join(SAVE_NAME)
    }

    fn save_slot(&self, slot: u8) -> PathBuf {
        self.0
            .join(SAVE_NAME.replacen("SaveGame0", &format!("SaveGame{slot}"), 1))
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(directory: &TestDirectory, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nusave"))
        .env("COLUMNS", "20")
        .arg(directory.path())
        .args(args)
        .output()
        .expect("run nusave")
}

fn success(directory: &TestDirectory, args: &[&str]) -> Output {
    let output = run(directory, args);
    assert!(
        output.status.success(),
        "nusave failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn assert_checksum(bytes: &[u8], payload_size: usize) {
    let expected = bytes[HEADER..HEADER + payload_size]
        .chunks(4)
        .fold(0x5c0999u32, |sum, chunk| {
            sum.wrapping_add(u32::from_le_bytes(chunk.try_into().unwrap()))
        });
    assert_eq!(word(bytes, HEADER + payload_size), expected);
}

fn write_word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_utf16_prefix(bytes: &mut [u8], offset: usize, value: &str) {
    for (index, unit) in value.encode_utf16().chain([0]).enumerate() {
        bytes[offset + index * 2..offset + index * 2 + 2].copy_from_slice(&unit.to_le_bytes());
    }
}

fn convert_to_windows_save(bytes: &mut Vec<u8>) {
    bytes.drain(HEADER + 0x7c20..HEADER + 0x7c2c);
    write_utf16_prefix(bytes, 0x28, "LEGO® Star Wars™: The Complete Saga");
    write_utf16_prefix(bytes, 0x828, "Save Slot 0");
    write_utf16_prefix(bytes, 0x1828, "9/6/2026 3:44:24 AM");
    bytes[HEADER + 0x7c20..HEADER + 0x7c24].copy_from_slice(&120.0f32.to_le_bytes());
    bytes[HEADER + 0x7cf8 + 339] = 3;
    write_word(bytes, HEADER + WINDOWS_PAYLOAD + 4, 7);
    let checksum = bytes[HEADER..HEADER + WINDOWS_PAYLOAD]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0x5c0999u32, |sum, chunk| {
            sum.wrapping_add(u32::from_le_bytes(*chunk))
        });
    write_word(bytes, HEADER + WINDOWS_PAYLOAD, checksum);
}

#[test]
fn creates_game_saves_and_applies_typed_edits() {
    let directory = TestDirectory::new();
    success(
        &directory,
        &[
            "create",
            "coins=12345",
            "completion=32769",
            "area_save[2].minikit_count=10",
            "episode_save[1].superstory_time_limit=12.5",
            "customizer.primary_name=text:TEST",
            "customizer.secondary_pieces[8]=-123",
            "character_save[339]=3",
        ],
    );
    let bytes = fs::read(directory.save()).unwrap();
    assert_eq!(bytes.len(), HEADER + PAYLOAD + 8);
    assert_eq!(
        &bytes[..12],
        &[b'H', b'M', b'G', b'R', 1, 0, 0, 0, 0x28, 0x20, 0, 0]
    );
    assert_eq!(bytes[HEADER + 1], 5);
    assert_eq!(word(&bytes, HEADER + 0x7c20), 12345);
    assert_eq!(bytes[HEADER + 0x782c + 2 * 12 + 5], 10);
    assert_eq!(
        f32::from_le_bytes(
            bytes[HEADER + 0x7b8c + 12..HEADER + 0x7b8c + 16]
                .try_into()
                .unwrap()
        ),
        12.5
    );
    assert_eq!(&bytes[HEADER + 0x7c44..HEADER + 0x7c49], b"TEST\0");
    assert_eq!(bytes[HEADER + 0x7d04 + 339], 3);
    assert_eq!(word(&bytes, bytes.len() - 4), 0xffff8001);
    assert_checksum(&bytes, PAYLOAD);
}

#[test]
fn create_makes_a_missing_savegame_folder() {
    let directory = TestDirectory::new();
    let nested = directory.path().join("missing").join("SavedGames");
    let output = Command::new(env!("CARGO_BIN_EXE_nusave"))
        .arg(&nested)
        .arg("create")
        .output()
        .expect("run nusave");
    assert!(
        output.status.success(),
        "nusave failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(nested.join(SAVE_NAME).is_file());
}

#[test]
fn listing_is_a_lossless_parameter_file() {
    let source = TestDirectory::new();
    success(&source, &["create"]);
    let mut bytes = fs::read(source.save()).unwrap();
    bytes[0x2a..0x32].copy_from_slice(b"opaque!!");
    bytes[HEADER + 0x7c2c..HEADER + 0x7c30].copy_from_slice(&0x7fc01234u32.to_le_bytes());
    bytes.splice(HEADER..HEADER, *b"odd");
    bytes[0x14..0x18].copy_from_slice(&3i32.to_le_bytes());
    fs::write(source.save(), &bytes).unwrap();

    let listed = success(&source, &["list", "--raw"]).stdout;
    let params = source.path().join("all.params");
    fs::write(&params, listed).unwrap();

    let destination = TestDirectory::new();
    success(
        &destination,
        &[
            "create",
            "--from",
            source.save().to_str().unwrap(),
            "--params",
            params.to_str().unwrap(),
            "--keep-derived",
        ],
    );
    assert_eq!(fs::read(destination.save()).unwrap(), bytes);
}

#[test]
fn invalid_edits_never_replace_the_input() {
    let directory = TestDirectory::new();
    success(&directory, &["create"]);
    let original = fs::read(directory.save()).unwrap();
    for invalid in [
        "coins=-1",
        "coins=4294967296",
        "missing=1",
        "character_save[340]=1",
        "byte[999999]=1",
        "customizer.secondary_pieces[0]=-32769",
        "header.extradata_offset=1",
        "header.size=1",
        "customizer.primary_name=hex:gg",
    ] {
        let output = run(&directory, &["edit", "coins=2", invalid]);
        assert!(!output.status.success(), "unexpectedly accepted {invalid}");
        assert_eq!(fs::read(directory.save()).unwrap(), original);
    }
    assert!(!run(&directory, &["create"]).status.success());
    assert_eq!(fs::read(directory.save()).unwrap(), original);
}

#[test]
fn supports_options_saves_and_non_destructive_output() {
    let directory = TestDirectory::new();
    success(
        &directory,
        &[
            "create",
            "--options",
            "options.left_control_x=0.25",
            "options.store_pack_flags=EPISODE_II|SITH",
            "options.store_bundle_flags=PREQUEL|COMPLETE",
        ],
    );
    let original = fs::read(directory.save_slot(3)).unwrap();
    assert_eq!(original.len(), HEADER + 24 + 8);
    assert_checksum(&original, 24);
    assert_eq!(&original[original.len() - 4..], &[0xff; 4]);
    let copy = directory.path().join("copy.sav");
    success(
        &directory,
        &[
            "edit",
            "--slot",
            "3",
            "--output",
            copy.to_str().unwrap(),
            "options.music_enabled=ON",
        ],
    );
    assert_eq!(fs::read(directory.save_slot(3)).unwrap(), original);
    assert_ne!(fs::read(copy).unwrap(), original);
}

#[test]
fn reads_and_edits_the_shared_windows_precursor_layout() {
    let directory = TestDirectory::new();
    success(&directory, &["create"]);
    let mut bytes = fs::read(directory.save()).unwrap();
    convert_to_windows_save(&mut bytes);
    fs::write(directory.save(), &bytes).unwrap();

    let listed = String::from_utf8(success(&directory, &["list"]).stdout).unwrap();
    assert!(listed.contains("Windows PC game progress"));
    assert!(listed.contains("Application  LEGO® Star Wars™: The Complete Saga"));
    assert!(listed.contains("Slot label   Save Slot 0"));
    assert!(listed.contains("Saved at     9/6/2026 3:44:24 AM"));
    assert!(listed.contains("Stored slot code  7"));
    assert!(listed.contains("Gameplay time"));
    assert!(listed.contains("2m 00s"));
    assert!(listed.contains("Unknown character (ID 339)"));
    assert!(!listed.contains("  Studs"));

    success(
        &directory,
        &[
            "edit",
            "gameplay_seconds=90",
            "character_save[339]=UNLOCKED",
            "shop_character_purchased_bits=macewindu_ep3",
            "extra_purchased_bits=adaptivedifficulty",
            "hint_completion_bits=bank1.GizForce_601",
        ],
    );
    let edited = fs::read(directory.save()).unwrap();
    assert_eq!(edited.len(), HEADER + WINDOWS_PAYLOAD + 8);
    assert_eq!(
        f32::from_le_bytes(edited[HEADER + 0x7c20..HEADER + 0x7c24].try_into().unwrap()),
        90.0
    );
    assert_eq!(edited[HEADER + 0x7cf8 + 339], 2);
    assert_eq!(word(&edited, edited.len() - 4), 7);
    assert_checksum(&edited, WINDOWS_PAYLOAD);

    let listed = String::from_utf8(success(&directory, &["list"]).stdout).unwrap();
    assert!(listed.contains("Purchased characters   macewindu_ep3"));
    assert!(listed.contains("Purchased extras       adaptivedifficulty"));
    assert!(listed.contains("Completed tutorials    bank1.GizForce_601"));
    let raw = String::from_utf8(success(&directory, &["list", "--raw"]).stdout).unwrap();
    assert!(raw.contains("shop_character_purchased_bits=macewindu_ep3"));
    assert!(raw.contains("extra_purchased_bits=adaptivedifficulty"));
    assert!(raw.contains("hint_completion_bits=bank1.GizForce_601"));

    let before_invalid = edited.clone();
    let invalid = run(&directory, &["edit", "coins=1"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("unknown save property `coins`"));
    assert_eq!(fs::read(directory.save()).unwrap(), before_invalid);
}

#[test]
fn logical_masks_use_names_across_word_boundaries() {
    let directory = TestDirectory::new();
    success(
        &directory,
        &[
            "create",
            "shop_character_purchased_bits=gonkdroid|wookie|slave1|BIT_127",
            "extra_purchased_bits=scorex2|scorex10",
            "hint_completion_bits=console.AutoJump_1568|touch.AutoJump_1568|BIT_191",
        ],
    );
    let bytes = fs::read(directory.save()).unwrap();
    assert_eq!(
        u128::from_le_bytes(bytes[HEADER + 0x7be0..HEADER + 0x7bf0].try_into().unwrap()),
        1 | (1u128 << 32) | (1u128 << 89) | (1u128 << 127)
    );
    assert_eq!(
        u64::from_le_bytes(bytes[HEADER + 0x7c00..HEADER + 0x7c08].try_into().unwrap()),
        (1u64 << 31) | (1u64 << 43)
    );
    let listed = String::from_utf8(
        success(&directory, &["list", "--raw", "--filter", "shop_character"]).stdout,
    )
    .unwrap();
    assert!(listed.contains("0=gonkdroid"));
    assert!(listed.contains("32=wookie"));
    assert!(listed.contains("89=slave1"));
    assert!(listed.contains("127=BIT_127"));
}

#[test]
fn list_defaults_to_a_compact_interpreted_summary() {
    let directory = TestDirectory::new();
    success(
        &directory,
        &[
            "create",
            "coins=12345",
            "options_save.music_enabled=ON",
            "shop_character_purchased_bits=gonkdroid|wookie",
            "area_save[2].complete=COMPLETE",
            "area_save[8].challenge_trial_time=1200",
            "episode_save[1].flags=SUPERSTORY_COMPLETE",
            "level_save[2].arcade_flags=BATTLE",
            "mission_save.best_times[0]=91",
            "mission_save.completed[0]=COMPLETE",
            "customizer.pieces[0]=1",
            "customizer.secondary_pieces[2]=4",
            "character_save[104]=AVAILABLE|UNLOCKED",
            "character_save[2]=UNLOCKED",
            "character_save[339]=AVAILABLE",
        ],
    );
    let listed = String::from_utf8(success(&directory, &["list"]).stdout).unwrap();
    assert!(listed.contains("Progress\n"));
    assert!(listed.contains("  Studs"));
    assert!(listed.contains("  12,345"));
    assert!(listed.contains("  Music"));
    assert!(listed.contains("  On"));
    assert!(listed.contains("  Purchased characters"));
    assert!(listed.contains("  gonkdroid, wookie"));
    assert!(listed.contains("  PalaceRescue"));
    assert!(listed.contains("  available"));
    assert!(listed.contains("Episode 1  Not completed; stored time 1h 00m 00s"));
    assert!(listed.contains("Episode 2  Completed; stored time 1h 00m 00s"));
    assert!(listed.contains("Stored challenge times\n  E1CharacterBonus  20m 00s"));
    assert!(listed.contains("ep1_failedneg_intro2  arcade: Battle"));
    assert!(listed.contains("Find QuiGonJinn  Complete; best time 1m 31s"));
    assert!(listed.contains("Owned / playable"));
    let owned = listed
        .lines()
        .find(|line| line.contains("Owned / playable"))
        .unwrap();
    assert!(owned.contains("QuiGonJinn"));
    assert!(owned.contains("Unknown character (ID 339)"));
    assert!(listed.contains("Unlocked / not owned"));
    assert!(listed.contains("zamwesell"));
    assert!(listed.contains("Primary custom character\n"));
    assert!(listed.contains("Hat / hair  hat_hair_01 (QuiGonJinn)"));
    assert!(listed.contains("Secondary custom character\n"));
    assert!(listed.contains("Weapon      blaster"));
    assert!(!listed.contains("Available IDs"));
    assert!(!listed.contains("header.field"));
    assert!(!listed.contains("challenge complete"));
    assert!(!listed.contains("\n\n"));
    assert!(listed.lines().count() < 100);
}
