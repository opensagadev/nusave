use crate::format::{HEADER_SIZE, OPTIONS_SIZE, Save, SaveKind};
use crate::layout::{
    AndroidProgressSummary, AreaSave, CustomiseSave, EpisodeSave, GameSave, GameSavePrefix,
    GameSaveSuffix, LevelSave, MissionSave, OptionsSave, SaveHeader, SuperOptions, WindowsGameSave,
};
use crate::names::{
    CUSTOMIZER_CATEGORIES, EXTRA_NAMES, TUTORIAL_HINT_BITS, area_name, character_name,
    customizer_piece, level_name, mission_name, shop_character_names,
};
use anstyle::{AnsiColor, Style};
use std::fmt::Write as _;
use std::mem::{offset_of, size_of};
use thiserror::Error;
use zerocopy::FromBytes;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Unsigned,
    Signed,
    Float,
    Bytes,
    Bitmask,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub kind: Kind,
}

#[derive(Default)]
struct EnumValues {
    values: Vec<(String, u32)>,
    flags: bool,
}

fn property(name: impl Into<String>, offset: usize, size: usize, kind: Kind) -> Property {
    Property {
        name: name.into(),
        offset,
        size,
        kind,
    }
}

pub fn properties(save: &Save) -> Vec<Property> {
    let mut result = vec![
        property(
            "header.field0_0x0",
            offset_of!(SaveHeader, magic),
            4,
            Kind::Signed,
        ),
        property(
            "header.field1_0x4",
            offset_of!(SaveHeader, version),
            4,
            Kind::Signed,
        ),
        property("header.size", offset_of!(SaveHeader, size), 4, Kind::Signed),
        property(
            "header.field3_0xc",
            offset_of!(SaveHeader, field3_0x0c),
            4,
            Kind::Signed,
        ),
        property(
            "header.field4_0x10",
            offset_of!(SaveHeader, field4_0x10),
            4,
            Kind::Signed,
        ),
        property(
            "header.extradata_offset",
            offset_of!(SaveHeader, extra_data_offset),
            4,
            Kind::Signed,
        ),
        property(
            "header.platform_data",
            offset_of!(SaveHeader, platform_data),
            16,
            Kind::Bytes,
        ),
        property(
            "header.application_metadata",
            offset_of!(SaveHeader, application_metadata),
            0x800,
            Kind::Bytes,
        ),
        property(
            "header.slot_metadata",
            offset_of!(SaveHeader, slot_metadata),
            0x800,
            Kind::Bytes,
        ),
        property(
            "header.reserved_metadata",
            offset_of!(SaveHeader, reserved_metadata),
            0x800,
            Kind::Bytes,
        ),
        property(
            "header.timestamp_metadata",
            offset_of!(SaveHeader, timestamp_metadata),
            0x800,
            Kind::Bytes,
        ),
    ];
    if save.payload > HEADER_SIZE {
        result.push(property(
            "extra_prefix",
            HEADER_SIZE,
            save.payload - HEADER_SIZE,
            Kind::Bytes,
        ));
    }
    let base = save.payload;
    if save.kind().is_game() {
        result.extend([
            property(
                "field_0x0",
                base + offset_of!(GameSavePrefix, field_0x0),
                1,
                Kind::Unsigned,
            ),
            property(
                "difficulty",
                base + offset_of!(GameSavePrefix, difficulty),
                1,
                Kind::Unsigned,
            ),
            property(
                "field_0x2",
                base + offset_of!(GameSavePrefix, field_0x2),
                2,
                Kind::Bytes,
            ),
            property(
                "shop_gold_brick_purchased_bits",
                base + offset_of!(GameSavePrefix, shop_gold_brick_purchased_bits),
                4,
                Kind::Bitmask,
            ),
            property(
                "shop_hint_purchased_bits",
                base + offset_of!(GameSavePrefix, shop_hint_purchased_bits),
                12,
                Kind::Bitmask,
            ),
            property(
                "shop_character_purchased_bits",
                base + offset_of!(GameSavePrefix, shop_character_purchased_bits),
                16,
                Kind::Bitmask,
            ),
            property(
                "extra_unlocked_bits",
                base + offset_of!(GameSavePrefix, extra_unlocked_bits),
                8,
                Kind::Bitmask,
            ),
            property(
                "extra_purchased_bits",
                base + offset_of!(GameSavePrefix, extra_purchased_bits),
                8,
                Kind::Bitmask,
            ),
            property(
                "hint_completion_bits",
                base + offset_of!(GameSavePrefix, hint_completion_bits),
                24,
                Kind::Bitmask,
            ),
            property(
                "suit_flags",
                base + offset_of!(GameSavePrefix, suit_flags),
                4,
                Kind::Unsigned,
            ),
            property(
                "level_save_padding",
                base + offset_of!(GameSavePrefix, level_save_padding),
                3,
                Kind::Bytes,
            ),
        ]);
        if save.kind() == SaveKind::AndroidGame {
            let summary = base + size_of::<GameSavePrefix>();
            result.extend([
                property(
                    "coins",
                    summary + offset_of!(AndroidProgressSummary, coins),
                    4,
                    Kind::Unsigned,
                ),
                property(
                    "completion",
                    summary + offset_of!(AndroidProgressSummary, completion),
                    2,
                    Kind::Unsigned,
                ),
                property(
                    "gold_bricks",
                    summary + offset_of!(AndroidProgressSummary, gold_bricks),
                    1,
                    Kind::Unsigned,
                ),
                property(
                    "reward_flags",
                    summary + offset_of!(AndroidProgressSummary, reward_flags),
                    1,
                    Kind::Unsigned,
                ),
                property(
                    "hub_build_flags",
                    summary + offset_of!(AndroidProgressSummary, hub_build_flags),
                    1,
                    Kind::Unsigned,
                ),
                property(
                    "indy_unlocked",
                    summary + offset_of!(AndroidProgressSummary, indy_unlocked),
                    1,
                    Kind::Unsigned,
                ),
                property(
                    "reserved_0x7c2a",
                    summary + offset_of!(AndroidProgressSummary, reserved_0x7c2a),
                    2,
                    Kind::Bytes,
                ),
            ]);
        }
        let suffix = base
            + size_of::<GameSavePrefix>()
            + if save.kind() == SaveKind::AndroidGame {
                size_of::<AndroidProgressSummary>()
            } else {
                0
            };
        result.extend([
            property(
                "gameplay_seconds",
                suffix + offset_of!(GameSaveSuffix, gameplay_seconds),
                4,
                Kind::Float,
            ),
            property(
                if save.kind() == SaveKind::AndroidGame {
                    "field_0x7c9f"
                } else {
                    "field_0x7c93"
                },
                suffix + offset_of!(GameSaveSuffix, field_after_customizer),
                1,
                Kind::Unsigned,
            ),
        ]);
        for (name, offset) in [
            ("player1_rumble", offset_of!(OptionsSave, player1_rumble)),
            ("player2_rumble", offset_of!(OptionsSave, player2_rumble)),
            ("surround_sound", offset_of!(OptionsSave, surround_sound)),
            ("sound_volume", offset_of!(OptionsSave, sound_volume)),
            ("music_volume", offset_of!(OptionsSave, music_volume)),
            ("master_volume", offset_of!(OptionsSave, master_volume)),
            ("music_enabled", offset_of!(OptionsSave, music_enabled)),
            ("field7_0x7", offset_of!(OptionsSave, field7_0x7)),
            ("field8_0x8", offset_of!(OptionsSave, field8_0x8)),
            ("field9_0x9", offset_of!(OptionsSave, field9_0x9)),
            ("field10_0xa", offset_of!(OptionsSave, field10_0xa)),
            ("widescreen", offset_of!(OptionsSave, widescreen)),
            ("brightness", offset_of!(OptionsSave, brightness)),
        ] {
            result.push(property(
                format!("options_save.{name}"),
                base + offset_of!(GameSavePrefix, options) + offset,
                1,
                Kind::Unsigned,
            ));
        }
        for index in 0..366 {
            let prefix = format!("level_save[{index}].");
            let offset = base + offset_of!(GameSavePrefix, levels) + index * size_of::<LevelSave>();
            for name_index in 0..10 {
                result.push(property(
                    format!("{prefix}minikit_names[{name_index}]"),
                    offset + name_index * 8,
                    8,
                    Kind::Bytes,
                ));
            }
            result.push(property(
                format!("{prefix}minikit_count"),
                offset + offset_of!(LevelSave, minikit_count),
                1,
                Kind::Unsigned,
            ));
            result.push(property(
                format!("{prefix}reserved_0x51"),
                offset + offset_of!(LevelSave, reserved_0x51),
                2,
                Kind::Bytes,
            ));
            result.push(property(
                format!("{prefix}arcade_flags"),
                offset + offset_of!(LevelSave, arcade_flags),
                1,
                Kind::Unsigned,
            ));
        }
        for index in 0..20 {
            result.push(property(
                format!("mission_save.best_times[{index}]"),
                suffix
                    + offset_of!(GameSaveSuffix, mission)
                    + offset_of!(MissionSave, best_times)
                    + index * 4,
                4,
                Kind::Float,
            ));
            result.push(property(
                format!("mission_save.completed[{index}]"),
                suffix
                    + offset_of!(GameSaveSuffix, mission)
                    + offset_of!(MissionSave, completed)
                    + index,
                1,
                Kind::Unsigned,
            ));
        }
        for index in 0..0x154 {
            result.push(property(
                format!("character_save[{index}]"),
                suffix + offset_of!(GameSaveSuffix, characters) + index,
                1,
                Kind::Unsigned,
            ));
        }
        for index in 0..72 {
            let prefix = format!("area_save[{index}].");
            let offset = base + offset_of!(GameSavePrefix, areas) + index * size_of::<AreaSave>();
            for (name, member, kind) in [
                ("complete", offset_of!(AreaSave, complete), Kind::Unsigned),
                (
                    "area_complete",
                    offset_of!(AreaSave, area_complete),
                    Kind::Unsigned,
                ),
                (
                    "story_buildup_complete",
                    offset_of!(AreaSave, story_buildup_complete),
                    Kind::Unsigned,
                ),
                (
                    "freeplay_buildup_complete",
                    offset_of!(AreaSave, freeplay_buildup_complete),
                    Kind::Unsigned,
                ),
                (
                    "minikit_complete",
                    offset_of!(AreaSave, minikit_complete),
                    Kind::Unsigned,
                ),
                (
                    "minikit_count",
                    offset_of!(AreaSave, minikit_count),
                    Kind::Unsigned,
                ),
                (
                    "red_brick_collected",
                    offset_of!(AreaSave, red_brick_collected),
                    Kind::Unsigned,
                ),
                (
                    "reserved_0x7",
                    offset_of!(AreaSave, reserved_0x7),
                    Kind::Unsigned,
                ),
                (
                    "challenge_trial_time",
                    offset_of!(AreaSave, challenge_trial_time),
                    Kind::Float,
                ),
            ] {
                result.push(property(
                    format!("{prefix}{name}"),
                    offset + member,
                    if kind == Kind::Float { 4 } else { 1 },
                    kind,
                ));
            }
        }
        for index in 0..6 {
            let prefix = format!("episode_save[{index}].");
            let offset =
                base + offset_of!(GameSavePrefix, episodes) + index * size_of::<EpisodeSave>();
            result.push(property(
                format!("{prefix}superstory_time_limit"),
                offset + offset_of!(EpisodeSave, superstory_time_limit),
                4,
                Kind::Float,
            ));
            result.push(property(
                format!("{prefix}superstory_score_target"),
                offset + offset_of!(EpisodeSave, superstory_score_target),
                4,
                Kind::Signed,
            ));
            result.push(property(
                format!("{prefix}flags"),
                offset + offset_of!(EpisodeSave, flags),
                4,
                Kind::Unsigned,
            ));
        }
        let custom = suffix + offset_of!(GameSaveSuffix, customizer);
        for index in 0..9 {
            result.push(property(
                format!("customizer.pieces[{index}]"),
                custom + offset_of!(CustomiseSave, pieces) + index * 2,
                2,
                Kind::Signed,
            ));
        }
        for index in 0..9 {
            result.push(property(
                format!("customizer.secondary_pieces[{index}]"),
                custom + offset_of!(CustomiseSave, secondary_pieces) + index * 2,
                2,
                Kind::Signed,
            ));
        }
        result.extend([
            property(
                "customizer.field_0x12",
                custom + offset_of!(CustomiseSave, field_0x12),
                2,
                Kind::Bytes,
            ),
            property(
                "customizer.primary_name",
                custom + offset_of!(CustomiseSave, primary_name),
                32,
                Kind::Bytes,
            ),
            property(
                "customizer.primary_use_saved_name",
                custom + offset_of!(CustomiseSave, primary_use_saved_name),
                1,
                Kind::Unsigned,
            ),
            property(
                "customizer.field_0x35",
                custom + offset_of!(CustomiseSave, field_0x35),
                3,
                Kind::Bytes,
            ),
            property(
                "customizer.secondary_name",
                custom + offset_of!(CustomiseSave, secondary_name),
                32,
                Kind::Bytes,
            ),
            property(
                "customizer.secondary_use_saved_name",
                custom + offset_of!(CustomiseSave, secondary_use_saved_name),
                1,
                Kind::Unsigned,
            ),
            property(
                "customizer.field_0x4a",
                custom + offset_of!(CustomiseSave, field_0x4a),
                2,
                Kind::Bytes,
            ),
            property(
                "customizer.field_0x6d",
                custom + offset_of!(CustomiseSave, field_0x6d),
                2,
                Kind::Bytes,
            ),
        ]);
    } else {
        debug_assert_eq!(save.payload_size, OPTIONS_SIZE);
        result.extend([
            property(
                "options.store_pack_flags",
                base + offset_of!(SuperOptions, store_pack_flags),
                2,
                Kind::Unsigned,
            ),
            property(
                "options.touch_controls",
                base + offset_of!(SuperOptions, touch_controls),
                1,
                Kind::Unsigned,
            ),
            property(
                "options.dpad_locked",
                base + offset_of!(SuperOptions, dpad_locked),
                1,
                Kind::Unsigned,
            ),
            property(
                "options.left_control_x",
                base + offset_of!(SuperOptions, left_control_x),
                4,
                Kind::Float,
            ),
            property(
                "options.left_control_y",
                base + offset_of!(SuperOptions, left_control_y),
                4,
                Kind::Float,
            ),
            property(
                "options.right_control_x",
                base + offset_of!(SuperOptions, right_control_x),
                4,
                Kind::Float,
            ),
            property(
                "options.right_control_y",
                base + offset_of!(SuperOptions, right_control_y),
                4,
                Kind::Float,
            ),
            property(
                "options.music_enabled",
                base + offset_of!(SuperOptions, music_enabled),
                1,
                Kind::Unsigned,
            ),
            property(
                "options.store_bundle_flags",
                base + offset_of!(SuperOptions, store_bundle_flags),
                1,
                Kind::Unsigned,
            ),
            property(
                "options.field9_0x16",
                base + offset_of!(SuperOptions, field9_0x16),
                2,
                Kind::Bytes,
            ),
        ]);
    }
    result.push(property(
        "checksum",
        base + save.payload_size,
        4,
        Kind::Unsigned,
    ));
    result.push(property(
        "slot_code",
        save.bytes.len() - 4,
        4,
        Kind::Unsigned,
    ));

    let mut covered = vec![false; save.bytes.len()];
    for field in &result {
        for byte in &mut covered[field.offset..field.offset + field.size] {
            *byte = true;
        }
    }
    for (offset, is_covered) in covered.into_iter().enumerate() {
        if !is_covered {
            result.push(property(
                format!("byte[{offset}]"),
                offset,
                1,
                Kind::Unsigned,
            ));
        }
    }
    result
}

