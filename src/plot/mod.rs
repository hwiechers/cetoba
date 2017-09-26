use std::collections::HashMap;
use std::io::Write;

use OpeningResult;
use Result;
use stats::dirichlet_pdf;

type Point = (f64, f64);
type Line = (Point, Point);
type Triangle = [Point; 3];

// Margins around the plot
const MARGIN: f64 = 80.;
// The length of the side of the triangle
const SIDE: f64 = 400.;
// Ratio of altitude to side f64::sqrt(3.) / 2.
const ALTITUDE_RATIO: f64 = 0.8660254037844386;
// The number of ticks on an axis
const NUM_TICKS: u32 = 10;
const TICK_LENGTH: f64 = 40.;
const TICK_TEXT_DY: f64 = -2.;

const AXIS_ARROW_SPACE: f64 = 60.;
const AXIS_LABEL_DY: f64 = -6.;
const AXIS_LABEL_FONT_SIZE: f64 = 16.; /* Must match the css included */

const W_CORNER: Point = (0., SIDE);
const D_CORNER: Point = (SIDE / 2., SIDE * (1. - ALTITUDE_RATIO));
const L_CORNER: Point = (SIDE, SIDE);

pub fn print_scatter_plot_svg<T: Write>(
    mut file: T,
    wdl_counts: &HashMap<&OpeningResult, u32>,
) -> Result<()> {
    writeln!(&mut file, "{}", include_str!("svg_start.txt"))?;
    writeln!(&mut file, "{}", include_str!("scatter_style.txt"))?;

    draw_main_group_start(&mut file)?;

    draw_main_triangle(&mut file)?;

    let max_count = f64::from(wdl_counts.values().cloned().max().unwrap_or(1));

    for (result, count) in wdl_counts {
        writeln!(
            &mut file,
            r#"    <circle cx="{:.3}" cy="{:.3}" r="{:.3}" />"#,
            SIDE * (result.black_win_proportion() + result.draw_proportion() / 2.),
            SIDE * (1. - result.draw_proportion() * ALTITUDE_RATIO),
            SIDE / 2. / f64::from(NUM_TICKS) * f64::sqrt(f64::from(*count) / max_count)
        )?;
    }

    writeln!(&mut file, "{}", include_str!("svg_end.txt"))?;
    Ok(())
}

pub fn print_dirichlet_plot_svg<T: Write>(mut file: T, alpha: &[f64; 3]) -> Result<()> {
    writeln!(&mut file, "{}", include_str!("svg_start.txt"))?;
    writeln!(&mut file, "{}", include_str!("dirichlet_style.txt"))?;
    draw_main_group_start(&mut file)?;
    const NUM_DIV: u32 = 25;

    let mut max: f64 = 0.;
    for triangle in iterate_triangles(NUM_DIV) {
        let midpoint = (
            (triangle.iter().map(|t| t.0).sum::<f64>() / 3.),
            (triangle.iter().map(|t| t.1).sum::<f64>() / 3.),
        );
        let value = dirichlet_pdf(alpha, midpoint.0, midpoint.1);
        if value > max {
            max = value;
        }
    }

    for triangle in iterate_triangles(NUM_DIV) {
        let point1 = convert_to_plot_coords(triangle[0]);
        let point2 = convert_to_plot_coords(triangle[1]);
        let point3 = convert_to_plot_coords(triangle[2]);
        let midpoint = (
            (triangle.iter().map(|t| t.0).sum::<f64>() / 3.),
            (triangle.iter().map(|t| t.1).sum::<f64>() / 3.),
        );
        let value = dirichlet_pdf(alpha, midpoint.0, midpoint.1);
        let hue = 240. * (1. - value / max);

        writeln!(
            &mut file,
            concat!(
                r#"    <polygon class="shading" fill="hsl({:.3},100%,50%)" "#,
                r#"points="{:.3},{:.3} {:.3},{:.3} {:.3},{:.3}" />"#
            ),
            hue,
            point1.0,
            point1.1,
            point2.0,
            point2.1,
            point3.0,
            point3.1,
        )?;
    }

    draw_main_triangle(&mut file)?;

    writeln!(&mut file, "{}", include_str!("svg_end.txt"))?;

    Ok(())
}

