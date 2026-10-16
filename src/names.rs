use crate::format::SaveKind;

const WINDOWS_SHOP_CHARACTER_NAMES: &str = include_str!("../data/pc/shop_character_names.txt");
const ANDROID_SHOP_CHARACTER_NAMES: &str = include_str!("../data/android/shop_character_names.txt");
const WINDOWS_SHOP_HINT_NAMES: &str = include_str!("../data/pc/shop_hint_names.txt");

const WINDOWS_TUTORIAL_NAMES: &str = include_str!("../data/pc/tutorial_names.txt");
const ANDROID_TUTORIAL_NAMES: &str = include_str!("../data/android/tutorial_names.txt");

pub const EXTRA_NAMES: &[&str] = &[
    "extratoggle",
    "poo",
    "disguises",
    "daisychains",
    "c3pobits",
    "towdeathstar",
    "silhouettes",
    "beepbeep",
    "supergonk",
    "poomoney",
    "walkietalkiedisable",
    "powerbrickdetector",
    "superslap",
    "forcezipup",
    "coinmagnet",
    "disarmtroopers",
    "characterstuds",
    "perfectdeflect",
    "explodingblasterbolts",
    "forcepull",
    "vehiclesmartbomb",
    "superastromech",
    "superjedislam",
    "superthermaldetonator",
    "deflectbolts",
    "darkside",
    "superblasters",
    "fastforce",
    "supersabres",
    "tractorbeam",
    "invincibility",
    "scorex2",
    "selfdestruct",
    "fastbuild",
    "scorex4",
    "regenerate",
    "minikitdetector",
    "scorex6",
    "superzapper",
    "rockets",
    "scorex8",
    "superewokcatapult",
    "infinitetorpedos",
    "scorex10",
];

const WINDOWS_CHARACTER_NAMES: &str = include_str!("../data/pc/character_names.txt");
const ANDROID_CHARACTER_NAMES: &str = include_str!("../data/android/character_names.txt");
const CUSTOMIZER_PIECES: &str = include_str!("../data/customizer_pieces.tsv");
const WINDOWS_AREA_NAMES: &str = include_str!("../data/pc/area_names.txt");
const ANDROID_AREA_NAMES: &str = include_str!("../data/android/area_names.txt");
const WINDOWS_LEVEL_NAMES: &str = include_str!("../data/pc/level_names.txt");
const ANDROID_LEVEL_NAMES: &str = include_str!("../data/android/level_names.txt");
const MISSION_NAMES: &str = include_str!("../data/mission_names.txt");

pub const CUSTOMIZER_CATEGORIES: [&str; 9] = [
    "Hat / hair",
    "Head",
    "Weapon",
    "Arms",
    "Hands",
    "Cape",
    "Body",
    "Underpants",
    "Legs",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomizerPiece {
    pub name: &'static str,
    pub source_character: Option<&'static str>,
}

fn names_for_kind(kind: SaveKind, windows: &'static str, android: &'static str) -> &'static str {
    match kind {
        SaveKind::WindowsGame => windows,
        SaveKind::AndroidGame | SaveKind::AndroidOptions => android,
    }
}

pub fn shop_character_names(kind: SaveKind) -> impl Iterator<Item = &'static str> {
    names_for_kind(
        kind,
        WINDOWS_SHOP_CHARACTER_NAMES,
        ANDROID_SHOP_CHARACTER_NAMES,
    )
    .lines()
}

pub fn character_name(kind: SaveKind, id: usize) -> Option<&'static str> {
    names_for_kind(kind, WINDOWS_CHARACTER_NAMES, ANDROID_CHARACTER_NAMES)
        .lines()
        .nth(id)
}

pub fn tutorial_names(kind: SaveKind) -> impl Iterator<Item = &'static str> {
    names_for_kind(kind, WINDOWS_TUTORIAL_NAMES, ANDROID_TUTORIAL_NAMES).lines()
}

pub fn shop_hint_names(kind: SaveKind) -> impl Iterator<Item = &'static str> {
    names_for_kind(kind, WINDOWS_SHOP_HINT_NAMES, "").lines()
}

pub fn customizer_piece(category: usize, index: u16) -> Option<CustomizerPiece> {
    let category = [
        "hathair",
        "head",
        "weapon",
        "arms",
        "hands",
        "cape",
        "body",
        "underpants",
        "legs",
    ]
    .get(category)?;
    let index = usize::from(index);
    CUSTOMIZER_PIECES
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            (fields.next()? == *category).then(|| CustomizerPiece {
                name: fields.next().expect("generated customizer piece name"),
                source_character: fields.next().filter(|source| !source.is_empty()),
            })
        })
        .nth(index)
}

pub fn area_name(kind: SaveKind, index: usize) -> Option<&'static str> {
    names_for_kind(kind, WINDOWS_AREA_NAMES, ANDROID_AREA_NAMES)
        .lines()
        .nth(index)
}