fn bit_names(kind: SaveKind, field: &Property) -> Vec<String> {
    let mut names: Vec<String> = (0..field.size * 8)
        .map(|bit| format!("BIT_{bit}"))
        .collect();
    match field.name.as_str() {
        "shop_character_purchased_bits" => {
            for (bit, name) in shop_character_names(kind).enumerate() {
                if bit < names.len() {
                    names[bit] = name.into();
                }
            }
        }
        "extra_unlocked_bits" | "extra_purchased_bits" => {
            for (bit, name) in EXTRA_NAMES.iter().copied().enumerate() {
                if bit < names.len() {
                    names[bit] = name.into();
                }
            }
            if kind == SaveKind::WindowsGame && names.len() > 44 {
                names[44] = "adaptivedifficulty".into();
            }
        }
        "hint_completion_bits" => {
            let bank_bits = names.len() / 2;
            for bank in 0..2 {
                for &(name, bit) in TUTORIAL_HINT_BITS {
                    if bit < bank_bits {
                        let bank_name = match (kind, bank) {
                            (SaveKind::AndroidGame, 0) => "console",
                            (SaveKind::AndroidGame, _) => "touch",
                            (_, 0) => "bank0",
                            (_, _) => "bank1",
                        };
                        names[bank * bank_bits + bit] = format!("{bank_name}.{name}");
                    }
                }
            }
        }
        "shop_gold_brick_purchased_bits" => {
            for (bit, name) in names.iter_mut().enumerate().take(14) {
                *name = format!("GOLD_BRICK_{bit}");
            }
        }
        _ => {}
    }
    names
}

