use std::mem::offset_of;
use zerocopy::byteorder::little_endian::{F32, I32, U16, U32};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct SaveHeader {
    pub magic: I32,
    pub version: I32,
    pub size: I32,
    pub field3_0x0c: I32,
    pub field4_0x10: I32,
    pub extra_data_offset: I32,
    pub platform_data: [u8; 16],
    pub application_metadata: [u8; 0x800],
    pub slot_metadata: [u8; 0x800],
    pub reserved_metadata: [u8; 0x800],
    pub timestamp_metadata: [u8; 0x800],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct OptionsSave {
    pub player1_rumble: u8,
    pub player2_rumble: u8,
    pub surround_sound: u8,
    pub sound_volume: u8,
    pub music_volume: u8,
    pub master_volume: u8,
    pub music_enabled: u8,
    pub field7_0x7: u8,
    pub field8_0x8: u8,
    pub field9_0x9: u8,
    pub field10_0xa: u8,
    pub widescreen: u8,
    pub brightness: u8,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct LevelSave {
    pub minikit_names: [[u8; 8]; 10],
    pub minikit_count: u8,
    pub reserved_0x51: [u8; 2],
    pub arcade_flags: u8,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct AreaSave {
    pub complete: u8,
    pub area_complete: u8,
    pub story_true_jedi_complete: u8,
    pub freeplay_true_jedi_complete: u8,
    pub minikit_complete: u8,
    pub minikit_count: u8,
    pub red_brick_collected: u8,
    pub challenge_complete: u8,
    pub challenge_trial_time: F32,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct EpisodeSave {
    pub superstory_time_limit: F32,
    pub superstory_score_target: I32,
    pub superstory_complete: u8,
    pub reserved_0x9: [u8; 3],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct CustomiseSave {
    pub pieces: [U16; 9],
    pub field_0x12: [u8; 2],
    pub name: [u8; 32],
    pub use_saved_name: u8,
    pub padding: [u8; 3],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct MissionSave {
    pub best_times: [F32; 20],
    pub completed: [u8; 20],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct GameSavePrefix {
    pub field_0x0: u8,
    pub difficulty: u8,
    pub field_0x2: [u8; 2],
    pub options: OptionsSave,
    pub levels: [LevelSave; 366],
    pub level_save_padding: [u8; 3],
    pub areas: [AreaSave; 72],
    pub episodes: [EpisodeSave; 6],
    pub shop_hint_purchased_bits: [U32; 3],
    pub shop_character_purchased_bits: [U32; 4],
    pub extra_unlocked_bits: [U32; 2],
    pub shop_gold_brick_purchased_bits: U32,
    pub suit_flags: U32,
    pub extra_purchased_bits: [U32; 2],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct ProgressSummary {
    pub coins: U32,
    pub completion: U16,
    pub gold_bricks: u8,
    pub reward_flags: u8,
    pub hub_build_flags: u8,
    pub indy_unlocked: u8,
    pub reserved: [u8; 2],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct GameSaveSuffix {
    pub gameplay_seconds: F32,
    pub customizer: [CustomiseSave; 2],
    pub mission: MissionSave,
    pub characters: [u8; 0x154],
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct GameSave {
    pub prefix: GameSavePrefix,
    pub hint_completion_bits: [U32; 6],
    pub progress: ProgressSummary,
    pub suffix: GameSaveSuffix,
}

/// Windows stores one tutorial bank; Android adds a second bank before progress.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct WindowsGameSave {
    pub prefix: GameSavePrefix,
    pub hint_completion_bits: [U32; 3],
    pub progress: ProgressSummary,
    pub suffix: GameSaveSuffix,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct SuperOptions {
    pub store_pack_flags: U16,
    pub touch_controls: u8,
    pub dpad_locked: u8,
    pub left_control_x: F32,
    pub left_control_y: F32,
    pub right_control_x: F32,
    pub right_control_y: F32,
    pub music_enabled: u8,
    pub store_bundle_flags: u8,
    pub field9_0x16: [u8; 2],
}

const _: () = {
    assert!(size_of::<SaveHeader>() == 0x2028);
    assert!(size_of::<OptionsSave>() == 0x0d);
    assert!(size_of::<LevelSave>() == 0x54);
    assert!(size_of::<AreaSave>() == 0x0c);
    assert!(size_of::<EpisodeSave>() == 0x0c);
    assert!(size_of::<CustomiseSave>() == 0x38);
    assert!(offset_of!(CustomiseSave, name) == 0x14);
    assert!(offset_of!(CustomiseSave, use_saved_name) == 0x34);
    assert!(size_of::<MissionSave>() == 0x64);
    assert!(size_of::<GameSavePrefix>() == 0x7c08);
    assert!(offset_of!(GameSavePrefix, levels) == 0x11);
    assert!(offset_of!(GameSavePrefix, areas) == 0x782c);
    assert!(offset_of!(GameSavePrefix, episodes) == 0x7b8c);
    assert!(size_of::<ProgressSummary>() == 0x0c);
    assert!(offset_of!(ProgressSummary, completion) == 4);
    assert!(offset_of!(ProgressSummary, reserved) == 10);
    assert!(size_of::<GameSaveSuffix>() == 0x22c);
    assert!(size_of::<GameSave>() == 0x7e58);
    assert!(size_of::<WindowsGameSave>() == 0x7e4c);
    assert!(offset_of!(WindowsGameSave, hint_completion_bits) == 0x7c08);
    assert!(offset_of!(WindowsGameSave, progress) == 0x7c14);
    assert!(offset_of!(WindowsGameSave, suffix) == 0x7c20);
    assert!(offset_of!(GameSave, hint_completion_bits) == 0x7c08);
    assert!(offset_of!(GameSave, progress) == 0x7c20);
    assert!(offset_of!(GameSave, suffix) == 0x7c2c);
    assert!(offset_of!(GameSaveSuffix, customizer) == 4);
    assert!(offset_of!(GameSaveSuffix, mission) == 0x74);
    assert!(offset_of!(GameSaveSuffix, characters) == 0xd8);
    assert!(size_of::<SuperOptions>() == 0x18);
};
