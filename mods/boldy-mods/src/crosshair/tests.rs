use super::*;

fn look(style: u8) -> Crosshair {
    Crosshair {
        on: true,
        style,
        ..DEFAULT
    }
}

/// A drawing of grid `size` from rows given as bits.
fn drawn(size: u8, rows: &[u64]) -> Drawing {
    let mut out = Drawing::empty(size);
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..64 {
            if bits >> col & 1 == 1 {
                out = out.painted(row, col, true, false);
            }
        }
    }
    out
}

/// Whether `point` is filled by the even-odd rule over `paths`.
fn filled(paths: &[Vec<[f32; 2]>], [px, py]: [f32; 2]) -> bool {
    let mut inside = false;
    for path in paths {
        let n = path.len();
        for i in 0..n {
            let ([x0, y0], [x1, y1]) = (path[i], path[(i + 1) % n]);
            if (y0 > py) != (y1 > py) && px < x0 + (py - y0) / (y1 - y0) * (x1 - x0) {
                inside = !inside;
            }
        }
    }
    inside
}

/// Whether `shape` covers `point`, turned back for an X.
fn covers(shape: &Shape, point: [f32; 2]) -> bool {
    let [mut x, mut y] = point;
    if shape.turned {
        let (m, turn) = (shape.side / 2.0, std::f32::consts::FRAC_1_SQRT_2);
        let (dx, dy) = (x - m, y - m);
        (x, y) = (m + (dx + dy) * turn, m + (dy - dx) * turn);
    }
    shape
        .rects
        .iter()
        .any(|&[x0, y0, x1, y1]| x >= x0 && x < x1 && y >= y0 && y < y1)
        || filled(&shape.circles, [x, y])
}

/// The pixel centres the shape covers, as offsets from the crosshair's middle pixel.
fn pixels(shape: &Shape) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let side = shape.side as i32;
    let middle = side / 2;
    for y in 0..side {
        for x in 0..side {
            if covers(shape, [x as f32 + 0.5, y as f32 + 0.5]) {
                out.push((x - middle, y - middle));
            }
        }
    }
    out
}

#[test]
fn a_cross_covers_whole_pixels_with_a_gap_in_the_middle() {
    // Scale 1: thickness 1, gap 2 and length 4 pixels.
    let shape = shape(&look(0), 1.0);
    let covered = pixels(&shape);
    // Four arms of four pixels each, one pixel thick, after two empty pixels.
    assert_eq!(covered.len(), 16, "{covered:?}");
    for d in 3..7 {
        for p in [(d, 0), (-d, 0), (0, d), (0, -d)] {
            assert!(covered.contains(&p), "{p:?} in {covered:?}");
        }
    }
    assert!(!covered.contains(&(0, 0)) && !covered.contains(&(2, 0)));
    assert_eq!(shape.side as u32 % 2, 0);
    // Every edge is on a whole pixel.
    for rect in &shape.rects {
        assert_eq!(rect.map(f32::fract), [0.0; 4], "{rect:?}");
    }
}

#[test]
fn arms_that_meet_make_one_solid_plus() {
    let solid = Crosshair {
        gap: 0.0,
        thickness: 1.0,
        length: 3.0,
        ..look(0)
    };
    let shape = shape(&solid, 1.0);
    let covered = pixels(&shape);
    assert!(covered.contains(&(0, 0)));
    // Three pixels out on every side, plus the middle: 13.
    assert_eq!(covered.len(), 13, "{covered:?}");
}

#[test]
fn a_t_has_no_top_arm_and_a_dot_sits_in_the_gap() {
    let t = pixels(&shape(&look(T_SHAPE), 1.0));
    assert_eq!(t.len(), 12);
    assert!(t.iter().all(|&(_, y)| y >= 0), "{t:?}");
    let dotted = pixels(&shape(&look(CROSS_DOT), 1.0));
    assert_eq!(dotted.len(), 17);
    assert!(dotted.contains(&(0, 0)));
}