fn pairs(values: &[(&str, u32)], flags: bool) -> EnumValues {
    EnumValues {
        values: values
            .iter()
            .map(|&(name, value)| (name.into(), value))
            .collect(),
        flags,
    }
}

fn enum_values(name: &str) -> EnumValues {
    if name.starts_with("extra_unlocked_bits[") || name.starts_with("extra_purchased_bits[") {
        let word = name
            .split_once('[')
            .and_then(|(_, rest)| rest.split_once(']'))
            .and_then(|(value, _)| value.parse::<usize>().ok())
            .unwrap_or(0);
        let mut values = vec![("NONE".into(), 0)];
        for (index, extra) in EXTRA_NAMES.iter().enumerate().skip(word * 32).take(32) {
            values.push(((*extra).into(), 1u32 << (index % 32)));
        }
        return EnumValues {
            values,
            flags: true,
        };
    }
    if name.starts_with("shop_hint_purchased_bits[")
        || name.starts_with("shop_character_purchased_bits[")
        || name.starts_with("hint_completion_bits[")
        || name == "shop_gold_brick_purchased_bits"
    {
        let mut values = vec![("NONE".into(), 0)];
        values.extend((0..32).map(|bit| (format!("BIT_{bit}"), 1u32 << bit)));
        return EnumValues {
            values,
            flags: true,
        };
    }
    if name == "hub_build_flags" {
        return pairs(
            &[
                ("NONE", 0),
                ("BUILD_0", 1),
                ("BUILD_1", 2),
                ("BUILD_2", 4),
                ("BUILD_3", 8),
                ("BUILD_4", 16),
                ("BUILD_5", 32),
                ("BUILD_6", 64),
                ("LEVEL_BUILD", 128),
            ],
            true,
        );
    }
    if name == "reward_flags" {
        return pairs(
            &[("NONE", 0), ("100_PERCENT", 1), ("ALL_GOLD_BRICKS", 2)],
            true,
        );
    }
    if name.starts_with("character_save[") {
        return pairs(&[("NONE", 0), ("AVAILABLE", 1), ("UNLOCKED", 2)], true);
    }
    if name.contains(".arcade_flags") {
        return pairs(
            &[("NONE", 0), ("BATTLE", 1), ("COLLECT", 2), ("HUNT", 4)],
            true,
        );
    }
    if name.starts_with("episode_save[") && name.contains(".flags") {
        return pairs(&[("NONE", 0), ("SUPERSTORY_COMPLETE", 1)], true);
    }
    if name == "suit_flags" {
        return pairs(
            &[
                ("NONE", 0),
                ("SHADOW", 1),
                ("GLIDE", 2),
                ("DEMOLITION", 4),
                ("SONAR", 8),
                ("WATER", 16),
                ("TECHNOLOGY", 32),
                ("MAGNET", 64),
                ("ATTRACT", 128),
                ("ALL", u32::MAX),
            ],
            true,
        );
    }
    if name == "options.store_pack_flags" {
        return pairs(
            &[
                ("NONE", 0),
                ("EPISODE_II", 1),
                ("EPISODE_III", 2),
                ("EPISODE_IV", 4),
                ("EPISODE_V", 8),
                ("EPISODE_VI", 16),
                ("ARCADE", 32),
                ("BONUS", 64),
                ("BOUNTY", 128),
                ("CHALLENGE", 256),
                ("JEDI", 512),
                ("SITH", 1024),
            ],
            true,
        );
    }
    if name == "options.store_bundle_flags" {
        return pairs(
            &[
                ("NONE", 0),
                ("PREQUEL", 1),
                ("ORIGINAL", 2),
                ("COMPLETE", 4),
            ],
            true,
        );
    }
    if name == "options.touch_controls" {
        return pairs(&[("VIRTUAL_CONSOLE", 0), ("TOUCH", 1)], false);
    }
    if name.contains("_use_saved_name") {
        return pairs(&[("CHARACTER_DEFAULT", 0), ("SAVED_NAME", 1)], false);
    }
    if name == "indy_unlocked"
        || name.contains(".completed[")
        || (name.starts_with("area_save[")
            && (name.contains("complete") || name.contains("red_brick_collected")))
    {
        return pairs(&[("INCOMPLETE", 0), ("COMPLETE", 1)], false);
    }
    if matches!(
        name,
        "options.music_enabled"
            | "options.dpad_locked"
            | "options_save.player1_rumble"
            | "options_save.player2_rumble"
            | "options_save.surround_sound"
            | "options_save.music_enabled"
            | "options_save.widescreen"
    ) {
        return pairs(&[("OFF", 0), ("ON", 1)], false);
    }
    EnumValues::default()
}

fn enum_text(enumeration: &EnumValues, mut value: u32) -> String {
    if let Some((name, _)) = enumeration
        .values
        .iter()
        .find(|(_, candidate)| *candidate == value)
    {
        return name.clone();
    }
    let mut result = Vec::new();
    if enumeration.flags {
        for (name, candidate) in &enumeration.values {
            if *candidate != 0 && value & candidate == *candidate {
                result.push(name.clone());
                value &= !candidate;
            }
        }
    }
    if result.is_empty() || value != 0 {
        result.push(value.to_string());
    }
    result.join("|")
}

fn parse_number(text: &str) -> Option<u64> {
    if text.is_empty() || text.starts_with(['-', '+', ' ']) || text.ends_with(char::is_whitespace) {
        return None;
    }
    if let Some(hex) = text.strip_prefix("0x") {
        (!hex.is_empty())
            .then(|| u64::from_str_radix(hex, 16).ok())
            .flatten()
    } else {
        text.parse().ok()
    }
}

fn parse_mask_number(text: &str, size: usize) -> Option<Vec<u8>> {
    let (digits, radix) = text
        .strip_prefix("0x")
        .map_or((text, 10u16), |hex| (hex, 16));
    if digits.is_empty() {
        return None;
    }
    let mut bytes = vec![0u8; size];
    for character in digits.bytes() {
        let digit = match character {
            b'0'..=b'9' => (character - b'0') as u16,
            b'a'..=b'f' => (character - b'a' + 10) as u16,
            b'A'..=b'F' => (character - b'A' + 10) as u16,
            _ => return None,
        };
        if digit >= radix {
            return None;
        }
        let mut carry = digit;
        for byte in &mut bytes {
            carry += *byte as u16 * radix;
            *byte = carry as u8;
            carry >>= 8;
        }
        if carry != 0 {
            return None;
        }
    }
    Some(bytes)
}