// Converts the (white win prob, draw prob) coordinates to coordinates on the plot.
fn convert_to_plot_coords(point: Point) -> Point {
    (
        SIDE * (1. - point.0 - point.1 / 2.),
        SIDE * (1. - ALTITUDE_RATIO * point.1),
    )
}

fn draw_main_group_start<T: Write>(mut file: T) -> Result<()> {
    writeln!(
        &mut file,
        r#"  <g transform="translate({:.3}, {:.3})">"#,
        MARGIN,
        MARGIN - D_CORNER.1
    )?;

    Ok(())
}

fn draw_main_triangle<T: Write>(mut file: T) -> Result<()> {
    writeln!(
        &mut file,
        r#"    <polygon class="main" points="0,{0:.3} {0:.3},{0:.3} {1:.3},{2:.3}" />"#,
        SIDE,
        SIDE / 2.,
        SIDE * (1. - ALTITUDE_RATIO)
    )?;

    // The ticks are ordered clockwise
    let right_ticks: Vec<Point> = fractions(NUM_TICKS)
        .map(|prob| (0., 1. - prob))
        .map(convert_to_plot_coords)
        .collect();

    let bottom_ticks: Vec<Point> = fractions(NUM_TICKS)
        .map(|prob| (prob, 0.))
        .map(convert_to_plot_coords)
        .collect();

    let left_ticks: Vec<Point> = fractions(NUM_TICKS)
        .map(|prob| (1. - prob, prob))
        .map(convert_to_plot_coords)
        .collect();

    let draw_lines: Vec<Line> = left_ticks
        .iter()
        .zip(right_ticks.iter().rev())
        .map(|(&start, &end)| ((start.0 - TICK_LENGTH, start.1), end))
        .collect();

    let black_lines: Vec<Line> = bottom_ticks
        .iter()
        .rev()
        .zip(right_ticks.iter())
        .map(|(&start, &end)| {
            (
                start,
                (
                    end.0 + TICK_LENGTH / 2.,
                    end.1 - TICK_LENGTH * ALTITUDE_RATIO,
                ),
            )
        })
        .collect();

    let white_lines: Vec<Line> = left_ticks
        .iter()
        .rev()
        .zip(bottom_ticks.iter())
        .map(|(&start, &end)| {
            (
                start,
                (
                    end.0 + TICK_LENGTH / 2.,
                    end.1 + TICK_LENGTH * ALTITUDE_RATIO,
                ),
            )
        })
        .collect();

    writeln!(&mut file, "<defs>",)?;
    for (index, line) in draw_lines.iter().enumerate() {
        writeln!(
            &mut file,
            r#"<path id="draw-line-{}" d="M {:.3} {:.3} L {:.3} {:.3}" />"#,
            index + 1,
            (line.0).0,
            (line.0).1,
            (line.1).0,
            (line.1).1
        )?;
    }
    for (index, line) in black_lines.iter().enumerate() {
        writeln!(
            &mut file,
            r#"<path id="black-line-{}" d="M {:.3} {:.3} L {:.3} {:.3}" />"#,
            index + 1,
            (line.0).0,
            (line.0).1,
            (line.1).0,
            (line.1).1
        )?;
    }
    for (index, line) in white_lines.iter().enumerate() {
        writeln!(
            &mut file,
            r#"<path id="white-line-{}" d="M {:.3} {:.3} L {:.3} {:.3}" />"#,
            index + 1,
            (line.0).0,
            (line.0).1,
            (line.1).0,
            (line.1).1
        )?;
    }

    let left_midpoint = (
        (W_CORNER.0 + D_CORNER.0) / 2.,
        (W_CORNER.1 + D_CORNER.1) / 2.,
    );
    let left_arrow_center = (
        left_midpoint.0 - AXIS_ARROW_SPACE * ALTITUDE_RATIO,
        left_midpoint.1 - AXIS_ARROW_SPACE / 2.,
    );
    writeln!(
        &mut file,
        concat!(
            r##"<path id="left-arrow" d="M {:.3} {:.3} L {:.3} {:.3}" "##,
            r##"marker-end="url(#arrow)" />"##
        ),
        left_arrow_center.0 - SIDE / 8.,
        left_arrow_center.1 + SIDE / 4. * ALTITUDE_RATIO,
        left_arrow_center.0 + SIDE / 8.,
        left_arrow_center.1 - SIDE / 4. * ALTITUDE_RATIO
    )?;

    let right_midpoint = (
        (D_CORNER.0 + L_CORNER.0) / 2.,
        (D_CORNER.1 + L_CORNER.1) / 2.,
    );
    let right_arrow_center = (
        right_midpoint.0 + AXIS_ARROW_SPACE * ALTITUDE_RATIO,
        right_midpoint.1 - AXIS_ARROW_SPACE / 2.,
    );
    writeln!(
        &mut file,
        concat!(
            r##"<path id="right-arrow" d="M {:.3} {:.3} L {:.3} {:.3}" "##,
            r##"marker-end="url(#arrow)" />"##
        ),
        right_arrow_center.0 - SIDE / 8.,
        right_arrow_center.1 - SIDE / 4. * ALTITUDE_RATIO,
        right_arrow_center.0 + SIDE / 8.,
        right_arrow_center.1 + SIDE / 4. * ALTITUDE_RATIO
    )?;

    let bottom_midpoint = (
        (L_CORNER.0 + W_CORNER.0) / 2.,
        (L_CORNER.1 + W_CORNER.1) / 2.,
    );
    let bottom_arrow_center = (bottom_midpoint.0, bottom_midpoint.1 + AXIS_ARROW_SPACE);
    writeln!(
        &mut file,
        concat!(
            r##"<path id="bottom-arrow" d="M {:.3} {:.3} L {:.3} {:.3}" "##,
            r##"marker-end="url(#arrow)" />"##
        ),
        bottom_arrow_center.0 + SIDE / 4.,
        bottom_arrow_center.1,
        bottom_arrow_center.0 - SIDE / 4.,
        bottom_arrow_center.1
    )?;
    writeln!(
        &mut file,
        r##"<path id="bottom-arrow-reverse" d="M {:.3} {:.3} L {:.3} {:.3}" />"##,
        bottom_arrow_center.0 - SIDE / 4.,
        bottom_arrow_center.1,
        bottom_arrow_center.0 + SIDE / 4.,
        bottom_arrow_center.1
    )?;

    writeln!(&mut file, "</defs>",)?;

    for ((index, _), perc) in draw_lines.iter().enumerate().zip(fractions(NUM_TICKS)) {
        writeln!(
            &mut file,
            r##"<use class="tick-line horizontal" href="#draw-line-{}" />"##,
            index + 1
        )?;
        writeln!(
            &mut file,
            concat!(
                r##"<text dy="{:.3}" class="tick">"##,
                r##"<textPath xlink:href="#draw-line-{}">{:.0}</textPath></text>"##
            ),
            TICK_TEXT_DY,
            index + 1,
            100. * perc
        )?;
    }

    for ((index, _), perc) in black_lines.iter().enumerate().zip(fractions(NUM_TICKS)) {
        writeln!(
            &mut file,
            r##"<use class="tick-line" href="#black-line-{}" />"##,
            index + 1
        )?;
        writeln!(
            &mut file,
            concat!(
                r##"<text text-anchor="end" dy="{:.3}" class="tick">"##,
                r##"<textPath xlink:href="#black-line-{}" startOffset="{:.3}">{:.0}"##,
                r##"</textPath></text>"##
            ),
            TICK_TEXT_DY,
            index + 1,
            SIDE * (1. - f64::from(index as u32 + 1) / f64::from(NUM_TICKS)) + TICK_LENGTH,
            100. * perc
        )?;
    }

    for ((index, _), perc) in white_lines.iter().enumerate().zip(fractions(NUM_TICKS)) {
        writeln!(
            &mut file,
            r##"<use class="tick-line" href="#white-line-{}" />"##,
            index + 1
        )?;
        writeln!(
            &mut file,
            concat!(
                r##"<text text-anchor="end" dy="{:.3}" class="tick">"##,
                r##"<textPath xlink:href="#white-line-{}" startOffset="{:.3}">{:.0}"##,
                r##"</textPath></text>"##
            ),
            TICK_TEXT_DY,
            index + 1,
            SIDE * (1. - f64::from(index as u32 + 1) / f64::from(NUM_TICKS)) + TICK_LENGTH,
            100. * perc
        )?;
    }

    writeln!(
        &mut file,
        r##"<use class="axis-arrow" href="#left-arrow" />"##
    )?;
    writeln!(
        &mut file,
        concat!(
            r##"<text text-anchor="middle" dy="{:.3}" class="axis-label">"##,
            r##"<textPath xlink:href="#left-arrow" startOffset="{:.3}">Draw"##,
            r##"</textPath></text>"##
        ),
        AXIS_LABEL_DY,
        SIDE / 4.
    )?;

    writeln!(
        &mut file,
        r##"<use class="axis-arrow" href="#right-arrow" />"##
    )?;
    writeln!(
        &mut file,
        concat!(
            r##"<text text-anchor="middle" dy="{:.3}" class="axis-label">"##,
            r##"<textPath xlink:href="#right-arrow" startOffset="{:.3}">Black Win"##,
            r##"</textPath></text>"##
        ),
        AXIS_LABEL_DY,
        SIDE / 4.
    )?;

    writeln!(
        &mut file,
        r##"<use class="axis-arrow" href="#bottom-arrow" />"##
    )?;
    writeln!(
        &mut file,
        concat!(
            r##"<text text-anchor="middle" dy="{:.3}" class="axis-label">"##,
            r##"<textPath xlink:href="#bottom-arrow-reverse" startOffset="{:.3}">White Win"##,
            r##"</textPath></text>"##
        ),
        AXIS_LABEL_FONT_SIZE - AXIS_LABEL_DY - 2.,
        SIDE / 4.
    )?;

    Ok(())
}

