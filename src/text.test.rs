use super::*;

fn two(layout: Layout) -> Cameras {
    let mut file = Cameras::new(layout);
    file.cameras
        .push(Camera::new("Close Arthas", -2728.98, 1922.12));
    let mut c = Camera::new(" Spaced  name ", 0.1, -0.0);
    c.local_yaw = if layout == Layout::Local { 15.5 } else { 0.0 };
    c.far_z = f32::from_bits(0x4000_0001); // a float with no short decimal
    file.cameras.push(c);
    file
}

#[test]
fn text_reads_back_to_the_same_bytes() {
    for layout in [Layout::Classic, Layout::Local] {
        let file = two(layout);
        let text = to_text(&file).unwrap();
        let back = from_text(&text, layout);
        assert_eq!(back.errors, vec![], "{text}");
        assert_eq!(
            back.file.unwrap().to_bytes().unwrap(),
            file.to_bytes().unwrap()
        );
        assert_eq!(text.contains("local_yaw="), layout == Layout::Local);
    }
    let text = to_text(&two(Layout::Classic)).unwrap();
    assert!(text.starts_with("[w3c]\nversion=0\n\n[Close Arthas]\ntarget_x=-2728.98\ntarget_y=1922.12\nz_offset=0\nrotation=90\n"), "{text}");
    assert!(
        text.contains("[ Spaced  name ]\ntarget_x=0.1\ntarget_y=-0\n"),
        "{text}"
    );
    assert!(is_camera_text(&bni_rs::parse(&text)));
}

#[test]
fn left_out_fields_take_the_defaults() {
    let file = from_text("[w3c]\n[Boss]\ntarget_x=5\n", Layout::Classic)
        .file
        .unwrap();
    assert_eq!(file.cameras, vec![Camera::new("Boss", 5.0, 0.0)]);
    assert_eq!(
        from_text("[w3c]\n", Layout::Local).file.unwrap().cameras,
        vec![]
    );
}

#[test]
fn errors_point_at_the_field() {
    let text = "[w3c]\nversion=3\nkind=x\n[A]\ntarget_x=abc\nnope=1\nlocal_yaw=2\nroll=1\nroll=2\n";
    let errors = from_text(text, Layout::Classic).errors;
    let got: Vec<(usize, &str)> = errors
        .iter()
        .map(|e| (e.line, e.message.as_str()))
        .collect();
    assert_eq!(got.len(), 6, "{got:?}");
    assert_eq!(got[0], (2, "the version is 0, the only one there is"));
    assert_eq!(got[1], (3, "[w3c] has only `version`"));
    assert_eq!(got[2], (5, "`target_x` is a number"));
    assert!(got[3].1.starts_with("no field `nope`"));
    assert!(got[4].1.contains("1.31 and later"));
    assert_eq!(got[5], (9, "`roll` is set twice"));
    assert_eq!((errors[2].col, errors[2].end_col), (1, 13));
    // In the newer layout the local rotation is fine.
    assert_eq!(
        from_text("[w3c]\n[A]\nlocal_yaw=2\n", Layout::Local).errors,
        vec![]
    );
}

#[test]
fn not_camera_text() {
    let parsed = from_text("x=1\n[A]\n", Layout::Classic);
    assert!(parsed.file.is_none());
    assert_eq!(parsed.errors.len(), 2);
    assert!(!is_camera_text(&bni_rs::parse("[w3u]\n")));
    let mut file = Cameras::new(Layout::Classic);
    file.cameras.push(Camera::new("a]b", 0.0, 0.0));
    assert!(to_text(&file).is_err());
}
