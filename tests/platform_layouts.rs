use nusave::{
    HEADER_SIZE, Kind, Save, SaveKind, apply_assignments, properties, property_editor,
    raw_list_text, summary_rows,
};

// Native offsets, deliberately independent of nusave's private Rust layouts.
fn specimen(pc: bool, extra: usize) -> Save {
    let size = if pc { 0x7e4c } else { 0x7e58 };
    let mut bytes = vec![0; HEADER_SIZE + extra + size + 8];
    bytes[0..4].copy_from_slice(&0x5247_4d48u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&0x2028u32.to_le_bytes());
    bytes[20..24].copy_from_slice(&(extra as u32).to_le_bytes());
    bytes[HEADER_SIZE..HEADER_SIZE + extra].fill(0xa5);
    Save::parse(bytes).unwrap()
}

fn offsets(pc: bool) -> (usize, usize) {
    if pc {
        (0x7c14, 0x7c20)
    } else {
        (0x7c20, 0x7c2c)
    }
}

fn set_u16(save: &mut Save, offset: usize, value: u16) {
    let start = save.payload + offset;
    save.bytes[start..start + 2].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn properties_cover_native_offsets_once_in_every_platform() {
    for pc in [true, false] {
        let save = specimen(pc, 3);
        let (progress, suffix) = offsets(pc);
        let fields = properties(&save);
        for (name, offset, size, kind) in [
            (
                "hint_completion_bits",
                0x7c08,
                if pc { 12 } else { 24 },
                Kind::Bitmask,
            ),
            ("coins", progress, 4, Kind::Unsigned),
            ("completion", progress + 4, 2, Kind::Unsigned),
            ("gold_bricks", progress + 6, 1, Kind::Unsigned),
            ("reward_flags", progress + 7, 1, Kind::Unsigned),
            ("hub_build_flags", progress + 8, 1, Kind::Unsigned),
            ("indy_unlocked", progress + 9, 1, Kind::Unsigned),
            ("gameplay_seconds", suffix, 4, Kind::Float),
            ("customizer.pieces[2]", suffix + 8, 2, Kind::Unsigned),
            (
                "customizer.secondary_pieces[6]",
                suffix + 0x48,
                2,
                Kind::Unsigned,
            ),
            (
                "customizer.secondary_padding",
                suffix + 0x71,
                3,
                Kind::Bytes,
            ),
            ("mission_save.best_times[0]", suffix + 0x74, 4, Kind::Float),
            ("character_save[2]", suffix + 0xda, 1, Kind::Unsigned),
            (
                "area_save[0].story_true_jedi_complete",
                0x782e,
                1,
                Kind::Unsigned,
            ),
            (
                "area_save[0].freeplay_true_jedi_complete",
                0x782f,
                1,
                Kind::Unsigned,
            ),
            ("area_save[0].challenge_complete", 0x7833, 1, Kind::Unsigned),
            (
                "episode_save[0].superstory_complete",
                0x7b94,
                1,
                Kind::Unsigned,
            ),
            ("episode_save[0].reserved_0x9", 0x7b95, 3, Kind::Bytes),
        ] {
            let field = fields.iter().find(|field| field.name == name).unwrap();
            assert_eq!(
                (field.offset, field.size, field.kind),
                (save.payload + offset, size, kind),
                "{pc}: {name}"
            );
        }
        let mut covered = vec![0; save.bytes.len()];
        for field in fields {
            for byte in &mut covered[field.offset..field.offset + field.size] {
                *byte += 1;
            }
        }
        assert!(covered.iter().all(|&count| count == 1));
    }
}

#[test]
fn tutorials_cannot_overwrite_progress_and_use_native_names() {
    for pc in [true, false] {
        let mut save = specimen(pc, 0);
        let (progress, _) = offsets(pc);
        save.bytes[save.payload + progress..save.payload + progress + 12].fill(0x5a);
        let before = save.bytes.clone();
        let value = if pc {
            "Tag_602|IndyUnlocked_1447|BIT_95"
        } else {
            "console.Lever_1548|touch.Lever_1548|BIT_191"
        };
        apply_assignments(&mut save, [format!("hint_completion_bits={value}")]).unwrap();
        let end = save.payload + progress;
        assert_eq!(
            &before[..save.payload + 0x7c08],
            &save.bytes[..save.payload + 0x7c08]
        );
        assert_eq!(&before[end..], &save.bytes[end..]);
        assert_eq!(save.bytes[save.payload + 0x7c08], 4);
        if !pc {
            assert_eq!(save.bytes[save.payload + 0x7c14], 4);
        }
        let raw = raw_list_text(&save, &properties(&save), "hint_completion_bits");
        assert!(raw.contains(value));
        let before_invalid = save.bytes.clone();
        let invalid: &[&str] = if pc {
            &[
                "BIT_96",
                "bank1.GizForce_601",
                "console.Lever_1548",
                "0x1000000000000000000000000",
            ]
        } else {
            &["BIT_192", "Tag_600"]
        };
        for value in invalid {
            assert!(
                apply_assignments(
                    &mut save,
                    [
                        "coins=1".to_owned(),
                        format!("hint_completion_bits={value}")
                    ]
                )
                .is_err()
            );
            assert_eq!(save.bytes, before_invalid);
        }
        let bad_word = if pc { 3 } else { 6 };
        assert!(
            apply_assignments(&mut save, [format!("hint_completion_bits[{bad_word}]=1")]).is_err()
        );
    }
}

#[test]
fn progress_edits_touch_only_the_native_fields_and_derived_footer() {
    for pc in [true, false] {
        let mut save = specimen(pc, 1);
        let (progress, _) = offsets(pc);
        let assignments = [
            ("coins=123456", progress, 4),
            ("completion=32769", progress + 4, 2),
            ("gold_bricks=160", progress + 6, 1),
            ("reward_flags=100_PERCENT|ALL_GOLD_BRICKS", progress + 7, 1),
            ("hub_build_flags=BUILD_2|LEVEL_BUILD", progress + 8, 1),
            ("indy_unlocked=COMPLETE", progress + 9, 1),
        ];
        for (assignment, offset, size) in assignments {
            let before = save.bytes.clone();
            apply_assignments(&mut save, [assignment]).unwrap();
            assert_eq!(
                &before[..save.payload + offset],
                &save.bytes[..save.payload + offset]
            );
            assert_eq!(
                &before[save.payload + offset + size..],
                &save.bytes[save.payload + offset + size..]
            );
        }
        let before_derive = save.bytes.clone();
        save.derive();
        assert_eq!(
            &before_derive[..save.payload + save.payload_size],
            &save.bytes[..save.payload + save.payload_size]
        );
        assert!(save.checksum_valid());
        assert_eq!(save.slot_code(), if pc { 0x8001 } else { 0xffff8001 });
        let rows = summary_rows(&save, &properties(&save));
        assert!(
            rows.iter()
                .any(|row| row.label == "Studs" && row.value == "123,456")
        );
        assert!(
            rows.iter()
                .any(|row| row.label == "Gold bricks" && row.value == "160")
        );
    }
}

#[test]
fn records_use_unsigned_pieces_completion_bytes_and_exact_legacy_views() {
    for pc in [true, false] {
        let mut save = specimen(pc, 0);
        let (_, suffix) = offsets(pc);
        apply_assignments(
            &mut save,
            [
                "customizer.pieces[2]=65535",
                "customizer.secondary_pieces[2]=5",
                "customizer.secondary_pieces[6]=49",
                "customizer.secondary_padding=hex:aabbcc",
                "customizer.field_0x6d=hex:1122",
                "customizer.field_0x35=hex:445566",
                "episode_save[0].flags=0xaabbcc02",
                "episode_save[0].superstory_complete=COMPLETE",
                "episode_save[1].superstory_complete=2",
                "episode_save[2].flags=0x100",
                "area_save[0].story_buildup_complete=COMPLETE",
                "area_save[0].freeplay_buildup_complete=COMPLETE",
                "area_save[0].reserved_0x7=COMPLETE",
            ],
        )
        .unwrap();
        assert_eq!(
            &save.bytes[save.payload + suffix + 0x71..save.payload + suffix + 0x74],
            &[0x11, 0x22, 0xcc]
        );
        assert_eq!(
            &save.bytes[save.payload + 0x7b94..save.payload + 0x7b98],
            &[1, 0xcc, 0xbb, 0xaa]
        );
        let rows = summary_rows(&save, &properties(&save));
        assert!(
            rows.iter()
                .any(|r| r.label == "Episode 2" && r.value.starts_with("Completed;"))
        );
        assert!(
            rows.iter()
                .any(|r| r.label == "Episode 3" && r.value.starts_with("Not completed;"))
        );
        assert!(
            rows.iter()
                .any(|r| r.value.contains("story True Jedi complete")
                    && r.value.contains("chapter challenge complete"))
        );
        assert!(
            rows.iter()
                .any(|r| r.label == "Body" && r.value == "Body_50 (atat_driver)")
        );
        assert!(
            rows.iter()
                .any(|r| r.label == "Weapon" && r.value == "blaster_blue")
        );
        let field = properties(&save)
            .into_iter()
            .find(|f| f.name == "customizer.pieces[2]")
            .unwrap();
        let editor = property_editor(&save, &field);
        assert_eq!(editor.value, "65535");
        let before = save.bytes.clone();
        assert!(apply_assignments(&mut save, ["customizer.pieces[2]=-1"]).is_err());
        assert!(apply_assignments(&mut save, ["customizer.pieces[2]=65536"]).is_err());
        assert_eq!(save.bytes, before);
        apply_assignments(
            &mut save,
            [if pc {
                "field_0x7c93=221"
            } else {
                "field_0x7c9f=221"
            }],
        )
        .unwrap();
        assert_eq!(save.bytes[save.payload + suffix + 0x73], 221);
        apply_assignments(
            &mut save,
            [
                "customizer.field_0x6d[1]=153",
                "customizer.field_0x35[2]=231",
            ],
        )
        .unwrap();
        assert_eq!(
            &save.bytes[save.payload + suffix + 0x71..save.payload + suffix + 0x74],
            &[0x11, 0x99, 221]
        );
        assert_eq!(
            &save.bytes[save.payload + suffix + 0x39..save.payload + suffix + 0x3c],
            &[0x44, 0x55, 0xe7]
        );
    }
}

#[test]
fn shop_gold_bricks_and_inherited_suit_indices_are_named_correctly() {
    for pc in [true, false] {
        let mut save = specimen(pc, 0);
        apply_assignments(
            &mut save,
            [
                "shop_gold_brick_purchased_bits=BIT_7|BIT_8|BIT_13",
                "suit_flags=BATMAN|ROBIN",
            ],
        )
        .unwrap();
        assert_eq!(save.read_u32(save.payload + 0x7bfc), 0x21);
        apply_assignments(&mut save, ["shop_hint_purchased_bits=BIT_5|BIT_13|BIT_95"]).unwrap();
        let hints = raw_list_text(&save, &properties(&save), "shop_hint_purchased_bits");
        assert!(hints.contains(if pc {
            "hint_635|hint_643|BIT_95"
        } else {
            "BIT_5|BIT_13|BIT_95"
        }));
        let raw = raw_list_text(&save, &properties(&save), "shop_gold_brick_purchased_bits");
        assert!(raw.contains(if pc {
            "GOLD_BRICK_7|GOLD_BRICK_8|GOLD_BRICK_13"
        } else {
            "GOLD_BRICK_7|BIT_8|BIT_13"
        }));
        assert!(
            summary_rows(&save, &properties(&save))
                .iter()
                .any(|row| row.label == "Suit availability" && row.value == "Batman, Robin")
        );
        if !pc {
            assert!(
                apply_assignments(&mut save, ["shop_gold_brick_purchased_bits=GOLD_BRICK_8"])
                    .is_err()
            );
        }
    }
}

#[test]
fn raw_export_roundtrips_both_platforms_with_opaque_and_nonfinite_data() {
    for pc in [true, false] {
        let mut save = specimen(pc, 3);
        let (progress, suffix) = offsets(pc);
        // Exercise every preserved byte, not just freshly zeroed records.
        for (i, byte) in save.bytes[HEADER_SIZE..].iter_mut().enumerate() {
            *byte = (i.wrapping_mul(71) & 255) as u8;
        }
        set_u16(&mut save, progress + 4, 816);
        save.write_u32(save.payload + suffix, 0x7fc01234);
        let params = raw_list_text(&save, &properties(&save), "");
        let mut rebuilt = specimen(pc, 3);
        apply_assignments(
            &mut rebuilt,
            params
                .lines()
                .filter(|line| !line.starts_with('#') && !line.is_empty()),
        )
        .unwrap();
        assert_eq!(rebuilt.bytes, save.bytes);
        assert_eq!(
            rebuilt.kind(),
            if pc {
                SaveKind::WindowsGame
            } else {
                SaveKind::AndroidGame
            }
        );
    }
}