fn assign_mask(destination: &mut [u8], kind: SaveKind, field: &Property, value: &str) -> bool {
    let names = bit_names(kind, field);
    let mut result = vec![0u8; field.size];
    for token in value.split('|') {
        if token == "NONE" {
            continue;
        }
        if let Some(bit) = names
            .iter()
            .enumerate()
            .find_map(|(bit, name)| (token == name || token == format!("BIT_{bit}")).then_some(bit))
        {
            result[bit / 8] |= 1 << (bit % 8);
            continue;
        }
        let Some(part) = parse_mask_number(token, field.size) else {
            return false;
        };
        for (output, part) in result.iter_mut().zip(part) {
            *output |= part;
        }
    }
    destination.copy_from_slice(&result);
    true
}

/// A `KEY=VALUE` edit could not be understood or applied.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AssignmentError {
    #[error("assignment `{assignment}` must use KEY=VALUE syntax")]
    MissingEquals { assignment: String },
    #[error("unknown save property `{property}`")]
    UnknownProperty { property: String },
    #[error("invalid value `{value}` for save property `{property}`; expected {expected}")]
    InvalidValue {
        property: String,
        value: String,
        expected: String,
    },
}

pub(crate) fn assign(
    save: &mut Save,
    fields: &[Property],
    assignment: &str,
) -> Result<(), AssignmentError> {
    let Some((raw_name, raw_value)) = assignment.split_once('=') else {
        return Err(AssignmentError::MissingEquals {
            assignment: assignment.to_owned(),
        });
    };
    let mut value = raw_value.to_owned();
    let name = match raw_name {
        "save_version" => "difficulty",
        "field30_0x7c2c" | "field30_0x7c20" => "gameplay_seconds",
        "initial_store_pack_flags" => "suit_flags",
        "field_0x7bf8" => "shop_gold_brick_purchased_bits",
        "customizer.primary_name_unlocked" => "customizer.primary_use_saved_name",
        "customizer.secondary_name_unlocked" => "customizer.secondary_use_saved_name",
        other => other,
    };
    let mut field = fields.iter().find(|field| field.name == name).cloned();
    if field.is_none()
        && name.ends_with(']')
        && let Some((base, index_text)) = name.rsplit_once('[')
        && let Some(index) = index_text
            .strip_suffix(']')
            .and_then(parse_number)
            .and_then(|value| usize::try_from(value).ok())
    {
        if base == "byte" && index < save.bytes.len() {
            field = Some(property(name, index, 1, Kind::Unsigned));
        } else if let Some(parent) = fields.iter().find(|candidate| candidate.name == base) {
            if parent.kind == Kind::Bytes && index < parent.size {
                field = Some(property(name, parent.offset + index, 1, Kind::Unsigned));
            } else if parent.kind == Kind::Bitmask && index < parent.size / 4 {
                field = Some(property(name, parent.offset + index * 4, 4, Kind::Unsigned));
            }
        }
    }
    let Some(field) = field else {
        return Err(AssignmentError::UnknownProperty {
            property: raw_name.to_owned(),
        });
    };
    let invalid_value = || AssignmentError::InvalidValue {
        property: raw_name.to_owned(),
        value: raw_value.to_owned(),
        expected: match field.kind {
            Kind::Bitmask => {
                "named flags joined with |, a decimal mask, or a 0x hexadecimal mask".to_owned()
            }
            Kind::Bytes => format!(
                "hex: followed by {} hex digits, or text: followed by at most {} bytes",
                field.size * 2,
                field.size - 1
            ),
            Kind::Float => "a finite 32-bit floating-point number".to_owned(),
            Kind::Signed | Kind::Unsigned => integer_expectation(&field),
        },
    };
    let save_kind = save.kind();
    let destination = &mut save.bytes[field.offset..field.offset + field.size];
    match field.kind {
        Kind::Bitmask => {
            return assign_mask(destination, save_kind, &field, &value)
                .then_some(())
                .ok_or_else(invalid_value);
        }
        Kind::Bytes => {
            if let Some(text) = value.strip_prefix("text:") {
                if text.len() >= field.size {
                    return Err(invalid_value());
                }
                destination.fill(0);
                destination[..text.len()].copy_from_slice(text.as_bytes());
                return Ok(());
            }
            let Some(hex) = value.strip_prefix("hex:") else {
                return Err(invalid_value());
            };
            if hex.len() != field.size * 2 {
                return Err(invalid_value());
            }
            for (index, byte) in destination.iter_mut().enumerate() {
                let pair = &hex.as_bytes()[index * 2..index * 2 + 2];
                let Ok(pair) = std::str::from_utf8(pair) else {
                    return Err(invalid_value());
                };
                let Ok(parsed) = u8::from_str_radix(pair, 16) else {
                    return Err(invalid_value());
                };
                *byte = parsed;
            }
            return Ok(());
        }
        Kind::Float => {
            let Ok(parsed) = value.parse::<f32>() else {
                return Err(invalid_value());
            };
            if !parsed.is_finite() {
                return Err(invalid_value());
            }
            destination.copy_from_slice(&parsed.to_le_bytes());
            return Ok(());
        }
        Kind::Unsigned | Kind::Signed => {}
    }

    let enumeration = enum_values(name);
    if !enumeration.values.is_empty() {
        let mut combined = 0u64;
        let mut symbolic = false;
        let tokens: Vec<_> = value.split('|').collect();
        if tokens.len() > 1 && !enumeration.flags {
            return Err(invalid_value());
        }
        for token in tokens {
            if let Some((_, part)) = enumeration
                .values
                .iter()
                .find(|(candidate, _)| candidate == token)
            {
                combined |= *part as u64;
                symbolic = true;
            } else if let Some(part) = parse_number(token) {
                combined |= part;
            } else {
                return Err(invalid_value());
            }
        }
        if symbolic || value.contains('|') {
            value = combined.to_string();
        }
    }
    let negative = value.starts_with('-');
    let Some(mut parsed) = parse_number(value.strip_prefix('-').unwrap_or(&value)) else {
        return Err(invalid_value());
    };
    let bits = field.size * 8;
    let limit = if field.kind == Kind::Signed {
        1u64 << (bits - 1)
    } else {
        1u64 << bits
    };
    if negative {
        if field.kind != Kind::Signed || parsed > limit {
            return Err(invalid_value());
        }
        parsed = 0u64.wrapping_sub(parsed);
    } else if parsed >= limit {
        return Err(invalid_value());
    }
    for (index, byte) in destination.iter_mut().enumerate() {
        *byte = (parsed >> (index * 8)) as u8;
    }
    Ok(())
}

fn mask_hex(bytes: &[u8]) -> String {
    let mut result = String::from("0x");
    for byte in bytes.iter().rev() {
        write!(result, "{byte:02x}").expect("write to string");
    }
    result
}

fn read_unsigned(bytes: &[u8]) -> u32 {
    let mut value = [0u8; 4];
    value[..bytes.len()].copy_from_slice(bytes);
    u32::from_le_bytes(value)
}

#[derive(Debug)]
struct SummaryRow {
    section: &'static str,
    label: String,
    value: String,
}

fn summary_row(
    rows: &mut Vec<SummaryRow>,
    section: &'static str,
    label: impl Into<String>,
    value: impl Into<String>,
) {
    rows.push(SummaryRow {
        section,
        label: label.into(),
        value: value.into(),
    });
}

fn human_words(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            if matches!(part, "II" | "III" | "IV" | "V" | "VI") {
                return part.to_owned();
            }
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + &characters.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn human_enum(name: &str, value: u32) -> String {
    let enumeration = enum_values(name);
    if enumeration.values.is_empty() {
        return value.to_string();
    }
    if let Some((label, _)) = enumeration
        .values
        .iter()
        .find(|(_, candidate)| *candidate == value)
    {
        return human_words(label);
    }
    if !enumeration.flags {
        return format!("Unknown (value {value})");
    }
    let mut remaining = value;
    let mut labels = Vec::new();
    for (label, flag) in &enumeration.values {
        if *flag != 0 && remaining & flag == *flag {
            labels.push(human_words(label));
            remaining &= !flag;
        }
    }
    if remaining != 0 {
        labels.push(format!("unknown bits 0x{remaining:x}"));
    }
    if labels.is_empty() {
        "None".into()
    } else {
        labels.join(", ")
    }
}

fn fixed_text(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if end == 0 {
        return "(empty)".into();
    }
    if bytes[..end]
        .iter()
        .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
    {
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    } else {
        "(non-text data)".into()
    }
}

fn duration(seconds: f32) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return format!("Invalid ({seconds:?})");
    }
    if seconds >= 60.0 && seconds.fract() == 0.0 {
        let total = seconds as u64;
        let hours = total / 3600;
        let minutes = total % 3600 / 60;
        let seconds = total % 60;
        if hours > 0 {
            format!("{hours}h {minutes:02}m {seconds:02}s")
        } else {
            format!("{minutes}m {seconds:02}s")
        }
    } else {
        format!("{seconds:.2}s")
    }
}

