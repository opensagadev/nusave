use crate::format::SaveKind;

const WINDOWS_SHOP_CHARACTER_NAMES: &str = include_str!("../data/pc/shop_character_names.txt");
const ANDROID_SHOP_CHARACTER_NAMES: &str = include_str!("../data/android/shop_character_names.txt");

pub const TUTORIAL_HINT_BITS: &[(&str, usize)] = &[
    ("hint_356", 0),
    ("GizForce_601", 1),
    ("Lever_1548", 2),
    ("AutoJump_1568", 3),
    ("AutoJump_1569", 4),
    ("GizBuildIts_604", 5),
    ("Attack_631", 6),
    ("Dodge_613", 7),
    ("Dodge_1501", 8),
    ("ZipUps_610", 9),
    ("Tag_602", 10),
    ("Push_615", 11),
    ("Teleport_616", 12),
    ("GizPanel_617", 13),
    ("GizPanel_618", 14),
    ("Tag_605", 15),
    ("Tag_606", 16),
    ("Tag_611", 17),
    ("Tag_650", 18),
    ("Tag_651", 19),
    ("ShinyMetal_643", 20),
    ("SmartBomb_1505", 21),
    ("GizPanel_607", 22),
    ("GizPanel_608", 23),
    ("GizPanel_609", 24),
    ("Shop_1516", 25),
    ("Shop_1515", 26),
    ("Shop_1514", 27),
    ("DragBomb_1518", 28),
    ("DragBomb_654", 29),
    ("Move_1521", 30),
    ("Move_1523", 31),
    ("Jump_1540", 32),
    ("Move_1522", 33),
    ("Jump_1542", 34),
    ("PlayerButton_1524", 35),
    ("PlayerButton_1525", 36),
    ("Jump_1544", 37),
    ("Jump_1546", 38),
    ("HoldTag_1526", 39),
    ("UnlockHubStuff_1561", 40),
    ("UnlockHubStuff_1562", 41),
    ("UnlockHubStuff_1563", 42),
    ("Tag_649", 43),
    ("VehicleStuff_1559", 44),
    ("ShinyMetal_1565", 45),
    ("ShinyMetal_696", 46),
    ("Teleport_1570", 47),
    ("Sith_1571", 48),
    ("GizPanel_1572", 49),
    ("GizPanel_1573", 50),
    ("HatMachine_1575", 51),
    ("HatMachine_1576", 52),
    ("Jump_1577", 53),
];

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

pub fn customizer_piece(category: usize, index: i16) -> Option<CustomizerPiece> {
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
    let index = usize::try_from(index).ok()?;
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
        area_name, character_name, customizer_piece, level_name, mission_name, shop_character_names,
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
        assert_eq!(area_name(SaveKind::WindowsGame, 69), Some("LostTemple"));
        assert_eq!(area_name(SaveKind::WindowsGame, 70), None);
        assert_eq!(area_name(SaveKind::AndroidGame, 70), Some("Vehicles"));
        assert_eq!(level_name(SaveKind::WindowsGame, 0), Some("titles"));
        assert_eq!(level_name(SaveKind::WindowsGame, 349), Some("LostTemple_A"));
        assert_eq!(level_name(SaveKind::WindowsGame, 350), None);
        assert_eq!(level_name(SaveKind::AndroidGame, 350), Some("Platform"));
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
        assert_eq!(customizer_piece(0, -1), None);
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
