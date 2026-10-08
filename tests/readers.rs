use nusave::{MAX_SAVE_SIZE, ParseError, ReadError, Save, properties, summary_rows};
use std::io::{self, Cursor, Read};

#[test]
fn reader_writer_preserve_both_save_kinds_and_unknown_bytes() {
    for options in [false, true] {
        let mut save = Save::fresh(options);
        save.bytes[0x1028] = 0xa7;
        let decoded = Save::from_reader(Cursor::new(&save.bytes)).unwrap();
        let mut output = Vec::new();
        decoded.write_to(&mut output).unwrap();
        assert_eq!(output, save.bytes);
        assert!(
            summary_rows(&decoded, &properties(&decoded))
                .iter()
                .any(|row| row.label == "Kind")
        );
    }
}

#[test]
fn reader_stops_at_the_limit_and_reports_io_errors() {
    struct Repeating(usize);
    impl Read for Repeating {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            out.fill(0);
            self.0 += out.len();
            Ok(out.len())
        }
    }
    let mut source = Repeating(0);
    assert!(matches!(
        Save::from_reader(&mut source),
        Err(ReadError::Invalid(ParseError::TooLarge { .. }))
    ));
    assert_eq!(source.0 as u64, MAX_SAVE_SIZE + 1);
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("broken"))
        }
    }
    assert!(matches!(Save::from_reader(Broken), Err(ReadError::Io(_))));
    assert!(matches!(
        Save::parse(vec![0; MAX_SAVE_SIZE as usize + 1]),
        Err(ParseError::TooLarge { .. })
    ));
}

#[test]
fn editor_controls_expose_current_values_and_preserve_unknown_flags() {
    use nusave::{apply_values, property_editor};
    let mut save = Save::fresh(true);
    apply_values(
        &mut save,
        [
            ("options.music_enabled", "1"),
            ("options.store_pack_flags", "32769"),
        ],
    )
    .unwrap();
    let fields = properties(&save);
    let music = property_editor(
        &save,
        fields
            .iter()
            .find(|f| f.name == "options.music_enabled")
            .unwrap(),
    );
    assert_eq!(music.input, "toggle");
    assert_eq!(music.value, "1");
    let flags = property_editor(
        &save,
        fields
            .iter()
            .find(|f| f.name == "options.store_pack_flags")
            .unwrap(),
    );
    assert_eq!(flags.input, "flags");
    assert!(flags.value.contains("32768"));
    let before = save.bytes.clone();
    apply_values(
        &mut save,
        [("options.store_pack_flags", flags.value.as_str())],
    )
    .unwrap();
    assert_eq!(before, save.bytes);
    assert!(
        apply_values(
            &mut save,
            [
                ("options.music_enabled", "0"),
                ("options.touch_controls", "999")
            ]
        )
        .is_err()
    );
    assert_eq!(before, save.bytes);
}
