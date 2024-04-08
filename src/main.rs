mod xml;

use std::{f64::consts::TAU, fmt::Write, fs};

use resvg::{
    tiny_skia::Pixmap,
    usvg::{self, fontdb},
};
use xml::{DisplayAlreadyEscaped, Writer};

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (1.0 - t) * a + t * b
}

fn lerp_vec(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t)]
}

fn r(x: f64) -> f64 {
    (x * 1_000.0).round() / 1_000.0
}

fn intersect(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> Option<[f64; 2]> {
    let d = (a1[0] - a2[0]) * (b1[1] - b2[1]) - (a1[1] - a2[1]) * (b1[0] - b2[0]);
    if d.abs() < 1e-6 {
        return None;
    }

    let t = ((a1[0] - b1[0]) * (b1[1] - b2[1]) - (a1[1] - b1[1]) * (b1[0] - b2[0])) / d;
    Some([lerp(a1[0], a2[0], t), lerp(a1[1], a2[1], t)])
}

struct PathBuilder {
    buf: String,
}

impl PathBuilder {
    fn new() -> Self {
        Self { buf: String::new() }
    }

    fn move_to(&mut self, [x, y]: [f64; 2]) {
        let _ = write!(self.buf, "M{},{}", r(x), r(y));
    }

    fn line_to(&mut self, [x, y]: [f64; 2]) {
        let _ = write!(self.buf, "L{},{}", r(x), r(y));
    }

    fn cubic_to(&mut self, [x1, y1]: [f64; 2], [x2, y2]: [f64; 2], [x, y]: [f64; 2]) {
        let _ = write!(
            self.buf,
            "C{},{} {},{} {},{}",
            r(x1),
            r(y1),
            r(x2),
            r(y2),
            r(x),
            r(y)
        );
    }

    fn close(&mut self) {
        self.buf.push('Z');
    }
}

fn main() {
    let mut buf = String::new();

    write(&mut buf, [1000.0, 950.0]).unwrap();

    fs::write("icon.svg", &buf).unwrap();

    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let mut pixmap = Pixmap::new(1000, 950).unwrap();
    resvg::render(
        &usvg::Tree::from_str(&buf, &Default::default(), &database).unwrap(),
        Default::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.save_png("icon.png").unwrap();
}

fn write(buf: &mut String, dim @ [width, height]: [f64; 2]) -> std::fmt::Result {
    let mut writer = Writer::new_with_default_header(buf)?;

    writer.tag_with_content(
        "svg",
        [
            (
                "viewBox",
                DisplayAlreadyEscaped(format_args!("0 0 {width} {height}")),
            ),
            (
                "xmlns",
                DisplayAlreadyEscaped(format_args!("http://www.w3.org/2000/svg")),
            ),
        ],
        |writer| {
            let mut center = dim.map(|v| 0.5 * v);
            center[1] += 55.0;
            let radius = 690.0;
            let corners = [0, 1, 2].map(|corner| {
                let angle = corner as f64 / 3.0 * TAU;
                [
                    angle.sin() * radius + center[0],
                    -angle.cos() * radius + center[1],
                ]
            });

            const ROUND_START: f64 = 0.3;
            const ROUND_END: f64 = 1.0 - ROUND_START;
            let circle_distance = 240.0;
            let circle_radius = 170.0;

            let circles = [
                (0, "#D73F4A", "#C22B36", "r"),
                (1, "#3FCD46", "#27BB2E", "g"),
                (2, "#3F5CD7", "#2B48C2", "b"),
            ]
            .map(|(corner, top, bottom, id)| {
                let angle = corner as f64 / 3.0 * TAU;
                (
                    id,
                    [
                        angle.sin() * circle_distance + center[0],
                        -angle.cos() * circle_distance + center[1],
                    ],
                    top,
                    bottom,
                )
            });

            writer.tag("defs", |writer| {
                writer.content(|writer| {
                    for (id, _, top, bottom) in circles {
                        writer.tag_with_content(
                            "linearGradient",
                            [
                                ("id", DisplayAlreadyEscaped(id)),
                                ("gradientTransform", DisplayAlreadyEscaped("rotate(90)")),
                            ],
                            |writer| {
                                writer.empty_tag(
                                    "stop",
                                    [("stop-color", DisplayAlreadyEscaped(top))],
                                )?;
                                writer.empty_tag(
                                    "stop",
                                    [
                                        ("offset", DisplayAlreadyEscaped("1")),
                                        ("stop-color", DisplayAlreadyEscaped(bottom)),
                                    ],
                                )?;
                                Ok(())
                            },
                        )?;
                    }

                    writer.tag_with_content(
                        "filter",
                        [("id", DisplayAlreadyEscaped("s"))],
                        |writer| {
                            writer.empty_tag("feOffset", [("dy", DisplayAlreadyEscaped("10"))])?;
                            writer.empty_tag(
                                "feGaussianBlur",
                                [
                                    ("result", DisplayAlreadyEscaped("b")),
                                    ("stdDeviation", DisplayAlreadyEscaped("20")),
                                ],
                            )?;
                            writer.empty_tag(
                                "feFlood",
                                [("flood-opacity", DisplayAlreadyEscaped(".8"))],
                            )?;
                            writer.empty_tag(
                                "feComposite",
                                [
                                    ("in2", DisplayAlreadyEscaped("b")),
                                    ("operator", DisplayAlreadyEscaped("in")),
                                ],
                            )?;
                            writer.empty_tag(
                                "feComposite",
                                [("in", DisplayAlreadyEscaped("SourceGraphic"))],
                            )
                        },
                    )?;

                    let mut tri = PathBuilder::new();

                    for i in 0..3 {
                        let before = corners[(i + 3 - 1) % 3];
                        let current = corners[i];
                        let after = corners[(i + 1) % 3];

                        if i == 0 {
                            let from = lerp_vec(before, current, ROUND_END);
                            tri.move_to(from);
                        }

                        let p1 = lerp_vec(before, current, ROUND_END + 0.125);
                        let p2 = lerp_vec(current, after, ROUND_START - 0.125);
                        let end = lerp_vec(current, after, ROUND_START);
                        tri.cubic_to(p1, p2, end);

                        if i != 2 {
                            let to = lerp_vec(current, after, ROUND_END);
                            tri.line_to(to);
                        }
                    }

                    tri.close();

                    writer.empty_tag(
                        "path",
                        [
                            ("id", DisplayAlreadyEscaped("t")),
                            ("d", DisplayAlreadyEscaped(&*tri.buf)),
                        ],
                    )
                })
            })?;

            writer.empty_tag(
                "use",
                [
                    ("href", DisplayAlreadyEscaped("#t")),
                    ("fill", DisplayAlreadyEscaped("#404040")),
                ],
            )?;

            let angle = -TAU / 8.0;
            let [dx, dy] = [angle.cos() * circle_radius, angle.sin() * circle_radius];
            let mut shadow = PathBuilder::new();
            let red_circle = circles[0].1;
            let blue_circle = circles[2].1;

            let top_intersect = intersect(
                [red_circle[0] + dx, red_circle[1] + dy],
                [red_circle[0] + dx - dy, red_circle[1] + dy + dx],
                corners[0],
                corners[1],
            )
            .unwrap();

            shadow.move_to(top_intersect);
            shadow.line_to([red_circle[0] + dx, red_circle[1] + dy]);
            shadow.line_to([red_circle[0] - dx, red_circle[1] - dy]);
            shadow.line_to([red_circle[0] - dx - 2.0 * dy, red_circle[1] - dy + 2.0 * dx]);
            shadow.line_to([
                blue_circle[0] + dx - 1.5 * dy,
                blue_circle[1] + dy + 1.5 * dx,
            ]);
            shadow.line_to([blue_circle[0] + dx, blue_circle[1] + dy]);
            shadow.line_to([blue_circle[0] - dx, blue_circle[1] - dy]);

            let bottom_intersect = intersect(
                [blue_circle[0] - dx, blue_circle[1] - dy],
                [blue_circle[0] - dx - dy, blue_circle[1] - dy + dx],
                corners[1],
                corners[2],
            )
            .unwrap();
            shadow.line_to(bottom_intersect);

            let before = corners[2];
            let current = corners[1];
            let after = corners[0];

            let from = lerp_vec(before, current, ROUND_END);
            shadow.line_to(from);

            let p1 = lerp_vec(before, current, ROUND_END + 0.125);
            let p2 = lerp_vec(current, after, ROUND_START - 0.125);
            let end = lerp_vec(current, after, ROUND_START);
            shadow.cubic_to(p1, p2, end);

            shadow.close();

            writer.empty_tag(
                "path",
                [
                    ("d", DisplayAlreadyEscaped(&*shadow.buf)),
                    ("fill-opacity", DisplayAlreadyEscaped(".125")),
                ],
            )?;

            for (id, [x, y], _, _) in circles {
                writer.empty_tag(
                    "circle",
                    [
                        ("cx", DisplayAlreadyEscaped(format_args!("{}", r(x)))),
                        ("cy", DisplayAlreadyEscaped(format_args!("{}", r(y)))),
                        ("r", DisplayAlreadyEscaped(format_args!("{circle_radius}"))),
                        ("fill", DisplayAlreadyEscaped(format_args!("url(#{})", id))),
                    ],
                )?;
            }

            writer.empty_tag(
                "use",
                [
                    ("href", DisplayAlreadyEscaped("#t")),
                    ("fill", DisplayAlreadyEscaped("none")),
                    ("stroke", DisplayAlreadyEscaped("#FFF")),
                    ("stroke-width", DisplayAlreadyEscaped("40")),
                    ("filter", DisplayAlreadyEscaped("url(#s)")),
                ],
            )
        },
    )
}
