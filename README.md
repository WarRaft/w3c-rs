# w3c-rs

Reader and writer for Warcraft III camera files, `war3map.w3c`: the
cameras a map's World Editor places (Camera Palette), which the map's
script names `gg_cam_<name>` (spaces as `_`). A library and a small CLI;
the [`wc3-mcp`](../wc3-mcp) server's camera tools and the
[`wc3-rs`](../wc3-rs) editor are built on it.

Reading and writing are exact: `to_bytes` gives back the bytes `parse`
read, checked on every map of the game and a corpus of custom maps (207
files, 2317 cameras).

## Format

Little-endian. `float` is an IEEE 754 single, `string` is UTF-8 ending in
a NUL.

| type   | field                                   |
|--------|-----------------------------------------|
| int    | format version: 0                       |
| int    | number of cameras, `n`                  |
|        | `n` times:                              |
| float  | target X                                |
| float  | target Y                                |
| float  | Z offset                                |
| float  | rotation (degrees)                      |
| float  | angle of attack (degrees)               |
| float  | distance                                |
| float  | roll                                    |
| float  | field of view (degrees)                 |
| float  | far Z                                   |
| float  | near Z                                  |
|        | if the game is 1.31 or later:           |
| float  | local pitch                             |
| float  | local yaw                               |
| float  | local roll                              |
|        | end if                                  |
| string | name                                    |

The version is 0 in every game, so **the file doesn't say its layout**.
Which game a map is for is in its `war3map.w3i`: the first four bytes are
that file's format version, 18 for Reign of Chaos, 25 for The Frozen
Throne, 28 for 1.31, 31 for 1.32. 28 and later means the local rotation
is there (`Layout::from_w3i`, `Layout::from_w3i_version`).

Without a `war3map.w3i`, the layout is 1.27a's (`Layout::default()`, the
older one). `parse_any` tries it first, then the newer: a file in the
wrong layout fails, because every camera must fit and every byte must be
read, though a file with no cameras fits both.

World Editor's defaults for a new camera (`Camera::new`), the same as
`bj_CAMERA_DEFAULT_*` in `Blizzard.j`: rotation 90, angle of attack 304,
distance 1650, roll 0, field of view 70, far Z 5000; near Z 100.

## Library

```rust
use w3c_rs::{Layout, parse};

let bytes = std::fs::read("war3map.w3c")?;
let layout = std::fs::read("war3map.w3i").ok()
    .and_then(|w3i| Layout::from_w3i(&w3i))
    .unwrap_or_default();
let mut file = parse(&bytes, layout)?;
file.cameras[0].far_z = 3000.0;
std::fs::write("war3map.w3c", file.to_bytes()?)?;
```

- `Cameras { version, layout, cameras }`, `Camera` with every field and
  its `name`; `fields()` gives the numbers in file order with their
  names, `field_mut(name)` one of them.
- Errors are `parser_rs::BinError`, with the byte offset.

## Text form

For people to read and edit (`text::to_text`, `text::from_text`): BNI, a
`[w3c]` section, then each camera as a section named after it, its fields
in file order. The [`wc3-rs`](../wc3-rs) editor opens `.w3c` files this
way.

```ini
[w3c]
version=0

[Close Arthas]
target_x=-2728.98
target_y=1922.12
z_offset=0
rotation=83.74
angle_of_attack=330.62
distance=830.11
roll=0
field_of_view=90
far_z=2500
near_z=100
```

- The text doesn't say its layout any more than the file does: it's read
  in the layout the caller gives, and `local_pitch`, `local_yaw`,
  `local_roll` are only in the newer one.
- Numbers are the shortest decimal that reads back as the same `f32`
  (`-0` stays negative), so the text gives back the exact bytes (the
  corpus test checks it).
- A field left out takes World Editor's default; cameras keep their
  order. A name with `]` or a line break can't be a section header, so
  such a file has no text form.
- Errors (`from_text`'s `errors`, with line and column): an unknown or
  repeated field, a value that isn't a number, a field the layout doesn't
  have, a version other than 0, anything before `[w3c]`.

## CLI

```sh
w3c-rs war3map.w3c [war3map.w3i]          # read and write back: a check
w3c-rs --list war3map.w3c [war3map.w3i]   # the cameras, tab-separated
```

## Tests

```sh
cargo test
W3C_CORPUS=<folder of map folders> cargo test --release corpus -- --ignored
```

The corpus test reads each folder's `war3map.w3c` with its `war3map.w3i`
(when there is one), writes it back and compares the bytes, and checks
that reading without the `war3map.w3i` gives the same.