fn number(value: impl ToString) -> String {
    let value = value.to_string();
    let (sign, digits) = value
        .strip_prefix('-')
        .map_or(("", value.as_str()), |digits| ("-", digits));
    let mut grouped = String::new();
    for (index, character) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    format!("{sign}{}", grouped.chars().rev().collect::<String>())
}

fn mask_summary(save: &Save, fields: &[Property], name: &str) -> String {
    let Some(field) = fields.iter().find(|field| field.name == name) else {
        return "None".into();
    };
    let data = &save.bytes[field.offset..field.offset + field.size];
    let labels = bit_names(save.kind(), field)
        .into_iter()
        .enumerate()
        .filter(|(bit, _)| data[*bit / 8] & (1 << (*bit % 8)) != 0)
        .map(|(bit, name)| {
            if name == format!("BIT_{bit}") {
                format!("unknown bit {bit}")
            } else {
                name
            }
        })
        .collect::<Vec<_>>();
    if labels.is_empty() {
        "None".into()
    } else {
        labels.join(", ")
    }
}

fn character_summary(kind: SaveKind, characters: &[u8], include: impl Fn(u8) -> bool) -> String {
    let names = characters
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, flags)| include(*flags))
        .map(|(id, _)| {
            character_name(kind, id)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Unknown character (ID {id})"))
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        return "None".into();
    }
    names.join(", ")
}

fn paint(text: &str, style: Style, color: bool) -> String {
    if color {
        format!("{style}{text}{style:#}")
    } else {
        text.to_owned()
    }
}

fn value_style(value: &str) -> Style {
    if matches!(
        value,
        "Valid" | "On" | "Unlocked" | "Complete" | "Completed"
    ) {
        Style::new().fg_color(Some(AnsiColor::Green.into()))
    } else if value == "Invalid" {
        Style::new().fg_color(Some(AnsiColor::Red.into())).bold()
    } else if matches!(value, "None" | "Off" | "Locked" | "(empty)") {
        Style::new().fg_color(Some(AnsiColor::BrightBlack.into()))
    } else {
        Style::new()
    }
}

fn render_summary(rows: Vec<SummaryRow>, filter: &str, color: bool) -> String {
    let filter = filter.to_ascii_lowercase();
    let rows = rows
        .into_iter()
        .filter(|row| {
            filter.is_empty()
                || row.section.to_ascii_lowercase().contains(&filter)
                || row.label.to_ascii_lowercase().contains(&filter)
                || row.value.to_ascii_lowercase().contains(&filter)
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return "No matching interpreted values.\n".into();
    }

    let mut output = String::new();
    let mut start = 0;
    let section_style = Style::new().fg_color(Some(AnsiColor::Cyan.into())).bold();
    let label_style = Style::new()
        .fg_color(Some(AnsiColor::BrightBlue.into()))
        .bold();
    while start < rows.len() {
        let section = rows[start].section;
        let end = rows[start..]
            .iter()
            .position(|row| row.section != section)
            .map_or(rows.len(), |relative| start + relative);
        let label_width = rows[start..end]
            .iter()
            .map(|row| row.label.chars().count())
            .max()
            .unwrap_or(0);
        writeln!(output, "{}", paint(section, section_style, color)).expect("write to string");
        for row in &rows[start..end] {
            let label = format!("{:<label_width$}", row.label);
            let style = value_style(&row.value);
            writeln!(
                output,
                "  {}  {}",
                paint(&label, label_style, color),
                paint(&row.value, style, color)
            )
            .expect("write to string");
        }
        start = end;
    }
    output
}

fn configured_name(name: Option<&str>, kind: &str, index: usize) -> String {
    name.map(str::to_owned)
        .unwrap_or_else(|| format!("Unknown {kind} (ID {index})"))
}

fn customizer_piece_summary(category: usize, index: i16) -> String {
    customizer_piece(category, index).map_or_else(
        || format!("Unknown piece (ID {index})"),
        |piece| match piece.source_character {
            Some(source) => format!("{} ({source})", piece.name),
            None => piece.name.to_owned(),
        },
    )
}

struct GameView<'a> {
    prefix: &'a GameSavePrefix,
    android_summary: Option<&'a AndroidProgressSummary>,
    suffix: &'a GameSaveSuffix,
}

fn game_view(save: &Save) -> GameView<'_> {
    let payload = &save.bytes[save.payload..save.payload + save.payload_size];
    match save.kind() {
        SaveKind::AndroidGame => {
            let game = GameSave::ref_from_bytes(payload).expect("validated Android game payload");
            GameView {
                prefix: &game.prefix,
                android_summary: Some(&game.android_summary),
                suffix: &game.suffix,
            }
        }
        SaveKind::WindowsGame => {
            let game =
                WindowsGameSave::ref_from_bytes(payload).expect("validated Windows game payload");
            GameView {
                prefix: &game.prefix,
                android_summary: None,
                suffix: &game.suffix,
            }
        }
        SaveKind::AndroidOptions => unreachable!("SuperOptions is not a game payload"),
    }
}

