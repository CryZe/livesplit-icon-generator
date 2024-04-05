mod xml;

use std::{f64::consts::TAU, fmt::Write};

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

struct PathBuilder {
    buf: String,
}

impl PathBuilder {
    fn new() -> Self {
        Self { buf: String::new() }
    }

    fn move_to(&mut self, x: f64, y: f64) {
        if !self.buf.is_empty() {
            self.buf.push(' ');
        }
        let _ = write!(self.buf, "M{},{}", r(x), r(y));
    }

    fn line_to(&mut self, x: f64, y: f64) {
        if !self.buf.is_empty() {
            self.buf.push(' ');
        }
        let _ = write!(self.buf, "L{},{}", r(x), r(y));
    }

    fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        if !self.buf.is_empty() {
            self.buf.push(' ');
        }
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
        if !self.buf.is_empty() {
            self.buf.push(' ');
        }
        self.buf.push('Z');
    }
}

fn main() {
    let mut buf = String::new();

    write(&mut buf, [1000.0; 2]).unwrap();

    std::fs::write("icon.svg", buf).unwrap();
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
            center[1] += 50.0;
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
                (0, "#D73F4A", "#C22B36"),
                (1, "#3FCD46", "#27BB2E"),
                (2, "#3F5CD7", "#2B48C2"),
            ]
            .map(|(corner, top, bottom)| {
                let angle = corner as f64 / 3.0 * TAU;
                (
                    corner,
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
                    for (corner, _, top, bottom) in circles {
                        writer.tag_with_content(
                            "linearGradient",
                            [
                                ("id", DisplayAlreadyEscaped(format_args!("{}", corner))),
                                (
                                    "gradientTransform",
                                    DisplayAlreadyEscaped(format_args!("rotate(90)")),
                                ),
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
                        [("id", DisplayAlreadyEscaped("3"))],
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
                            tri.move_to(from[0], from[1]);
                        }

                        let p1 = lerp_vec(before, current, ROUND_END + 0.125);
                        let p2 = lerp_vec(current, after, ROUND_START - 0.125);
                        let end = lerp_vec(current, after, ROUND_START);
                        tri.cubic_to(p1[0], p1[1], p2[0], p2[1], end[0], end[1]);

                        if i != 2 {
                            let to = lerp_vec(current, after, ROUND_END);
                            tri.line_to(to[0], to[1]);
                        }
                    }

                    tri.close();

                    writer.empty_tag(
                        "path",
                        [
                            ("id", DisplayAlreadyEscaped("4")),
                            ("d", DisplayAlreadyEscaped(&*tri.buf)),
                        ],
                    )
                })
            })?;

            writer.empty_tag(
                "use",
                [
                    ("href", DisplayAlreadyEscaped("#4")),
                    ("fill", DisplayAlreadyEscaped("#404040")),
                ],
            )?;

            let angle = -TAU / 8.0;
            let [dx, dy] = [angle.cos() * circle_radius, angle.sin() * circle_radius];
            let mut shadow = PathBuilder::new();
            let red_circle = circles[0].1;
            let blue_circle = circles[2].1;
            shadow.move_to(red_circle[0] + dx - 1.5 * dy, red_circle[1] + dy + 1.5 * dx);
            shadow.line_to(red_circle[0] + dx, red_circle[1] + dy);
            shadow.line_to(red_circle[0] - dx, red_circle[1] - dy);
            shadow.line_to(red_circle[0] - dx - 2.0 * dy, red_circle[1] - dy + 2.0 * dx);
            shadow.line_to(
                blue_circle[0] + dx - 1.5 * dy,
                blue_circle[1] + dy + 1.5 * dx,
            );
            shadow.line_to(blue_circle[0] + dx, blue_circle[1] + dy);
            shadow.line_to(blue_circle[0] - dx, blue_circle[1] - dy);
            shadow.line_to(
                blue_circle[0] - dx - 0.9 * dy,
                blue_circle[1] - dy + 0.9 * dx,
            );
            let before = corners[2];
            let current = corners[1];
            let after = corners[0];

            let from = lerp_vec(before, current, ROUND_END);
            shadow.line_to(from[0], from[1]);

            let p1 = lerp_vec(before, current, ROUND_END + 0.125);
            let p2 = lerp_vec(current, after, ROUND_START - 0.125);
            let end = lerp_vec(current, after, ROUND_START);
            shadow.cubic_to(p1[0], p1[1], p2[0], p2[1], end[0], end[1]);

            shadow.close();

            writer.empty_tag(
                "path",
                [
                    ("d", DisplayAlreadyEscaped(&*shadow.buf)),
                    ("fill-opacity", DisplayAlreadyEscaped(".125")),
                ],
            )?;

            for (corner, [x, y], _, _) in circles {
                writer.empty_tag(
                    "circle",
                    [
                        ("cx", DisplayAlreadyEscaped(format_args!("{}", r(x)))),
                        ("cy", DisplayAlreadyEscaped(format_args!("{}", r(y)))),
                        ("r", DisplayAlreadyEscaped(format_args!("{circle_radius}"))),
                        (
                            "fill",
                            DisplayAlreadyEscaped(format_args!("url(#{})", corner)),
                        ),
                    ],
                )?;
            }

            writer.empty_tag(
                "use",
                [
                    ("href", DisplayAlreadyEscaped("#4")),
                    ("fill", DisplayAlreadyEscaped("none")),
                    ("stroke", DisplayAlreadyEscaped("#fff")),
                    ("stroke-width", DisplayAlreadyEscaped("40")),
                    ("filter", DisplayAlreadyEscaped("url(#3)")),
                ],
            )?;

            Ok(())
        },
    )
}
