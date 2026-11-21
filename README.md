# w3c-rs

Reader and writer for Warcraft III camera files, `war3map.w3c`: the
cameras a map's World Editor places (Camera Palette), which the map's
script names `gg_cam_<name>` (spaces as `_`). A library and a small CLI;
the [`wc3-mcp`](../wc3-mcp) server's camera tools are built on it.

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