pub fn summary_text(save: &Save, fields: &[Property], filter: &str, color: bool) -> String {
    let mut rows = Vec::new();
    summary_row(&mut rows, "Save", "Kind", save.kind().description());
    summary_row(
        &mut rows,
        "Save",
        "Checksum",
        if save.checksum_valid() {
            "Valid"
        } else {
            "Invalid"
        },
    );
    summary_row(
        &mut rows,
        "Save",
        "File size",
        format!("{} bytes", number(save.bytes.len())),
    );
    if save.payload > HEADER_SIZE {
        summary_row(
            &mut rows,
            "Save",
            "Opaque prefix",
            format!("{} bytes preserved", save.payload - HEADER_SIZE),
        );
    }

    let metadata = save.header_metadata();
    for (label, value) in [
        ("Application", metadata.application),
        ("Slot label", metadata.slot),
        ("Reserved label", metadata.reserved),
        ("Saved at", metadata.timestamp),
    ] {
        if let Some(value) = value {
            summary_row(&mut rows, "Envelope metadata", label, value);
        }
    }

    if save.kind() == SaveKind::AndroidOptions {
        let options = SuperOptions::ref_from_bytes(
            &save.bytes[save.payload..save.payload + save.payload_size],
        )
        .expect("validated options payload");
        summary_row(
            &mut rows,
            "Controls",
            "Mode",
            human_enum("options.touch_controls", options.touch_controls.into()),
        );
        summary_row(
            &mut rows,
            "Controls",
            "D-pad lock",
            human_enum("options.dpad_locked", options.dpad_locked.into()),
        );
        summary_row(
            &mut rows,
            "Controls",
            "Left position",
            format!(
                "({}, {})",
                options.left_control_x.get(),
                options.left_control_y.get()
            ),
        );
        summary_row(
            &mut rows,
            "Controls",
            "Right position",
            format!(
                "({}, {})",
                options.right_control_x.get(),
                options.right_control_y.get()
            ),
        );
        summary_row(
            &mut rows,
            "Audio",
            "Music",
            human_enum("options.music_enabled", options.music_enabled.into()),
        );
        summary_row(
            &mut rows,
            "Store entitlements",
            "Packs",
            human_enum(
                "options.store_pack_flags",
                options.store_pack_flags.get().into(),
            ),
        );
        summary_row(
            &mut rows,
            "Store entitlements",
            "Bundles",
            human_enum(
                "options.store_bundle_flags",
                options.store_bundle_flags.into(),
            ),
        );
        return render_summary(rows, filter, color);
    }

    let game = game_view(save);
    if let Some(summary) = game.android_summary {
        summary_row(&mut rows, "Progress", "Studs", number(summary.coins.get()));
        summary_row(
            &mut rows,
            "Progress",
            "Completion points",
            number(summary.completion.get()),
        );
        summary_row(
            &mut rows,
            "Progress",
            "Gold bricks",
            number(summary.gold_bricks),
        );
        summary_row(
            &mut rows,
            "Progress",
            "Indiana Jones",
            if summary.indy_unlocked == 0 {
                "Locked"
            } else {
                "Unlocked"
            },
        );
        summary_row(
            &mut rows,
            "Progress",
            "Completion rewards",
            human_enum("reward_flags", summary.reward_flags.into()),
        );
        summary_row(
            &mut rows,
            "Progress",
            "Hub builds",
            human_enum("hub_build_flags", summary.hub_build_flags.into()),
        );
    } else {
        summary_row(
            &mut rows,
            "Progress",
            "Stored slot code",
            number(save.slot_code() as i32),
        );
    }
    summary_row(
        &mut rows,
        "Progress",
        "Gameplay time",
        duration(game.suffix.gameplay_seconds.get()),
    );
    summary_row(
        &mut rows,
        "Progress",
        "Difficulty",
        game.prefix.difficulty.to_string(),
    );
    summary_row(
        &mut rows,
        "Progress",
        "Suit abilities",
        human_enum("suit_flags", game.prefix.suit_flags.get()),
    );

    let options = &game.prefix.options;
    for (label, name, value) in [
        (
            "Player 1 rumble",
            "options_save.player1_rumble",
            options.player1_rumble,
        ),
        (
            "Player 2 rumble",
            "options_save.player2_rumble",
            options.player2_rumble,
        ),
        (
            "Surround sound",
            "options_save.surround_sound",
            options.surround_sound,
        ),
        ("Music", "options_save.music_enabled", options.music_enabled),
        ("Widescreen", "options_save.widescreen", options.widescreen),
    ] {
        summary_row(
            &mut rows,
            "Game settings",
            label,
            human_enum(name, value.into()),
        );
    }
    summary_row(
        &mut rows,
        "Game settings",
        "Sound volume",
        format!("{}/10", options.sound_volume),
    );
    summary_row(
        &mut rows,
        "Game settings",
        "Music volume",
        format!("{}/10", options.music_volume),
    );
    summary_row(
        &mut rows,
        "Game settings",
        "Master volume",
        format!("{}/10", options.master_volume),
    );
    summary_row(
        &mut rows,
        "Game settings",
        "Brightness",
        format!("{}/10", options.brightness),
    );

    for (label, name) in [
        ("Purchased characters", "shop_character_purchased_bits"),
        ("Purchased hints", "shop_hint_purchased_bits"),
        ("Unlocked extras", "extra_unlocked_bits"),
        ("Purchased extras", "extra_purchased_bits"),
        ("Purchased gold bricks", "shop_gold_brick_purchased_bits"),
        ("Completed tutorials", "hint_completion_bits"),
    ] {
        summary_row(
            &mut rows,
            "Collections",
            label,
            mask_summary(save, fields, name),
        );
    }

    for (index, episode) in game.prefix.episodes.iter().enumerate() {
        let mut values = vec![
            if episode.flags.get() & 1 != 0 {
                "Completed".to_owned()
            } else {
                "Not completed".to_owned()
            },
            format!(
                "stored time {}",
                duration(episode.superstory_time_limit.get())
            ),
            format!(
                "stored score {}",
                number(episode.superstory_score_target.get())
            ),
        ];
        if episode.flags.get() & !1 != 0 {
            values.push(format!("unknown flags 0x{:x}", episode.flags.get() & !1));
        }
        summary_row(
            &mut rows,
            "Episodes",
            format!("Episode {}", index + 1),
            values.join("; "),
        );
    }

    for (index, area) in game.prefix.areas.iter().enumerate() {
        let mut values = Vec::new();
        if area.complete != 0 {
            values.push("available".to_owned());
        }
        if area.area_complete != 0 {
            values.push("complete".to_owned());
        }
        if area.story_buildup_complete != 0 {
            values.push("story buildup complete".to_owned());
        }
        if area.freeplay_buildup_complete != 0 {
            values.push("free-play buildup complete".to_owned());
        }
        if area.minikit_complete != 0 {
            values.push("minikit complete".to_owned());
        } else if area.minikit_count != 0 {
            values.push(format!("{} minikits", area.minikit_count));
        }
        if area.red_brick_collected != 0 {
            values.push("red brick collected".to_owned());
        }
        if !values.is_empty() {
            summary_row(
                &mut rows,
                "Areas with progress",
                configured_name(area_name(save.kind(), index), "area", index),
                values.join(", "),
            );
        }
    }

    for (index, area) in game.prefix.areas.iter().enumerate() {
        if area.challenge_trial_time.get() != 0.0 {
            summary_row(
                &mut rows,
                "Stored challenge times",
                configured_name(area_name(save.kind(), index), "area", index),
                duration(area.challenge_trial_time.get()),
            );
        }
    }

    for (index, level) in game.prefix.levels.iter().enumerate() {
        let names = level
            .minikit_names
            .iter()
            .take(level.minikit_count.min(10).into())
            .map(|name| fixed_text(name))
            .filter(|name| name != "(empty)")
            .collect::<Vec<_>>();
        let arcade = human_enum(
            &format!("level_save[{index}].arcade_flags"),
            level.arcade_flags.into(),
        );
        if !names.is_empty() || arcade != "None" {
            let mut values = Vec::new();
            if !names.is_empty() {
                values.push(format!("minikits: {}", names.join(", ")));
            }
            if arcade != "None" {
                values.push(format!("arcade: {arcade}"));
            }
            summary_row(
                &mut rows,
                "Levels with progress",
                configured_name(level_name(save.kind(), index), "level", index),
                values.join("; "),
            );
        }
    }

    for (index, mission) in game.suffix.mission.completed.iter().enumerate() {
        let time = game.suffix.mission.best_times[index].get();
        if *mission != 0 || time != 0.0 {
            let state = if *mission == 0 {
                "Incomplete"
            } else {
                "Complete"
            };
            let value = if time == 0.0 {
                state.to_owned()
            } else {
                format!("{state}; best time {}", duration(time))
            };
            summary_row(
                &mut rows,
                "Missions with progress",
                mission_name(index)
                    .map(|name| format!("Find {name}"))
                    .unwrap_or_else(|| format!("Unknown mission (ID {index})")),
                value,
            );
        }
    }

    summary_row(
        &mut rows,
        "Characters",
        "Owned / playable",
        character_summary(save.kind(), &game.suffix.characters, |flags| flags & 1 != 0),
    );
    summary_row(
        &mut rows,
        "Characters",
        "Unlocked / not owned",
        character_summary(save.kind(), &game.suffix.characters, |flags| {
            flags & 2 != 0 && flags & 1 == 0
        }),
    );
    let unusual = character_summary(save.kind(), &game.suffix.characters, |flags| {
        flags & !3 != 0
    });
    if unusual != "None" {
        summary_row(&mut rows, "Characters", "Unknown state bits", unusual);
    }

    let custom = &game.suffix.customizer;
    summary_row(
        &mut rows,
        "Primary custom character",
        "Name",
        fixed_text(&custom.primary_name),
    );
    summary_row(
        &mut rows,
        "Primary custom character",
        "Naming",
        human_enum(
            "customizer.primary_use_saved_name",
            custom.primary_use_saved_name.into(),
        ),
    );
    for (category, (label, piece)) in CUSTOMIZER_CATEGORIES
        .iter()
        .zip(custom.pieces.iter())
        .enumerate()
    {
        summary_row(
            &mut rows,
            "Primary custom character",
            *label,
            customizer_piece_summary(category, piece.get()),
        );
    }
    summary_row(
        &mut rows,
        "Secondary custom character",
        "Name",
        fixed_text(&custom.secondary_name),
    );
    summary_row(
        &mut rows,
        "Secondary custom character",
        "Naming",
        human_enum(
            "customizer.secondary_use_saved_name",
            custom.secondary_use_saved_name.into(),
        ),
    );
    for (category, (label, piece)) in CUSTOMIZER_CATEGORIES
        .iter()
        .zip(custom.secondary_pieces.iter())
        .enumerate()
    {
        summary_row(
            &mut rows,
            "Secondary custom character",
            *label,
            customizer_piece_summary(category, piece.get()),
        );
    }

    render_summary(rows, filter, color)
}