pub fn level_name(kind: SaveKind, index: usize) -> Option<&'static str> {
    names_for_kind(kind, WINDOWS_LEVEL_NAMES, ANDROID_LEVEL_NAMES)
        .lines()
        .nth(index)
}

pub fn mission_name(index: usize) -> Option<&'static str> {
    MISSION_NAMES.lines().nth(index)
}

#[cfg(test)]
mod tests {
    use super::{
        area_name, character_name, customizer_piece, level_name, mission_name,
        shop_character_names, tutorial_names,
    };
    use crate::format::SaveKind;

    #[test]
    fn character_names_follow_chars_txt_ids() {
        assert_eq!(
            character_name(SaveKind::WindowsGame, 0),
            Some("slave1_lsw1")
        );
        assert_eq!(
            character_name(SaveKind::WindowsGame, 104),
            Some("QuiGonJinn")
        );
        assert_eq!(character_name(SaveKind::WindowsGame, 307), Some("Whip"));
        assert_eq!(
            character_name(SaveKind::WindowsGame, 317),
            Some("hansolo_indy")
        );
        assert_eq!(character_name(SaveKind::AndroidGame, 307), Some("WA7"));
        assert_eq!(character_name(SaveKind::AndroidGame, 317), Some("plokoon"));
        assert_eq!(character_name(SaveKind::AndroidGame, 318), Some("raft"));
        assert_eq!(character_name(SaveKind::AndroidGame, 319), None);
        assert_eq!(character_name(SaveKind::WindowsGame, 339), None);
    }

    #[test]
    fn configured_save_indices_have_game_data_names() {
        assert_eq!(area_name(SaveKind::WindowsGame, 0), Some("Negotiations"));
        assert_eq!(area_name(SaveKind::WindowsGame, 67), Some("LostTemple"));
        assert_eq!(area_name(SaveKind::WindowsGame, 68), None);
        assert_eq!(area_name(SaveKind::AndroidGame, 68), Some("Vehicles"));
        assert_eq!(area_name(SaveKind::AndroidGame, 69), None);
        assert_eq!(level_name(SaveKind::WindowsGame, 0), Some("titles"));
        assert_eq!(level_name(SaveKind::WindowsGame, 91), Some("Jedi_Outro"));
        assert_eq!(level_name(SaveKind::AndroidGame, 92), Some("Jedi_Status"));
        assert_eq!(level_name(SaveKind::WindowsGame, 334), Some("LostTemple_A"));
        assert_eq!(level_name(SaveKind::WindowsGame, 335), None);
        assert_eq!(level_name(SaveKind::AndroidGame, 335), Some("Platform"));
        assert_eq!(level_name(SaveKind::AndroidGame, 336), None);
        assert_eq!(mission_name(0), Some("QuiGonJinn"));
        assert_eq!(mission_name(19), Some("hansolo"));
        assert_eq!(mission_name(20), None);

        let piece = customizer_piece(0, 1).unwrap();
        assert_eq!(piece.name, "hat_hair_01");
        assert_eq!(piece.source_character, Some("QuiGonJinn"));
        assert_eq!(customizer_piece(2, 5).unwrap().name, "blaster_blue");
        assert_eq!(customizer_piece(6, 49).unwrap().name, "Body_50");
        assert_eq!(customizer_piece(8, 34).unwrap().name, "leg_35");
        assert_eq!(customizer_piece(8, 35), None);
        assert_eq!(customizer_piece(0, u16::MAX), None);
    }

    #[test]
    fn tutorials_follow_each_native_table() {
        assert_eq!(tutorial_names(SaveKind::WindowsGame).count(), 64);
        assert_eq!(tutorial_names(SaveKind::AndroidGame).count(), 54);
        assert_eq!(
            tutorial_names(SaveKind::WindowsGame).nth(2),
            Some("Tag_602")
        );
        assert_eq!(
            tutorial_names(SaveKind::WindowsGame).nth(63),
            Some("IndyUnlocked_1447")
        );
        assert_eq!(
            tutorial_names(SaveKind::AndroidGame).nth(2),
            Some("Lever_1548")
        );
        assert_eq!(character_name(SaveKind::WindowsGame, 2), Some("zamwesell"));
        assert_eq!(
            character_name(SaveKind::WindowsGame, 305),
            Some("ZamWesell")
        );
    }

    #[test]
    fn shop_bit_order_is_selected_by_release() {
        assert_eq!(
            shop_character_names(SaveKind::WindowsGame).nth(37),
            Some("macewindu_ep3")
        );
        assert_eq!(shop_character_names(SaveKind::WindowsGame).count(), 88);
        assert_eq!(
            shop_character_names(SaveKind::AndroidGame).nth(37),
            Some("disguisedclone")
        );
        assert_eq!(
            shop_character_names(SaveKind::AndroidGame).nth(39),
            Some("anakin_jedi_scarred")
        );
        assert_eq!(shop_character_names(SaveKind::AndroidGame).count(), 90);
    }
}