// Returns fractions between 0.0 and 1.0 with denominator in ascending order
// i.e. 1/denominator, 2/denominator, ..., (denominator-1)/denominator
fn fractions(denominator: u32) -> FractionIterator {
    FractionIterator {
        current: 0,
        denominator,
    }
}

struct FractionIterator {
    current: u32,
    denominator: u32,
}

impl Iterator for FractionIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        if self.current < self.denominator {
            Some(f64::from(self.current) / f64::from(self.denominator))
        } else {
            None
        }
    }
}

fn iterate_triangles(num_rows: u32) -> TriangleIterator {
    TriangleIterator {
        row: 0,
        index: 0,
        num_rows,
        point1: (0., 0.),
        point2: (0., 0.),
        point3: (0., 0.),
    }
}

struct TriangleIterator {
    row: u32,
    index: u32,
    num_rows: u32,
    point1: Point,
    point2: Point,
    point3: Point,
}

impl Iterator for TriangleIterator {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row > self.num_rows {
            return None;
        }

        self.index += 1;
        if self.index >= 2 * self.row {
            self.row += 1;
            if self.row > self.num_rows {
                return None;
            }

            let top = 1. - f64::from(self.row - 1) / f64::from(self.num_rows);
            let bottom = 1. - f64::from(self.row) / f64::from(self.num_rows);
            let left = f64::from(self.row) / f64::from(self.num_rows);
            let right = f64::from(self.row - 1) / f64::from(self.num_rows);

            self.point1 = (left, bottom);
            self.point2 = (right, top);
            self.point3 = (right, bottom);

            self.index = 1;
        } else {
            self.point1 = self.point2;
            self.point2 = self.point3;
            self.point3 = (
                f64::from(self.row - 1 - self.index / 2) / f64::from(self.num_rows),
                1. - f64::from(self.row - (self.index - 1) % 2) / f64::from(self.num_rows),
            );
        }

        Some([self.point1, self.point2, self.point3])
    }
}
