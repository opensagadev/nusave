# nusave

[![CI](https://github.com/opensagadev/nusave/actions/workflows/ci.yml/badge.svg)](https://github.com/opensagadev/nusave/actions/workflows/ci.yml)

`nusave` is a standalone command-line viewer, editor, and format inspector for the savegames used by
**LEGO Star Wars: The Complete Saga**. It can display an interpreted save summary, export reusable
assignments, edit a slot safely, or create a deterministic new save
without requiring the game or any game assets.

The implementation is a Rust port of the save tooling developed in
[`opensagadev/saga`](https://github.com/opensagadev/saga). It preserves unknown bytes instead of
normalizing the file, understands both game-progress and `SuperOptions` records, and reproduces
the original tool's serialized output.

## Building

Rust 1.88 or newer is required.

```console
git clone https://github.com/opensagadev/nusave.git
cd nusave
cargo build --release
```

The executable is written to `target/release/nusave` (`nusave.exe` on Windows). To install it
directly from GitHub:

```console
cargo install --git https://github.com/opensagadev/nusave.git
```

## Command line

```text
nusave [OPTIONS] [SAVEGAME_FOLDER] [COMMAND]
```

`SAVEGAME_FOLDER` is optional and defaults to `res/SavedGames`. The program constructs the
game's filename inside that folder:

| Slot | File | Contents |
| ---: | --- | --- |
| 0–2 | `SaveGameN.LEGO Star Wars - The Complete Saga_SavedGame` | game progress |
| 3 | `SaveGame3.LEGO Star Wars - The Complete Saga_SavedGame` | standalone `SuperOptions` |

Slot 0 is the default. Select another with the global `--slot N` option. Running without a
command is the same as `list`.

### Inspect a save

```console
nusave "/path/to/SavedGames" list
nusave --slot 1 "/path/to/SavedGames" list --filter minikit
nusave --slot 3 "/path/to/SavedGames" list
```

By default, `list` produces a compact, human-readable report. It groups progress, settings,
collections, episodes, changed areas and levels, missions, characters, and custom characters.
Stored enum and flag numbers are translated to their known meanings, empty repeated records are
collapsed, and unknown envelope/padding data is left out. Configured areas, levels, missions, and
customizer pieces use their shipped game-data names.

Each group uses compact, aligned key/value columns. Values are never wrapped, and sections follow
one another without spacer lines. Section titles, labels, and states use restrained color when
stdout is a terminal; redirected output does not contain ANSI escape codes.

```text
Progress
  Studs              125,000
  Completion points  42
  Gameplay time      3h 14m 08s
  Indiana Jones      Unlocked
  Suit abilities     Shadow, Technology
```

For format work and exact reconstruction, `--raw` writes every property as `key=value`. The raw
output is deliberately accepted as a parameter file, including unknown header and payload bytes:

```console
nusave "/path/to/SavedGames" list --raw > slot.params
nusave "/other/SavedGames" create --params slot.params --keep-derived
```

The first comment in a raw listing reports the payload size and whether the stored checksum is
valid. `--filter TEXT` searches interpreted labels and values in the normal view, or property
names in the raw view.

### Edit a save

```console
nusave "/path/to/SavedGames" edit coins=100000 gold_bricks=160
nusave --slot 1 "/path/to/SavedGames" edit \
  'area_save[2].complete=COMPLETE' \
  'shop_character_purchased_bits=gonkdroid|wookie'
nusave "/path/to/SavedGames" edit --output test-save.bin completion=32769
nusave --slot 3 "/path/to/SavedGames" edit options.music_enabled=ON
```

Assignments from each `--params FILE` are applied in command-line order, followed by direct
assignments. Quote assignments containing `|`, `[` or `]` when the shell treats those characters
specially.

By default `edit` updates the checksum and trailing slot code, writes a sibling temporary file,
flushes it, then atomically replaces the selected slot. `--output PATH` leaves the input alone
and refuses to overwrite an existing destination. Parsing or assignment errors occur before any
output is published.

### Create a save

```console
nusave "/path/to/SavedGames" create
nusave --slot 2 "/path/to/SavedGames" create coins=50000
nusave "/path/to/SavedGames" create --from template.bin --params changes.params
nusave "/path/to/SavedGames" create --options options.music_enabled=ON
```

`create` makes the destination folder if necessary and refuses to replace an existing slot.
Without `--from`, a game save starts from the deterministic, asset-independent state used by the
original harness: difficulty 5, the first area available, six one-hour/100,000-point Super Story
targets, suit flags `0x21`, and both custom characters set to use their stored names. Fields whose
defaults normally depend on loaded game assets remain zero. `--options` creates slot 3 and cannot
be combined with `--from`.

## Assignment syntax

The complete property inventory is documented under [Binary format](#binary-format). The general
input forms are:

| Stored type | Accepted input | Example |
| --- | --- | --- |
| unsigned integer | decimal, `0x` hexadecimal, or a named enum | `coins=250000` |
| signed integer | signed decimal or `0x` hexadecimal | `customizer.pieces[0]=-1` |
| `f32` | finite decimal or scientific notation | `gameplay_seconds=12.5` |
| fixed bytes | exact `hex:` data, `text:` data padded with zeros, or an indexed byte | `customizer.primary_name=text:Leia` |
| bitmask | a full-width number, `NONE`, or names/numbers joined by `|` | `reward_flags=100_PERCENT|ALL_GOLD_BRICKS` |

Logical masks such as `shop_character_purchased_bits` and `hint_completion_bits` span their full
storage width, so they do not need to be edited one 32-bit word at a time. Unknown bits always
remain addressable as `BIT_n`. Every stored byte can also be addressed by its absolute file offset
as `byte[N]`. When `list` encounters a non-finite, subnormal, or otherwise awkward floating-point
bit pattern, it emits byte assignments so a round trip remains exact.

The envelope magic, envelope version, header size, and extra-data offset are structural and may
only be assigned their current valid values. `checksum` and `slot_code` are derived after edits;
`--keep-derived` is available for byte-for-byte reconstruction and format research.

## Binary format

All numeric fields are little-endian. A file is an envelope followed by an optional opaque prefix,
one recognized payload, and two derived 32-bit words:

```text
0x0000  SaveHeader                 0x2028 bytes
0x2028  opaque extra prefix        header.extra_data_offset bytes
        payload                    0x7e58 (game) or 0x18 (options) bytes
        checksum                   u32
        slot_code                  u32
```

With no extra prefix, a game save is `0x9e88` (40,584) bytes and an options save is `0x2048`
(8,264) bytes. The parser accepts an opaque prefix because real containers may place extra data
between the fixed header and payload. Its size is the signed 32-bit value at header offset `0x14`.
The input safety limit is 16 MiB.

### Schema notation

The tables below are the complete editable schema. Header offsets are absolute file offsets;
payload offsets are relative to `P = 0x2028 + header.extradata_offset`. Array ranges are inclusive,
indices are zero-based, and `stride` is measured in bytes. Integer ranges follow their storage
width even where the game normally uses a smaller range. Unknown fields are deliberately named
and preserved instead of being assigned speculative meanings.

The storage ranges are `u8=0..255`, `u16=0..65535`, `u32=0..4294967295`,
`i16=-32768..32767`, and `i32=-2147483648..2147483647`. `f32` assignments must be finite.
`bytes[N]` accepts `hex:` followed by exactly `2N` hexadecimal digits or `text:` followed by at
most `N-1` bytes, leaving room for a zero terminator.

Any fixed byte array also supports `property[index]=0..255`, every byte in the file supports the
absolute `byte[offset]=0..255` alias, and wide bitmasks support their named bits, `BIT_n`, a decimal
integer, or a `0x` hexadecimal integer. `wide_mask[word]` addresses one zero-based 32-bit word of
the mask directly.

### Common envelope schema

| Property | File offset | Type | Meaning / constraint |
| --- | ---: | --- | --- |
| `header.field0_0x0` | `0x0000` | `i32` | Magic; must remain `0x52474d48` (`HMGR` in file byte order). |
| `header.field1_0x4` | `0x0004` | `i32` | Envelope version; must remain `1`. |
| `header.size` | `0x0008` | `i32` | Header size; must remain `0x2028`. |
| `header.field3_0xc` | `0x000c` | `i32` | Unknown signed word. |
| `header.field4_0x10` | `0x0010` | `i32` | Unknown signed word. |
| `header.extradata_offset` | `0x0014` | `i32` | Opaque-prefix size; edits must preserve the parsed layout. |
| `header.field6_0x18` | `0x0018` | `bytes[16]` | Unknown bytes. |
| `header.field7_0x28` | `0x0028` | `i16` | Unknown signed word. |
| `header.field8_0x2a` | `0x002a` | `bytes[2046]` | Unknown bytes. |
| `header.field9_0x828` | `0x0828` | `i16` | Unknown signed word. |
| `header.field10_0x82a` | `0x082a` | `bytes[2046]` | Unknown bytes. |
| `header.field11_0x1028` | `0x1028` | `i16` | Unknown signed word. |
| `header.field12_0x102a` | `0x102a` | `bytes[2046]` | Unknown bytes. |
| `header.field13_0x1828` | `0x1828` | `i16` | Unknown signed word. |
| `header.field14_0x182a` | `0x182a` | `bytes[2046]` | Unknown bytes. |
| `extra_prefix` | `0x2028` | `bytes[extradata_offset]` | Optional opaque bytes before the payload. |
| `checksum` | `P + payload_size` | `u32` | Derived payload checksum; directly editable only with `--keep-derived`. |
| `slot_code` | `P + payload_size + 4` | `u32` | Derived completion code, or `0xffffffff` for options. |

### Game payload schema (`GAMESAVE_s`, `0x7e58` bytes)

#### Top-level and embedded options

| Property | Payload offset | Type | Meaning / accepted symbolic values |
| --- | ---: | --- | --- |
| `field_0x0` | `0x0000` | `u8` | Unknown; cleared by `NewGame`. |
| `difficulty` | `0x0001` | `u8` | AI difficulty threshold; initialized to `5`. |
| `field_0x2` | `0x0002` | `bytes[2]` | Unknown. |
| `options_save.player1_rumble` | `0x0004` | `u8` | `OFF=0`, `ON=1`. |
| `options_save.player2_rumble` | `0x0005` | `u8` | `OFF=0`, `ON=1`. |
| `options_save.surround_sound` | `0x0006` | `u8` | `OFF=0`, `ON=1`. |
| `options_save.sound_volume` | `0x0007` | `u8` | Effects volume; menu range `0..10`. |
| `options_save.music_volume` | `0x0008` | `u8` | Music volume; menu range `0..10`. |
| `options_save.master_volume` | `0x0009` | `u8` | Master volume; menu range `0..10`. |
| `options_save.music_enabled` | `0x000a` | `u8` | `OFF=0`, `ON=1`. |
| `options_save.field7_0x7` | `0x000b` | `u8` | Unknown; initialized to zero. |
| `options_save.field8_0x8` | `0x000c` | `u8` | Unknown; initialized to zero. |
| `options_save.field9_0x9` | `0x000d` | `u8` | Unknown. |
| `options_save.field10_0xa` | `0x000e` | `u8` | Unknown. |
| `options_save.widescreen` | `0x000f` | `u8` | `OFF=0`, `ON=1`. |
| `options_save.brightness` | `0x0010` | `u8` | Brightness; menu range `0..10`. |

#### Repeated records

| Property | First payload offset | Type / count / stride | Meaning / accepted symbolic values |
| --- | ---: | --- | --- |
| `level_save[0..365].minikit_names[0..9]` | `0x0011 + 8 × name` | `bytes[8]`, 366 records, stride `0x54` | Ten fixed pickup-name slots; `text:` accepts at most 7 bytes. |
| `level_save[0..365].minikit_count` | `0x0061` | `u8`, 366, stride `0x54` | Used-name count; gameplay range `0..10`. |
| `level_save[0..365].reserved_0x51` | `0x0062` | `bytes[2]`, 366, stride `0x54` | Unknown. |
| `level_save[0..365].arcade_flags` | `0x0064` | `u8`, 366, stride `0x54` | `NONE=0`, `BATTLE=1`, `COLLECT=2`, `HUNT=4`; flags combine with `|`. |
| `level_save_padding` | `0x7829` | `bytes[3]` | Padding between level and area storage. |
| `area_save[0..71].complete` | `0x782c` | `u8`, 72, stride `0x0c` | Area availability: `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].area_complete` | `0x782d` | `u8`, 72, stride `0x0c` | Area completion: `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].story_buildup_complete` | `0x782e` | `u8`, 72, stride `0x0c` | `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].freeplay_buildup_complete` | `0x782f` | `u8`, 72, stride `0x0c` | `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].minikit_complete` | `0x7830` | `u8`, 72, stride `0x0c` | Full-set reward: `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].minikit_count` | `0x7831` | `u8`, 72, stride `0x0c` | Stored area minikit count. |
| `area_save[0..71].red_brick_collected` | `0x7832` | `u8`, 72, stride `0x0c` | `INCOMPLETE=0`, `COMPLETE=1`. |
| `area_save[0..71].reserved_0x7` | `0x7833` | `u8`, 72, stride `0x0c` | Unknown. |
| `area_save[0..71].challenge_trial_time` | `0x7834` | `f32`, 72, stride `0x0c` | Stored challenge time in seconds. |
| `episode_save[0..5].superstory_time_limit` | `0x7b8c` | `f32`, 6, stride `0x0c` | Stored Super Story time; initialized to 3600 seconds. |
| `episode_save[0..5].superstory_score_target` | `0x7b90` | `i32`, 6, stride `0x0c` | Stored score; initialized to 100,000. |
| `episode_save[0..5].flags` | `0x7b94` | `u32`, 6, stride `0x0c` | `NONE=0`, `SUPERSTORY_COMPLETE=1`; flags combine with `|`. |

Super Story has an explicit completion bit: bit 0 of `episode_save[].flags`. Challenge trials do
not have a dedicated completion flag. The game initializes their stored times from `AREAS.TXT` and
later compares the stored value with that configured value. `nusave list` therefore reports the
stored challenge value without inferring a completion state.

#### Collections and progress

| Property | Payload offset | Type | Meaning / accepted symbolic values |
| --- | ---: | --- | --- |
| `shop_hint_purchased_bits` | `0x7bd4` | `bits96` | Purchased hints; no names are known, so use `BIT_0..BIT_95`. |
| `shop_character_purchased_bits` | `0x7be0` | `bits128` | Purchased shop entries; named bits `0..89` are listed below. |
| `extra_unlocked_bits` | `0x7bf0` | `bits64` | Unlocked extras; named bits `0..43` are listed below. |
| `shop_gold_brick_purchased_bits` | `0x7bf8` | `bits32` | `GOLD_BRICK_0..GOLD_BRICK_13`; bits `14..31` remain `BIT_n`. |
| `suit_flags` | `0x7bfc` | `u32` | `NONE=0`, `SHADOW=1`, `GLIDE=2`, `DEMOLITION=4`, `SONAR=8`, `WATER=16`, `TECHNOLOGY=32`, `MAGNET=64`, `ATTRACT=128`, `ALL=0xffffffff`. |
| `extra_purchased_bits` | `0x7c00` | `bits64` | Purchased extras; same named bits `0..43` as the unlock mask. |
| `hint_completion_bits` | `0x7c08` | `bits192` | Console tutorial bits `0..95`, touch bits `96..191`; names below. |
| `coins` | `0x7c20` | `u32` | Stored stud balance. |
| `completion` | `0x7c24` | `u16` | Completion points, not a percentage; also feeds `slot_code`. |
| `gold_bricks` | `0x7c26` | `u8` | Earned gold-brick total. |
| `reward_flags` | `0x7c27` | `u8` | `NONE=0`, `100_PERCENT=1`, `ALL_GOLD_BRICKS=2`; flags combine with `|`. |
| `hub_build_flags` | `0x7c28` | `u8` | `NONE=0`, `BUILD_0..BUILD_6=1<<n`, `LEVEL_BUILD=128`; flags combine with `|`. |
| `indy_unlocked` | `0x7c29` | `u8` | `INCOMPLETE=0`, `COMPLETE=1`. |
| `reserved_0x7c2a` | `0x7c2a` | `bytes[2]` | Unknown alignment bytes. |
| `gameplay_seconds` | `0x7c2c` | `f32` | Accumulated gameplay time in seconds. |

#### Custom characters, missions, and character state

| Property | Payload offset | Type / count | Meaning / accepted symbolic values |
| --- | ---: | --- | --- |
| `customizer.pieces[0..8]` | `0x7c30` | `i16`, 9, stride 2 | Primary piece indices: hat/hair, head, cape, body, arms, hands, weapon, underpants, legs. |
| `customizer.field_0x12` | `0x7c42` | `bytes[2]` | Unknown. |
| `customizer.primary_name` | `0x7c44` | `bytes[32]` | `text:` accepts at most 31 bytes and zero-pads. |
| `customizer.primary_use_saved_name` | `0x7c64` | `u8` | `CHARACTER_DEFAULT=0`, `SAVED_NAME=1`. |
| `customizer.field_0x35` | `0x7c65` | `bytes[3]` | Unknown. |
| `customizer.secondary_pieces[0..8]` | `0x7c68` | `i16`, 9, stride 2 | Secondary piece indices in the same category order. |
| `customizer.field_0x4a` | `0x7c7a` | `bytes[2]` | Unknown. |
| `customizer.secondary_name` | `0x7c7c` | `bytes[32]` | `text:` accepts at most 31 bytes and zero-pads. |
| `customizer.secondary_use_saved_name` | `0x7c9c` | `u8` | `CHARACTER_DEFAULT=0`, `SAVED_NAME=1`. |
| `customizer.field_0x6d` | `0x7c9d` | `bytes[2]` | Unknown. |
| `field_0x7c9f` | `0x7c9f` | `u8` | Unknown byte before mission storage. |
| `mission_save.best_times[0..19]` | `0x7ca0` | `f32`, 20, stride 4 | Mission best times in seconds. |
| `mission_save.completed[0..19]` | `0x7cf0` | `u8`, 20, stride 1 | `INCOMPLETE=0`, `COMPLETE=1`. |
| `character_save[0..339]` | `0x7d04` | `u8`, 340, stride 1 | `NONE=0`, `AVAILABLE=1`, `UNLOCKED=2`; flags combine with `|`. |

Character-state indices map to the `file` identifiers in `CHARS/CHARS.TXT`. The bundled complete
mapping for IDs 0–318 is in [`data/character_names.txt`](data/character_names.txt); it was extracted
from the PC `GAME.DAT` with [`nudat`](https://github.com/opensagadev/nudat) in the exact
`char_start` order used by `ConfigureCharacterList`. IDs 319–339 are reserved but unnamed.

The corresponding shipped-index mappings for customizer pieces, areas, levels, and missions are
embedded from `CHARS/CUSTOMISER.TXT`, `LEVELS/AREAS.TXT`, `LEVELS/LEVELS.TXT`, and
`LEVELS/MISSIONS.TXT`. They are used only to name stored indices; the program does not reinterpret
or normalize the save values.

### Options payload schema (`SUPEROPTIONS_s`, `0x18` bytes)

| Property | Payload offset | Type | Meaning / accepted symbolic values |
| --- | ---: | --- | --- |
| `options.store_pack_flags` | `0x00` | `u16` | `NONE=0`, `EPISODE_II=1`, `EPISODE_III=2`, `EPISODE_IV=4`, `EPISODE_V=8`, `EPISODE_VI=16`, `ARCADE=32`, `BONUS=64`, `BOUNTY=128`, `CHALLENGE=256`, `JEDI=512`, `SITH=1024`; flags combine with `|`. |
| `options.touch_controls` | `0x02` | `u8` | `VIRTUAL_CONSOLE=0`, `TOUCH=1`. |
| `options.dpad_locked` | `0x03` | `u8` | `OFF=0`, `ON=1`. |
| `options.left_control_x` | `0x04` | `f32` | Left touch-control horizontal position. |
| `options.left_control_y` | `0x08` | `f32` | Left touch-control vertical position. |
| `options.right_control_x` | `0x0c` | `f32` | Right touch-control horizontal position. |
| `options.right_control_y` | `0x10` | `f32` | Right touch-control vertical position. |
| `options.music_enabled` | `0x14` | `u8` | `OFF=0`, `ON=1`. |
| `options.store_bundle_flags` | `0x15` | `u8` | `NONE=0`, `PREQUEL=1`, `ORIGINAL=2`, `COMPLETE=4`; flags combine with `|`. |
| `options.field9_0x16` | `0x16` | `bytes[2]` | Unknown trailing bytes. |

### Named wide-mask bits

`shop_character_purchased_bits` uses shop-entry order—not character IDs:

```text
 0 gonkdroid                 30 bodyguard                 60 bespinguard
 1 pkdroid                   31 grievous                  61 princessleia_prisoner
 2 battledroid               32 wookie                    62 gamorreanguard
 3 battledroid_security      33 clone_ep3                 63 bibfortuna
 4 battledroid_commander     34 clone_ep3_pilot           64 palaceguard
 5 destroyer                 35 clone_ep3_swamp           65 bossk
 6 captaintarpals            36 clone_ep3_walker          66 skiffguard
 7 bossnass                  37 disguisedclone            67 bobafett
 8 royalguard                38 macewindu_ep3             68 ewok
 9 padme                     39 anakin_jedi_scarred       69 imperialguard
10 watto                     40 rebelscum                 70 theemperor
11 pitdroid                  41 stormtrooper              71 admiralackbar
12 darthmaul                 42 imperialshuttlepilot      72 ig88
13 zamwesell                 43 tuskenraider              73 dengar
14 dexter                    44 jawa                      74 4lom
15 clone                     45 sandtrooper               75 ghostbenkenobi
16 lamasu                    46 greedo                    76 anakin_ghost
17 taunwe                    47 imperialspy               77 yoda_ghost
18 geonosian                48 beachtrooper              78 r2q5
19 battledroid_geonosian    49 deathstartrooper          79 sebulbaspod
20 superbattledroid         50 tiefighterpilot           80 tiefighter
21 jangofett                 51 imperialofficer           81 zamsspeeder
22 bobafett_boy              52 grandmofftarkin           82 droidtrifighter
23 luminara                  53 hansolo_hood              83 vulturedroid
24 kiadimundi                54 rebelhoth                 84 clonearc
25 kitfisto                  55 rebelpilot                85 tieinterceptor
26 shaakti                   56 snowtrooper               86 tiefighterdarth
27 aylasecura                57 lukeskywalker_hoth        87 tiebomber
28 plokoon                   58 lobot                     88 imperialshuttle
29 countdooku                59 ugnaught                  89 slave1
```

Bits 90–127 have no recovered shop name and use `BIT_90` through `BIT_127`.

Both `extra_unlocked_bits` and `extra_purchased_bits` use this mapping:

```text
 0 extratoggle             11 powerbrickdetector    22 superjedislam          33 fastbuild
 1 poo                     12 superslap             23 superthermaldetonator  34 scorex4
 2 disguises               13 forcezipup            24 deflectbolts           35 regenerate
 3 daisychains             14 coinmagnet            25 darkside               36 minikitdetector
 4 c3pobits                15 disarmtroopers        26 superblasters          37 scorex6
 5 towdeathstar            16 characterstuds        27 fastforce              38 superzapper
 6 silhouettes             17 perfectdeflect        28 supersabres            39 rockets
 7 beepbeep                18 explodingblasterbolts 29 tractorbeam            40 scorex8
 8 supergonk               19 forcepull             30 invincibility          41 superewokcatapult
 9 poomoney                20 vehiclesmartbomb      31 scorex2                42 infinitetorpedos
10 walkietalkiedisable     21 superastromech        32 selfdestruct           43 scorex10
```

Bits 44–63 use `BIT_n`. `hint_completion_bits` stores the same tutorial-name sequence twice:
console bit `n`, then touch bit `96 + n`.

```text
 0 hint_356                18 Tag_650              36 PlayerButton_1525
 1 GizForce_601            19 Tag_651              37 Jump_1544
 2 Lever_1548              20 ShinyMetal_643       38 Jump_1546
 3 AutoJump_1568           21 SmartBomb_1505       39 HoldTag_1526
 4 AutoJump_1569           22 GizPanel_607         40 UnlockHubStuff_1561
 5 GizBuildIts_604         23 GizPanel_608         41 UnlockHubStuff_1562
 6 Attack_631              24 GizPanel_609         42 UnlockHubStuff_1563
 7 Dodge_613               25 Shop_1516            43 Tag_649
 8 Dodge_1501              26 Shop_1515            44 VehicleStuff_1559
 9 ZipUps_610              27 Shop_1514            45 ShinyMetal_1565
10 Tag_602                 28 DragBomb_1518        46 ShinyMetal_696
11 Push_615                29 DragBomb_654         47 Teleport_1570
12 Teleport_616            30 Move_1521            48 Sith_1571
13 GizPanel_617            31 Move_1523            49 GizPanel_1572
14 GizPanel_618            32 Jump_1540            50 GizPanel_1573
15 Tag_605                 33 Move_1522            51 HatMachine_1575
16 Tag_606                 34 Jump_1542            52 HatMachine_1576
17 Tag_611                 35 PlayerButton_1524    53 Jump_1577
```

For example, tutorial 3 is `console.AutoJump_1568` at bit 3 and
`touch.AutoJump_1568` at bit 99. Console bits 54–95 and touch bits 150–191 use `BIT_n`.

### Compatibility property aliases

The editor also accepts names emitted by older versions of the original harness:

| Alias | Canonical property |
| --- | --- |
| `save_version` | `difficulty` |
| `field30_0x7c2c` | `gameplay_seconds` |
| `initial_store_pack_flags` | `suit_flags` |
| `field_0x7bf8` | `shop_gold_brick_purchased_bits` |
| `customizer.primary_name_unlocked` | `customizer.primary_use_saved_name` |
| `customizer.secondary_name_unlocked` | `customizer.secondary_use_saved_name` |

### Derived trailer

The checksum is the wrapping sum of every little-endian `u32` in the payload, starting from
`0x005c0999`:

```text
checksum = 0x005c0999 + Σ payload_u32  (mod 2^32)
```

Game `slot_code` is the 16-bit completion field interpreted as signed and sign-extended to 32
bits. For example, completion `0x8001` produces `0xffff8001`. Options use `0xffffffff`.

## Implementation notes

The crate has a conventional library/binary split. `src/lib.rs` exposes format parsing, typed
properties, transactional assignment, derived-field calculation, text views, and atomic writing.
`src/main.rs` contains only the `clap` command model, parameter-file input, terminal I/O, and
command dispatch. Consumers can therefore use `nusave` as a Rust library without pulling command-line
behavior into their application.

[`zerocopy`](https://docs.rs/zerocopy/) supplies checked, unaligned views over the original byte
buffer plus explicit little-endian integer and float wrappers. This is a better fit than decoding
into an owned model and serializing it again: edits change only the requested bytes, opaque fields
survive exactly, and layout sizes are asserted at compile time. Rust's `offset_of!` derives schema
offsets from the same structures used to access the data, reducing duplicated format arithmetic.

[`clap`](https://docs.rs/clap/) provides the complete command-line parser and generated help.
[`anstyle`](https://docs.rs/anstyle/) supplies the terminal styling without coupling the data model
to a particular renderer. [`thiserror`](https://docs.rs/thiserror/) defines structured public
errors for parsing, reading, assigning, editing, and atomic writing. Their messages identify the
bad header field, path, property, value, expected range or syntax, and underlying I/O error as
appropriate. No C/C++ library or game asset is needed at runtime.

## Development

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The integration suite exercises the interpreted summary, typed edits, derived values, wide named
masks, invalid-input rollback, game/options payloads, non-destructive output,
and lossless `list --raw` → `--params` reconstruction with opaque and non-canonical float data.

## License

MIT. See [LICENSE](LICENSE).