#[test]
fn circles_are_rings_with_room_round_the_dot() {
    // Scale 2: thickness 2, so the ring runs from 8 to 10 pixels out and the dot is 4 wide.
    let ring = pixels(&shape(&look(CIRCLE), 2.0));
    assert!(!ring.contains(&(0, 0)), "a ring is empty in the middle");
    let dotted = pixels(&shape(&look(CIRCLE_DOT), 2.0));
    assert!(dotted.contains(&(0, 0)), "the dot fills the middle");
    for (near, far) in [
        ((4, 0), (8, 0)),
        ((-5, 0), (-9, 0)),
        ((0, 4), (0, 8)),
        ((0, -5), (0, -9)),
    ] {
        assert!(
            !dotted.contains(&near),
            "free between dot and ring at {near:?}"
        );
        assert!(
            dotted.contains(&far) && ring.contains(&far),
            "ring at {far:?}"
        );
    }
    let dot = pixels(&shape(&look(DOT), 2.0));
    assert!(dot.contains(&(0, 0)) && dot.len() >= 4 && !dot.contains(&(4, 0)));
}

#[test]
fn an_x_is_a_cross_turned_and_everything_fits_its_box() {
    let x = shape(&look(X), 2.0);
    let covered = pixels(&x);
    assert!(
        covered.iter().any(|&(a, b)| a > 3 && b > 3),
        "diagonal arms"
    );
    assert!(
        !covered.iter().any(|&(a, b)| b == 0 && a > 4),
        "no straight arms"
    );
    assert!(
        x.turned
            && patch(Some(&View {
                look: look(X),
                viewport: [400.0, 300.0, 2.0]
            }))
            .contains(r#","rotation":45,"origin":[0.5,0.5]}"#)
    );
    let full = Drawing::empty(64).resized(64);
    let mut big_drawing = full;
    for row in 0..64 {
        big_drawing = big_drawing.painted(row, row, true, true);
    }
    for style in 0..STYLES.len() as u8 {
        for scale in [1.0, 2.0, 3.0, 4.5] {
            let big = Crosshair {
                length: LENGTH.max,
                thickness: THICKNESS.max,
                gap: GAP.max,
                pixels: big_drawing,
                pixel_size: MAX_PIXEL_SIZE,
                ..look(style)
            };
            for look in [look(style), big] {
                let shape = shape(&look, scale);
                // The outline reaches `stroke` further out, and has to fit too.
                let name = STYLES[usize::from(style)];
                let inside = |v: f32| v >= shape.stroke && v <= shape.side - shape.stroke;
                for point in shape.circles.iter().flatten() {
                    assert!(
                        point.iter().all(|v| inside(*v)),
                        "{name} at {scale}: {point:?}"
                    );
                }
                for rect in &shape.rects {
                    assert!(
                        rect.iter().all(|v| inside(*v)),
                        "{name} at {scale}: {rect:?}"
                    );
                    assert!(rect[2] > rect[0] && rect[3] > rect[1], "{rect:?}");
                }
                // Cinnabar draws nothing of a wider path, outline included.
                for point in shape.circles.iter().flatten() {
                    for v in point {
                        let from_middle = (v - shape.centre).abs() + shape.stroke;
                        assert!(from_middle * 2.0 <= MAX_PATH, "{name} at {scale}: {v}");
                    }
                }
                assert!(shape.circles.iter().all(|path| path.len() >= 3));
            }
        }
    }
}

#[test]
fn values_step_and_clamp() {
    assert_eq!(LENGTH.stepped(4.0, 1.0), 4.5);
    assert_eq!(LENGTH.stepped(LENGTH.max, 1.0), LENGTH.max);
    assert_eq!(GAP.stepped(0.0, -1.0), 0.0);
    assert_eq!(THICKNESS.clamp(f32::NAN), THICKNESS.min);
    assert_eq!(THICKNESS.clamp(1.26), 1.5);
    assert_eq!(step_opacity(100, true), 100);
    assert_eq!(step_opacity(100, false), 90);
    assert_eq!(step_opacity(MIN_OPACITY, false), MIN_OPACITY);
    assert_eq!(clamp_opacity(47.0), 50);
    assert_eq!(step_style(0, false), STYLES.len() as u8 - 1);
    assert_eq!(step_style(STYLES.len() as u8 - 1, true), 0);
    for style in 0..STYLES.len() as u8 {
        assert_eq!(style_from_key(style_key(style)), Some(style));
    }
    assert_eq!(style_from_key(" X "), Some(X));
    assert_eq!(style_from_key("star"), None);
    assert_eq!(step_resolution(16, true), 32);
    assert_eq!(step_resolution(64, true), 64);
    assert_eq!(step_resolution(32, false), 16);
    assert_eq!(step_resolution(16, false), 16);
    assert_eq!(clamp_pixel_size(0.0), MIN_PIXEL_SIZE);
    assert_eq!(clamp_pixel_size(99.0), MAX_PIXEL_SIZE);
    assert_eq!(clamp_pixel_size(f32::NAN), DEFAULT_PIXEL_SIZE);
    assert_eq!(Drawing::empty(7).size(), 16);
}

/// The number after `"name":` (or after `"name":[` and `skip` commas) in a patch.
fn number(patch: &str, name: &str, skip: usize) -> f32 {
    let at = patch.find(&format!("\"{name}\":")).expect(name) + name.len() + 3;
    let rest = patch[at..].trim_start_matches('[');
    let item = rest.split(',').nth(skip).unwrap();
    item.trim_end_matches([']', '}']).parse().expect(item)
}

fn balanced(json: &str) {
    for line in json.lines() {
        for (open, close) in [('{', '}'), ('[', ']')] {
            assert_eq!(
                line.matches(open).count(),
                line.matches(close).count(),
                "{line}"
            );
        }
        assert!(!line.contains(",]") && !line.contains(",}") && !line.contains("NaN"));
    }
}

/// Every `{"offset":[x,y],"size":[w,h]}` piece of `template` in a patch, as `[x, y, w, h]`.
fn pieces(patch: &str, template: &str) -> Vec<[f32; 4]> {
    let marker = format!("@boldy_hud.{template}\":{{\"offset\":[");
    patch
        .match_indices(&marker)
        .map(|(at, _)| {
            let rest = &patch[at + marker.len()..];
            let numbers: Vec<f32> = rest[..rest.find("]}").unwrap()]
                .replace("],\"size\":[", ",")
                .split(',')
                .map(|n| n.parse().unwrap())
                .collect();
            [numbers[0], numbers[1], numbers[2], numbers[3]]
        })
        .collect()
}

#[test]
fn patches_define_the_crosshair_on_whole_pixels_and_colour_it() {
    let view = View {
        look: look(CROSS_DOT),
        viewport: [427.0, 240.5, 3.0],
    };
    let shown = patch(Some(&view));
    balanced(&shown);
    assert!(
        shown.starts_with(r##"{"controls":{"xpart0":"##),
        "parts first"
    );
    let header = shown.lines().last().unwrap();
    assert!(header.starts_with(r##"{"controls":{"xhair":{"type":"panel","size":["##));
    assert!(shown.contains(r##""binding_changes":{"#crosshair_visible":true,"##));
    assert!(
        shown.contains(r#"{"p8@boldy_hud.xpart8":{}}"#),
        "every part is placed"
    );
    assert!(
        shown.contains(r#""cinnabar_transform":{"scale":0.3333333333333333,"origin":[0.5,0.5]}"#),
        "{shown}"
    );
    let size = number(header, "size", 0);
    // The box's top-left corner lands on a whole pixel.
    for (axis, length) in [427.0f32, 240.5].into_iter().enumerate() {
        let offset = number(header, "offset", axis);
        let corner = (length - size) / 2.0 * 3.0 + offset * 3.0;
        assert!((corner - corner.round()).abs() < 0.02, "{corner}");
    }
    // Four arms and the dot, each over a black piece larger all round by the outline.
    let fills = pieces(&shown, "xf");
    let edges = pieces(&shown, "xe");
    assert_eq!((fills.len(), edges.len()), (5, 5));
    let pixel = view_shape(&view).stroke;
    for (fill, edge) in fills.iter().zip(&edges) {
        assert!(
            (edge[0] - (fill[0] - pixel)).abs() < 0.01,
            "{fill:?} {edge:?}"
        );
        assert!(
            (edge[2] - (fill[2] + 2.0 * pixel)).abs() < 0.01,
            "{fill:?} {edge:?}"
        );
        // On whole pixels: the pieces' panel is scaled down by the GUI scale, 3.
        for v in fill {
            assert_eq!(v.fract(), 0.0, "{fill:?}");
        }
    }
    assert!(!shown.contains("vector_path"), "no circles in a cross");
    // A ring and its dot are paths, outline under fill, the same points in both.
    let ring = patch(Some(&View {
        look: look(CIRCLE_DOT),
        viewport: [427.0, 240.5, 3.0],
    }));
    balanced(&ring);
    let edge = ring.find(r#""ring_edge":"#).unwrap();
    let fill = ring.find(r#""ring_fill":"#).unwrap();
    assert!(edge < fill);
    let paths = |from: usize| {
        let at = ring[from..].find(r#""paths":"#).unwrap() + from;
        &ring[at..at + ring[at..].find("]]]").unwrap()]
    };
    assert_eq!(paths(edge), paths(fill));
    assert_eq!(paths(fill).matches("[[").count(), 3);
    // Colours are bound; without an outline there are no black pieces.
    let plain = patch(Some(&View {
        look: Crosshair {
            outline: false,
            opacity: 50,
            rgb: [255, 0, 0],
            ..look(0)
        },
        viewport: [400.0, 300.0, 2.0],
    }));
    balanced(&plain);
    assert!(pieces(&plain, "xe").is_empty(), "no outline: {plain}");
    assert!(
        plain.contains(r##""#crosshair_color":[1,0,0,0.5]"##),
        "{plain}"
    );
    assert_eq!(
        patch(None),
        r##"{"binding_changes":{"#crosshair_visible":false}}"##
    );
    assert_eq!(
        visibility(true),
        r##"{"binding_changes":{"#crosshair_visible":true}}"##
    );
    let mut text = String::new();
    contours(
        &mut text,
        &[
            vec![[0.0, 3.0], [12.25, 107.04]],
            vec![[f32::NAN, -1.0], [0.95, 9.96]],
        ],
    );
    assert_eq!(text, "[[[0,3],[12.3,107]],[[0,0],[1,10]]]");
    text.clear();
    contours(&mut text, &[vec![[123456.0, 9999.94]]]);
    assert_eq!(text, "[[[9999.9,9999.9]]]");
}

#[test]
fn the_default_drawing_is_a_centred_cross() {
    let custom = shape(&look(CUSTOM), 2.0);
    // Four arms, four rectangles, each on whole pixels.
    assert_eq!(rects(&DEFAULT_DRAWING).len(), 4);
    for rect in &custom.rects {
        assert_eq!(rect.map(f32::fract), [0.0; 4], "{rect:?}");
    }
    // Centred: the same on both sides of the middle, along both axes.
    let covered = pixels(&custom);
    assert!(!covered.is_empty());
    for &(x, y) in &covered {
        assert!(
            covered.contains(&(-1 - x, y)) && covered.contains(&(x, -1 - y)),
            "{x},{y}"
        );
    }
    assert!(!covered.contains(&(0, 0)), "an empty middle");
}

#[test]
fn drawings_paint_mirror_merge_resize_and_round_trip() {
    for size in RESOLUTIONS {
        let last = usize::from(size) - 1;
        let drawing = Drawing::empty(size).painted(0, 0, true, true);
        for (row, col) in [(0, 0), (0, last), (last, 0), (last, last)] {
            assert!(drawing.pixel(row, col), "{size}: {row},{col}");
        }
        let one = drawing.painted(0, last, false, false);
        assert!(!one.pixel(0, last) && one.pixel(0, 0));
        assert_eq!(one.painted(last + 1, 3, true, true), one, "off the grid");
        assert_eq!(Drawing::parse(&one.text(), size), one);
    }
    // A filled block of rows with the same run is one rectangle; a ring is four.
    let block = drawn(16, &[0, 0, 0, 0b1110, 0b1110, 0b1110, 0b1110]);
    assert_eq!(*rects(&block), vec![[1, 3, 4, 7]]);
    let ring = drawn(
        16,
        &[0, 0, 0b1111100, 0b1000100, 0b1000100, 0b1000100, 0b1111100],
    );
    assert_eq!(rects(&ring).len(), 4);
    // Pixels drawn are exactly the rectangles' cells.
    let covered: usize = rects(&ring)
        .iter()
        .map(|[c0, r0, c1, r1]| usize::from(c1 - c0) * usize::from(r1 - r0))
        .sum();
    assert_eq!(covered, 16);
    // A run at the grid's far edge, at every size.
    let edge = drawn(64, &[u64::MAX, u64::MAX]);
    assert_eq!(*rects(&edge), vec![[0, 0, 64, 2]]);
    // The older form, rows of `.` and `#`, is read into the smallest grid.
    assert_eq!(
        Drawing::parse("#.x/..1/junk", 16),
        drawn(16, &[0b101, 0b100])
    );
    assert_eq!(Drawing::parse("7c3e", 16), drawn(16, &[0x7C3E]));
    // Eight digits at a time: every byte value, alone among digits, and the digits' order.
    for byte in 0..=u8::MAX {
        let mut word = *b"0123abcd";
        for place in 0..8 {
            word[place] = byte;
            let spelled = u64::from_le_bytes(word);
            assert_eq!(
                hex8(spelled).is_some(),
                byte.is_ascii_hexdigit(),
                "{byte:#x}"
            );
            word = *b"0123abcd";
        }
    }
    assert_eq!(hex8(u64::from_le_bytes(*b"89abCDEF")), Some(0x89AB_CDEF));
    assert_eq!(hex8(u64::from_le_bytes(*b"00000001")), Some(1));
    assert_eq!(hex8(u64::from_le_bytes(*b"10000000")), Some(0x1000_0000));
    // The one-pass reader and the general one read the same text the same way.
    for (text, size) in [
        ("", 16),
        ("7c3e/0/FFFF/ffff/1/2", 16),
        ("7c3e/ 7c3e", 16),
        ("fffffffffffffffff/1", 16),
        ("1fffffffffffffff/2", 16),
        ("12345678/1/", 32),
        ("////", 16),
        ("1/2/3/4/5/6/7/8/9/a/b/c/d/e/f/10/11/12", 16),
        ("-1/+2/0x3/4 /g", 16),
        (
            "0123456789abcdef/FEDCBA9876543210/00000000ffffffff/0123456789abcdeg",
            64,
        ),
        ("0123456789abcdef0/1", 64),
        ("0123456789abcdef/", 64),
    ] {
        assert_eq!(
            Drawing::parse(text, size),
            Drawing::parse_general(text, size),
            "{text}"
        );
    }
    // The README's example is the default drawing, written as the settings file writes it.
    assert_eq!(
        DEFAULT_DRAWING.text(),
        "0000/0180/0180/0180/0180/0180/0000/7c3e/7c3e/0000/0180/0180/0180/0180/0180/0000"
    );
    // Resizing keeps the shape: up and back down is the same drawing.
    for size in [32, 64] {
        let bigger = DEFAULT_DRAWING.resized(size);
        assert_eq!(bigger.size(), usize::from(size));
        assert_eq!(rects(&bigger).len(), 4);
        assert_eq!(bigger.resized(16), DEFAULT_DRAWING);
    }
    let lone = drawn(64, &[0, 0, 0, 1 << 5]);
    assert_eq!(lone.resized(16), drawn(16, &[1 << 1]));
    // A pixel size follows, so the crosshair keeps its size on screen.
    let finer = with_resolution(&look(CUSTOM), 32);
    assert_eq!((finer.pixels.size(), finer.pixel_size), (32, 1));
    // A checkerboard is far too detailed; a hand-edited one is drawn only in part.
    let checks = drawn(
        64,
        &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA].repeat(32),
    );
    assert!(too_detailed(&checks) && !too_detailed(&DEFAULT_DRAWING));
    let busy = Crosshair {
        pixels: checks,
        ..look(CUSTOM)
    };
    assert_eq!(shape(&busy, 2.0).rects.len(), MAX_RECTS);
    // Stripes cost little (a row like the one above is free); stripes changing every row
    // are too many runs to work out in a frame, though few rectangles.
    let stripes = drawn(64, &[0x5555_5555_5555_5555; 64]);
    assert_eq!(runs(&stripes), 32);
    assert!(!too_detailed(&stripes) && rects(&stripes).len() == 32);
    let shifting: Vec<u64> = (0..64)
        .map(|row| 0x5555_5555_5555_5555u64 ^ (1 << (2 * (row % 32))))
        .collect();
    let shifting = drawn(64, &shifting);
    assert!(runs(&shifting) > MAX_RUNS && too_detailed(&shifting));
    // Estimated costs grow with the work, and a known drawing costs nothing more.
    let _ = rects(&DEFAULT_DRAWING);
    assert_eq!(rects_cost(&DEFAULT_DRAWING), 0);
    assert!(rects_cost(&shifting) > rects_cost(&stripes));
}

/// Resizing pixel by pixel, as it is meant to work, to check the fast way against.
fn resized_slowly(drawing: &Drawing, size: u8) -> Drawing {
    let mut out = Drawing::empty(size);
    let (from, to) = (drawing.size(), out.size());
    for row in 0..to {
        for col in 0..to {
            let drawn = if to >= from {
                let f = to / from;
                drawing.pixel(row / f, col / f)
            } else {
                let f = from / to;
                (0..f).any(|r| (0..f).any(|c| drawing.pixel(row * f + r, col * f + c)))
            };
            if drawn {
                out = out.painted(row, col, true, false);
            }
        }
    }
    out
}

#[test]
fn resizing_row_by_row_is_resizing_pixel_by_pixel() {
    // Drawings with pixels everywhere, edges and corners included, at every size.
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    for from in RESOLUTIONS {
        for _ in 0..6 {
            let mut drawing = Drawing::empty(from);
            for row in 0..usize::from(from) {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                for col in 0..usize::from(from) {
                    if seed >> col & 1 == 1 {
                        drawing = drawing.painted(row, col, true, false);
                    }
                }
            }
            for to in RESOLUTIONS {
                assert_eq!(
                    drawing.resized(to),
                    resized_slowly(&drawing, to),
                    "{from} to {to}"
                );
            }
            assert_eq!(Drawing::parse(&drawing.text(), from), drawing);
        }
    }
    assert_eq!(Drawing::empty(64).text().len(), 64 * 17 - 1);
}

#[test]
fn a_fast_stroke_leaves_no_gaps() {
    let line = Drawing::empty(64).painted_line((2, 3), (9, 40), true, false);
    // One pixel in every column it crosses, and its ends.
    for col in 3..=40 {
        assert!((0..64).any(|row| line.pixel(row, col)), "column {col}");
    }
    assert!(line.pixel(2, 3) && line.pixel(9, 40));
    let steep = Drawing::empty(16).painted_line((15, 1), (0, 4), true, true);
    for row in 0..16 {
        assert!((0..16).any(|col| steep.pixel(row, col)), "row {row}");
    }
    // Mirrored like a single pixel, and a line of one pixel is that pixel.
    assert!(steep.pixel(15, 14) && steep.pixel(0, 1));
    assert_eq!(
        Drawing::empty(32).painted_line((5, 6), (5, 6), true, false),
        Drawing::empty(32).painted(5, 6, true, false)
    );
}

#[test]
fn large_drawings_are_sent_in_parts_and_only_what_changed() {
    // Forty separate pixels: three parts.
    let mut drawing = Drawing::empty(64);
    for index in 0..40 {
        drawing = drawing.painted(index, (index * 3) % 64, true, false);
    }
    let view = View {
        look: Crosshair {
            pixels: drawing,
            ..look(CUSTOM)
        },
        viewport: [400.0, 300.0, 2.0],
    };
    let shape = view_shape(&view);
    assert_eq!(shape.rects.len(), 40);
    let plan = plan(&view, &shape);
    assert!(plan.parts[..3].iter().all(|part| *part != empty_part()));
    assert!(plan.parts[3..].iter().all(|part| *part == empty_part()));
    for index in 0..PARTS {
        let patch = part_patch(&view, &shape, index);
        balanced(&patch);
        let expected = part(&shape.rects, index).len();
        assert_eq!(pieces(&patch, "xf").len(), expected, "part {index}");
        assert!(part_cost(&shape, index) >= 20_000);
    }
    // A pixel more changes the part it falls in, and not the header.
    let more = View {
        look: Crosshair {
            pixels: drawing.painted(63, 0, true, false),
            ..view.look
        },
        ..view
    };
    let more_plan = plan_of(&more);
    assert_eq!(more_plan.header, plan.header);
    assert_ne!(more_plan.parts, plan.parts);
    // A colour changes the header only.
    let red = View {
        look: Crosshair {
            rgb: [255, 0, 0],
            ..view.look
        },
        ..view
    };
    let red_plan = plan_of(&red);
    assert_ne!(red_plan.header, plan.header);
    assert_eq!(red_plan.parts, plan.parts);
}

fn plan_of(view: &View) -> Plan {
    plan(view, &view_shape(view))
}

#[test]
fn an_empty_drawing_is_an_empty_box_and_the_grid_shows_the_pixels() {
    let empty = View {
        look: Crosshair {
            pixels: Drawing::empty(16),
            ..look(CUSTOM)
        },
        viewport: [400.0, 300.0, 2.0],
    };
    let shown = patch(Some(&empty));
    balanced(&shown);
    assert!(pieces(&shown, "xf").is_empty(), "{shown}");
    assert!(
        plan_of(&empty)
            .parts
            .iter()
            .all(|part| *part == empty_part())
    );
    // The grid's cells: four arms, in 12-unit squares at 16, in the drawing's colour.
    let grid = grid_patch(&DEFAULT_DRAWING, [255, 0, 0]);
    balanced(&grid);
    assert!(grid.starts_with(r##"{"controls":{"gridlines":{"##));
    assert!(grid.contains("@boldy_hud.lines16"));
    let cells = pieces(&grid, "gp");
    assert_eq!(cells.len(), 4, "{grid}");
    assert!(
        cells.contains(&[14.0, 98.0, 70.0, 28.0]),
        "left arm: {cells:?}"
    );
    assert!(grid.contains(r##""#drawgrid_color":[1,0,0,1]"##));
    let at64 = grid_patch(&DEFAULT_DRAWING.resized(64), [255, 0, 0]);
    assert!(at64.contains("@boldy_hud.lines64"));
    assert!(
        pieces(&at64, "gp").contains(&[14.0, 98.0, 70.0, 28.0]),
        "same place at 64"
    );
    // The highlight: on the pixel under the pointer, and with Mirror on its reflections.
    let one = grid_cursor_patch(64, Some((1, 2)), false);
    balanced(&one);
    assert!(one.contains(r##""#drawgrid_cursor_size":3.5"##), "{one}");
    assert!(
        one.contains(r##""#drawgrid_cursor0_offset":[7,3.5]"##),
        "{one}"
    );
    assert!(
        one.contains(r##""#drawgrid_cursor1_visible":false"##),
        "{one}"
    );
    let four = grid_cursor_patch(16, Some((0, 0)), true);
    assert!(
        four.contains(r##""#drawgrid_cursor3_offset":[210,210]"##),
        "{four}"
    );
    let none = grid_cursor_patch(16, None, true);
    assert!(!none.contains("true") && !none.contains("offset"), "{none}");
    let blank = grid_patch(&Drawing::empty(32), [255, 0, 0]);
    balanced(&blank);
    assert!(pieces(&blank, "gp").is_empty());
    assert_ne!(
        grid_plan(&DEFAULT_DRAWING, [1, 2, 3]).header,
        grid_plan(&DEFAULT_DRAWING.resized(32), [1, 2, 3]).header
    );
}
