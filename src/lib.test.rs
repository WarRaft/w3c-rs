use super::*;

/// A file of two cameras, in `layout`.
fn file(layout: Layout) -> Vec<u8> {
    let mut w = Writer::new();
    w.u32(0);
    w.u32(2);
    for (i, name) in ["Camera 001", "Intro"].iter().enumerate() {
        for v in [
            100.0 * i as f32,
            -256.5,
            0.0,
            90.0,
            304.0,
            1650.0,
            0.0,
            70.0,
            5000.0,
            100.0,
        ] {
            w.f32(v);
        }
        if layout == Layout::Local {
            w.f32(1.0);
            w.f32(2.0);
            w.f32(3.0);
        }
        w.cstr(name).unwrap();
    }
    w.into_bytes()
}

#[test]
fn reads_both_layouts_and_writes_them_back() {
    let classic = file(Layout::Classic);
    let cams = parse(&classic, Layout::Classic).unwrap();
    assert_eq!(cams.cameras.len(), 2);
    let c = &cams.cameras[1];
    assert_eq!(
        (
            c.name.as_str(),
            c.target_x,
            c.target_y,
            c.distance,
            c.local_yaw
        ),
        ("Intro", 100.0, -256.5, 1650.0, 0.0)
    );
    assert_eq!(cams.to_bytes().unwrap(), classic);

    let local = file(Layout::Local);
    let cams = parse(&local, Layout::Local).unwrap();
    assert_eq!(
        (cams.cameras[0].local_pitch, cams.cameras[0].local_roll),
        (1.0, 3.0)
    );
    assert_eq!(cams.to_bytes().unwrap(), local);
}

#[test]
fn the_wrong_layout_fails_and_any_finds_the_right_one() {
    let local = file(Layout::Local);
    assert!(parse(&local, Layout::Classic).is_err());
    assert!(parse(&file(Layout::Classic), Layout::Local).is_err());
    assert_eq!(parse_any(&local).unwrap().layout, Layout::Local);
    assert_eq!(
        parse_any(&file(Layout::Classic)).unwrap().layout,
        Layout::Classic
    );
    // An empty file fits both: the older layout, as 1.27a writes it.
    assert_eq!(parse_any(&[0; 8]).unwrap().layout, Layout::Classic);
}

#[test]
fn layout_from_w3i() {
    assert_eq!(
        Layout::from_w3i(&18u32.to_le_bytes()),
        Some(Layout::Classic)
    );
    assert_eq!(
        Layout::from_w3i(&25u32.to_le_bytes()),
        Some(Layout::Classic)
    );
    assert_eq!(Layout::from_w3i(&[28, 0, 0, 0, 9]), Some(Layout::Local));
    assert_eq!(Layout::from_w3i(&31u32.to_le_bytes()), Some(Layout::Local));
    assert_eq!(Layout::from_w3i(&[1, 2]), None);
    assert_eq!(Layout::default(), Layout::Classic);
}

#[test]
fn errors_have_offsets() {
    let mut bytes = file(Layout::Classic);
    bytes[0] = 7;
    assert_eq!(
        parse(&bytes, Layout::Classic).unwrap_err().to_string(),
        "unknown w3c version 7 (at byte 0x0)"
    );
    let bytes = file(Layout::Classic);
    assert!(parse(&bytes[..bytes.len() - 3], Layout::Classic).is_err());
    let mut more = bytes.clone();
    more.push(0);
    assert_eq!(
        parse(&more, Layout::Classic).unwrap_err().message,
        "1 extra byte after the end of the data"
    );
}

#[test]
fn new_cameras_and_fields() {
    let mut cams = Cameras::new(Layout::Classic);
    cams.cameras.push(Camera::new("Boss", 512.0, -1024.0));
    *cams.cameras[0].field_mut("far_z").unwrap() = 3000.0;
    assert!(cams.cameras[0].field_mut("nope").is_none());
    let back = parse(&cams.to_bytes().unwrap(), Layout::Classic).unwrap();
    assert_eq!(back, cams);
    assert_eq!(back.cameras[0].fields()[8], ("far_z", 3000.0));
    // A name can't hold NUL.
    cams.cameras[0].name = "a\0b".into();
    assert!(cams.to_bytes().is_err());
}

/// Every map folder under `$W3C_CORPUS` (a `war3map.w3c`, and its
/// `war3map.w3i` if it has one) reads and writes back byte for byte:
/// `W3C_CORPUS=dir cargo test corpus -- --ignored`.
#[test]
#[ignore]
fn corpus() {
    let dir = std::env::var("W3C_CORPUS").expect("set W3C_CORPUS to a folder of map folders");
    let mut checked = 0;
    let mut cameras = 0;
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        let Ok(bytes) = std::fs::read(path.join("war3map.w3c")) else {
            continue;
        };
        let layout = std::fs::read(path.join("war3map.w3i"))
            .ok()
            .and_then(|w| Layout::from_w3i(&w));
        let file = match layout {
            Some(layout) => parse(&bytes, layout),
            None => parse_any(&bytes),
        }
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            file.to_bytes().unwrap() == bytes,
            "{}: differs after writing",
            path.display()
        );
        // And through the text form.
        let text = text::to_text(&file).unwrap();
        let back = text::from_text(&text, file.layout);
        assert!(
            back.errors.is_empty(),
            "{}: {:?}",
            path.display(),
            back.errors
        );
        assert!(
            back.file.unwrap().to_bytes().unwrap() == bytes,
            "{}: differs after the text form",
            path.display()
        );
        // Without the w3i, the default reading gives the same.
        assert_eq!(parse_any(&bytes).unwrap(), file, "{}", path.display());
        checked += 1;
        cameras += file.cameras.len();
    }
    assert!(checked > 0, "no war3map.w3c under {dir}");
    eprintln!("{checked} files, {cameras} cameras");
}
