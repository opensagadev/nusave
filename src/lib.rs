//! Library support for reading, inspecting, editing, and writing Nu savegames.
//!
//! Command-line parsing and terminal I/O live in the `nusave` binary. This
//! crate contains only reusable save-format and editing functionality.

mod format;
mod layout;
mod names;
mod schema;

use thiserror::Error;

pub use format::{
    ANDROID_GAME_SIZE, GAME_SIZE, HEADER_SIZE, HeaderMetadata, OPTIONS_SIZE, ParseError, ReadError,
    Save, SaveKind, WINDOWS_GAME_SIZE, WriteError, save_file_name, write_atomic,
};
pub use schema::{
    AssignmentError, Kind, Property, properties, property_description, raw_list_text, summary_text,
};

/// A requested edit could not be applied safely.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EditError {
    /// One assignment was malformed, named no known property, or had an invalid value.
    #[error(transparent)]
    Assignment(#[from] AssignmentError),
    /// An assignment changed or invalidated the structural save envelope.
    #[error("edits would change or invalidate the save layout")]
    InvalidLayout,
}

/// Apply assignments transactionally, leaving `save` unchanged on any error.
pub fn apply_assignments<I, S>(save: &mut Save, assignments: I) -> Result<(), EditError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut edited = save.clone();
    let original_payload = edited.payload;
    let fields = properties(&edited);
    for assignment in assignments {
        schema::assign(&mut edited, &fields, assignment.as_ref())?;
    }
    if !edited.validate_layout() || edited.payload != original_payload {
        return Err(EditError::InvalidLayout);
    }
    *save = edited;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_errors_are_specific_and_transactional() {
        let mut save = Save::fresh(false);
        let original = save.bytes.clone();
        let error = apply_assignments(&mut save, ["coins=12", "coins=-1"]).unwrap_err();

        assert_eq!(save.bytes, original);
        assert!(matches!(
            error,
            EditError::Assignment(AssignmentError::InvalidValue {
                ref property,
                ref value,
                ..
            }) if property == "coins" && value == "-1"
        ));
        assert!(error.to_string().contains("expected 0..4294967295"));
    }

    #[test]
    fn parse_errors_identify_the_bad_header_field() {
        let mut save = Save::fresh(false);
        save.bytes[0] = 0;

        assert_eq!(
            Save::parse(save.bytes).unwrap_err(),
            ParseError::InvalidMagic { found: 0x5247_4d00 }
        );
    }
}