pub fn raw_list_text(save: &Save, fields: &[Property], filter: &str) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "# kind={} payload_size={} checksum_valid={}",
        save.kind().description(),
        save.payload_size,
        save.checksum_valid()
    )
    .expect("write to string");
    for field in fields.iter().filter(|field| field.name.contains(filter)) {
        let data = &save.bytes[field.offset..field.offset + field.size];
        if field.kind == Kind::Bitmask {
            let names = bit_names(save.kind(), field);
            let set: Vec<_> = names
                .iter()
                .enumerate()
                .filter(|(bit, _)| data[*bit / 8] & (1 << (*bit % 8)) != 0)
                .collect();
            write!(output, "# {}: {}; set bits:", field.name, mask_hex(data))
                .expect("write to string");
            if set.is_empty() {
                output.push_str(" none");
            } else {
                for (bit, name) in &set {
                    write!(output, " {bit}={name}").expect("write to string");
                }
            }
            writeln!(output).expect("write to string");
            writeln!(
                output,
                "{}={}",
                field.name,
                if set.is_empty() {
                    "NONE".into()
                } else {
                    set.iter()
                        .map(|(_, name)| name.as_str())
                        .collect::<Vec<_>>()
                        .join("|")
                }
            )
            .expect("write to string");
            continue;
        }

        let enumeration = enum_values(&field.name);
        if !enumeration.values.is_empty() {
            write!(output, "# {}: {}", field.name, mask_hex(data)).expect("write to string");
            if enumeration.flags {
                output.push_str("; set bits:");
                let mut any = false;
                for bit in 0..field.size * 8 {
                    if data[bit / 8] & (1 << (bit % 8)) != 0 {
                        write!(output, " {bit}={}", enum_text(&enumeration, 1u32 << bit))
                            .expect("write to string");
                        any = true;
                    }
                }
                if !any {
                    output.push_str(" none");
                }
            }
            writeln!(output).expect("write to string");
        }
        write!(output, "{}=", field.name).expect("write to string");
        match field.kind {
            Kind::Bytes => {
                output.push_str("hex:");
                for byte in data {
                    write!(output, "{byte:02x}").expect("write to string");
                }
            }
            Kind::Float => {
                let value = f32::from_le_bytes(data.try_into().expect("four-byte float"));
                if value.is_finite() && (value == 0.0 || value.is_normal()) {
                    write!(output, "{value}").expect("write to string");
                } else {
                    output.push_str("0\n");
                    for (index, byte) in data.iter().enumerate() {
                        write!(output, "byte[{}]={byte}", field.offset + index)
                            .expect("write to string");
                        if index + 1 != data.len() {
                            output.push('\n');
                        }
                    }
                }
            }
            Kind::Signed => {
                let value = read_unsigned(data);
                let signed = match field.size {
                    1 => value as i8 as i32,
                    2 => value as i16 as i32,
                    4 => value as i32,
                    _ => unreachable!("supported signed width"),
                };
                write!(output, "{signed}").expect("write to string");
            }
            Kind::Unsigned => {
                write!(output, "{}", enum_text(&enumeration, read_unsigned(data)))
                    .expect("write to string");
            }
            Kind::Bitmask => unreachable!(),
        }
        output.push('\n');
    }
    output
}

fn normalized_name(mut name: String) -> String {
    let mut start = 0;
    while let Some(begin_relative) = name[start..].find('[') {
        let begin = start + begin_relative;
        let Some(end_relative) = name[begin..].find(']') else {
            break;
        };
        let end = begin + end_relative;
        name.replace_range(begin + 1..end, "");
        start = begin + 2;
    }
    name
}

