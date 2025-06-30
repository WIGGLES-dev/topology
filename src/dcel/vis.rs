use crate::{coord::Coordinate, dcel::flavor::Flavor};
use std::fmt::Write;

use super::Dcel;

fn offset_line(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    lateral_offset: f32,
    trim_fraction: f32, // e.g., 0.2 to trim 20% from both ends
) -> ((f32, f32), (f32, f32)) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return ((x1, y1), (x2, y2)); // avoid divide-by-zero
    }

    let ux = dx / len;
    let uy = dy / len;

    // Lateral offset (perpendicular)
    let ox = -uy * lateral_offset;
    let oy = ux * lateral_offset;

    // Trim the ends
    let trim = len * trim_fraction;
    let tx = ux * trim;
    let ty = uy * trim;

    let new_start = (x1 + tx + ox, y1 + ty + oy);
    let new_end = (x2 - tx + ox, y2 - ty + oy);

    (new_start, new_end)
}

pub fn vis_svg<F: Flavor>(dcel: &Dcel<F>) -> String
where
    F::Vertex: Coordinate,
{
    let mut svg = String::new();
    svg.push_str(
		r#"<svg viewBox="-7.5 -7.5 15 15" width="100%" height="100%" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid meet">
		<defs>
			<marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5"
				markerWidth="2" markerHeight="2"
				orient="auto-start-reverse">
			<path d="M 0 0 L 10 5 L 0 10 z" fill="white" />
			</marker>
		</defs>
		"#,
    );

    let mut labels = String::new();
    let mut edges = String::new();
    let mut vertices = String::new();

    // Draw vertices
    for (vertex, key) in dcel.vertices.iter() {
        let [x, y] = vertex.weight.xy();
        write!(
            &mut vertices,
            r#"<circle cx="{x}" cy="{y}" r="0.5" fill="red"/>"#
        )
        .unwrap();

        write!(
            &mut labels,
            r#"<text x="{x}" y="{y}" font-size="0.4" fill="white" dx="0" dy="0">{}</text>"#,
            key.get()
        )
        .unwrap();
    }

    // Draw edges with direction
    for (edge, key) in dcel.edges.iter() {
        println!("vis {key} {} {}", edge.origin, edge.next);
        let from = &dcel.vertices[edge.origin];
        let to = &dcel.vertices[dcel.edges[edge.next].origin];

        let [x1, y1] = from.weight.xy();
        let [x2, y2] = to.weight.xy();

        let ((x1, y1), (x2, y2)) = offset_line(x1, y1, x2, y2, 0.2, 0.2);

        write!(
				&mut edges,
				r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="black" stroke-width="0.2" marker-end="url(#arrow)"/>"#
			)
			.unwrap();

        // Compute midpoint for label
        let mx = (x1 + x2) / 2.0;
        let my = (y1 + y2) / 2.0;

        write!(
            &mut labels,
            r#"<text x="{mx}" y="{my}" font-size="0.4" fill="white" dx="0" dy="0">{}</text>"#,
            key.get()
        )
        .unwrap();
    }

    svg.push_str(&vertices);
    svg.push_str(&edges);
    svg.push_str(&labels);
    svg.push_str("</svg>");

    svg
}