/// Return the currently known purpose of a property, or an explicit reserved-field description.
pub fn property_description(name: &str) -> &'static str {
    match normalized_name(name.to_owned()).as_str() {
        "field_0x0" => {
            "Unidentified byte cleared by NewGame; no field-specific consumer established."
        }
        "field_0x2" => "Two unidentified bytes before the options record; cleared by NewGame.",
        "options_save.field7_0x7" => {
            "Initialized to zero by InitGameBeforeConfig; no further meaning established."
        }
        "options_save.field8_0x8" => {
            "Initialized to zero by InitGameBeforeConfig; no further meaning established."
        }
        "options_save.field9_0x9" => {
            "Unidentified option byte preserved by whole-record copies; no field-specific use established."
        }
        "options_save.field10_0xa" => {
            "Unidentified option byte preserved by whole-record copies; no field-specific use established."
        }
        "level_save[].reserved_0x51" => {
            "Two unidentified bytes after the pickup count; preserved without assigning gameplay meanings."
        }
        "area_save[].reserved_0x7" => {
            "Byte before the aligned challenge-time float; no recovered gameplay use."
        }
        "reserved_0x7c2a" => {
            "Two bytes before the aligned gameplay-time float; no recovered gameplay use."
        }
        "field_0x7c9f" => {
            "Last byte before mission storage; retained separately from the recovered customizer prefix."
        }
        "options.field9_0x16" => {
            "Two trailing SuperOptions bytes; no field-specific consumer established."
        }
        "difficulty" => {
            "AI difficulty threshold; NewGame initializes 5. Used by ResetAICreatures, ManageGameObjects, KillGameObject and ReleaseHearts; not a format version."
        }
        "options_save.player1_rumble" => "Player 1 controller vibration toggle.",
        "options_save.player2_rumble" => "Player 2 controller vibration toggle.",
        "options_save.surround_sound" => {
            "Dolby Pro Logic/surround toggle; passed to NuSound3SetDPL."
        }
        "options_save.sound_volume" => {
            "Sound effects volume, menu range 0..10; multiplied by master_volume/10."
        }
        "options_save.music_volume" => {
            "Music volume, menu range 0..10; multiplied by master_volume/10."
        }
        "options_save.master_volume" => {
            "Master volume, menu range 0..10; scales effects, music and cutscenes."
        }
        "options_save.music_enabled" => "Per-game music toggle checked by GamePlayMusic.",
        "options_save.widescreen" => "Widescreen video toggle.",
        "options_save.brightness" => {
            "Brightness, menu range 0..10; passed to NuVideoSetBrightness divided by 10."
        }
        "level_save[].minikit_names[]" => {
            "Ten fixed 8-byte pickup-name slots per level. First minikit_count slots are used by GizmoPickups_Reset and SuperCounter_AnyCollected."
        }
        "level_save[].minikit_count" => {
            "Number of stored pickup names, gameplay range 0..10. Record stride 84; 366 slots fit before area storage."
        }
        "level_save[].arcade_flags" => {
            "Arcade modes won; Arcade_AwardPoint sets one bit per mode. All three bits complete the area."
        }
        "level_save_padding" => {
            "Three bytes between the level-record capacity and area storage; no recovered field use."
        }
        "shop_hint_purchased_bits" => {
            "96-bit shop-hint purchase set, stored as three u32s. Original ShopHintTab contains only its -1 sentinel, so no named shop entries are established."
        }
        "shop_character_purchased_bits" => {
            "128-bit purchase set, stored as four u32s. Named bits follow the 90 buy_in_shop entries in the shipped collection configuration, not character IDs. Remaining bits are preserved."
        }
        "extra_unlocked_bits" => {
            "64-bit extra-unlock set, stored as two u32s. Names index the original 44-entry Cheat table; codes/red bricks unlock extras."
        }
        "extra_purchased_bits" => {
            "64-bit extra-purchase set, stored as two u32s. Named independently of unlocks and the non-persisted enabled-cheat state."
        }
        "shop_gold_brick_purchased_bits" => {
            "Purchased shop gold bricks: bit i is shop brick i; original completion calculation scans 14 entries."
        }
        "suit_flags" => {
            "Suit availability mask retained from the shared Batman engine. Separate from SuperOptions store entitlements."
        }
        "gold_bricks" => "Earned gold-brick total; AddToGoldBricks caps it at GOLDBRICKPOINTS.",
        "reward_flags" => {
            "One-time completion reward callbacks already fired; bit 0 also enables SuperWeirdo."
        }
        "hub_build_flags" => {
            "Hub construction progress bits used by Hub_Update and GizBuildIt_FinishFn_Game; meanings are indexed by hub build data."
        }
        "indy_unlocked" => {
            "Indiana Jones unlock marker tested by IndyUnlocked_UpdateHint and the hub."
        }
        "gameplay_seconds" => {
            "Accumulated gameplay time in seconds; GameTiming and hub time display."
        }
        "mission_save.best_times[]" => {
            "Twenty mission best-time floats in seconds; mission index follows the configured mission table."
        }
        "mission_save.completed[]" => {
            "Twenty mission completion bytes, following the 80-byte time array."
        }
        "area_save[].minikit_complete" => {
            "Complete minikit-set reward marker, independent of the count at the next byte."
        }
        "area_save[].red_brick_collected" => {
            "Area power/red brick collected marker; BuyAllShopExtras also sets it for each extra's associated area."
        }
        "options.store_pack_flags" => {
            "Eleven individual store entitlements; names recovered from original StorePack data. Initial value 65535 enables all, including reserved bits."
        }
        "options.store_bundle_flags" => {
            "Three purchase-bundle bits; original StoreBundle data names prequel, original trilogy and complete. Initial value 255."
        }
        "options.dpad_locked" => {
            "Virtual D-pad position lock; toggled by its lock button callback."
        }
        "customizer.primary_use_saved_name" => {
            "GameObj_GetName selects the stored primary name when nonzero; otherwise the character's text-table name. NewGame sets SAVED_NAME."
        }
        "customizer.secondary_use_saved_name" => {
            "GameObj_GetName selects the stored secondary name when nonzero; otherwise the character's text-table name. NewGame sets SAVED_NAME."
        }
        "header.field0_0x0" => "Envelope magic: must remain 0x52474d48 (HMGR in file byte order).",
        "header.field1_0x4" => "Envelope version: must remain 1.",
        "header.size" => "Envelope size in bytes: must remain 8232 (0x2028).",
        "header.extradata_offset" => {
            "Bytes skipped after the header before the payload. Edits must preserve this layout."
        }
        "header.platform_data" => {
            "Optional 16-byte PC platform value. The Windows writer copies it from a generic PC API setting; this game does not provide that setting, so observed saves contain zeroes. Android also writes zeroes."
        }
        "header.application_metadata" => {
            "Nominal 1024-code-unit UTF-16 application label. The Windows writer starts it with the configured title and converts a fixed 1024-byte source region, so bytes after the first terminator are incidental and must be preserved. Android leaves the block zeroed."
        }
        "header.slot_metadata" => {
            "Nominal 1024-code-unit UTF-16 slot label. Windows starts it with 'Save Slot N'; its fixed-length conversion also preserves stale temporary-buffer data after the first terminator. Android leaves the block zeroed."
        }
        "header.reserved_metadata" => {
            "Third 1024-code-unit metadata block. It is empty in the inspected Windows and Android writers; no meaning is established."
        }
        "header.timestamp_metadata" => {
            "Nominal 1024-code-unit UTF-16 localized save date/time. Windows builds it with GetDateFormatA and GetTimeFormatA; bytes after the first terminator are incidental conversion tail data. Android leaves the block zeroed."
        }
        "extra_prefix" => {
            "Opaque bytes skipped by the game loader before the payload; preserved verbatim."
        }
        "coins" => "Stored stud/coin balance, as an unsigned integer.",
        "completion" => {
            "Completion points, not a percentage. Displayed percent is points * 100 / game COMPLETIONPOINTS; also feeds slot_code."
        }
        "character_save[]" => {
            "Character ID state: AVAILABLE is owned/playable (Collection_Got); UNLOCKED is CollectIDUnlocked. NewGame sets both for defaults."
        }
        "hint_completion_bits" => {
            "192-bit tutorial set: console IDs 0..95 at bits 0..95; touch IDs 0..95 at bits 96..191. Names identify original Hints_LSW callback and console text ID. Stored as six u32s."
        }
        "area_save[].complete" => {
            "Area availability/progression marker used when choosing the next area; NewGame sets starting areas to 1."
        }
        "area_save[].area_complete" => {
            "Area completion marker; game completion paths set this to 1."
        }
        "area_save[].story_buildup_complete" => {
            "Story-mode stud-buildup completion marker; normally 0 or 1."
        }
        "area_save[].freeplay_buildup_complete" => {
            "Free-play stud-buildup completion marker; normally 0 or 1."
        }
        "area_save[].minikit_count" => {
            "Stored area minikit count. The utility enforces byte width, not gameplay consistency."
        }
        "area_save[].challenge_trial_time" => {
            "Challenge trial time in seconds; NewGame obtains the starting value from area data."
        }
        "episode_save[].superstory_time_limit" => {
            "Stored best Super Story time in seconds; initialized to 3600 for episodes 0..5 and reduced by better times."
        }
        "episode_save[].superstory_score_target" => {
            "Stored best Super Story score; initialized to 100000 for episodes 0..5 and increased by better scores."
        }
        "episode_save[].flags" => {
            "Super Story status word. The low byte becomes 1 on first completion; remaining bits are preserved as numeric values."
        }
        "customizer.pieces[]" => {
            "Nine primary custom-character piece IDs, interpreted using the game's customizer data."
        }
        "customizer.secondary_pieces[]" => {
            "Nine secondary custom-character piece IDs, interpreted using the game's customizer data."
        }
        "customizer.primary_name" => {
            "Primary custom-character name: 32-byte buffer. text:NAME zero-pads and requires room for a terminator."
        }
        "customizer.secondary_name" => {
            "Secondary custom-character name: 32-byte buffer. text:NAME zero-pads and requires room for a terminator."
        }
        "options.touch_controls" => {
            "Touch control selection: 0 selects runtime control mode 1, nonzero selects mode 2."
        }
        "options.left_control_x" => {
            "Saved left touch-control horizontal position; coordinate scale/range is not yet established."
        }
        "options.left_control_y" => {
            "Saved left touch-control vertical position; coordinate scale/range is not yet established."
        }
        "options.right_control_x" => {
            "Saved right touch-control horizontal position; coordinate scale/range is not yet established."
        }
        "options.right_control_y" => {
            "Saved right touch-control vertical position; coordinate scale/range is not yet established."
        }
        "options.music_enabled" => "Music enabled flag: 0 disables music, nonzero enables it.",
        "checksum" => {
            "Derived: ChecksumSaveData(payload), sum of little-endian u32 words plus 0x5c0999 modulo 2^32. Editing requires --keep-derived."
        }
        "slot_code" => {
            "Android game saves derive this from the sign-extended completion field; Android options use 0xffffffff. Windows stores the MakeSaveHash result but has no matching completion field in its payload, so ordinary edits preserve it. Editing it directly requires --keep-derived."
        }
        "byte[]" => {
            "Unidentified/padding byte at an absolute file offset. Raw byte[N] aliases are also available for every named field."
        }
        _ => {
            "Unknown/reserved field: storage layout is recovered, but its meaning is not established. Preserved unless explicitly edited."
        }
    }
}

fn integer_expectation(field: &Property) -> String {
    let signed = field.kind == Kind::Signed;
    let limit = 1u64 << (field.size * 8 - usize::from(signed));
    let mut expectation = format!(
        "{}..{}",
        if signed { -(limit as i64) } else { 0 },
        limit - 1
    );
    let enumeration = enum_values(&field.name);
    if !enumeration.values.is_empty() {
        expectation.push_str("; ");
        expectation.push_str(
            &enumeration
                .values
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        if enumeration.flags {
            expectation.push_str("; combine with |; unknown bits accept numeric values");
        }
    }
    expectation
}
