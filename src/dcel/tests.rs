use crate::dcel::{self, Dcel, Flavor, Traverser, draw::Draw, error::Error, ops, vis};

pub struct TestFlavor;
impl Flavor for TestFlavor {
    type Vertex = [f32; 2];
    type Edge = ();
    type Face = ();
}

#[test]
fn test_find_next_prev() {}

#[test]
fn mev_cycle() {
    let mut hourglass = make_hourglass();

    let left = hourglass.vertices.key(1).unwrap();
    let right = hourglass.vertices.key(3).unwrap();

    let collapse = ops::CollapseEdge::new(
        &hourglass,
        left,
        [
            hourglass.edges.key(5).unwrap(),
            hourglass.edges.key(6).unwrap(),
        ],
        right,
    );

    let uncollapse = collapse.apply(&mut hourglass);

    // uncollapse.apply(&mut hourglass);

    std::fs::write("./test.mev_cycle.svg", vis::vis_svg(&hourglass)).unwrap();
}

#[test]
fn mev_kve() {}

#[test]
fn mve_kev() {}

/*

makes this shape:

   O      O
   |\    /|
   | \__/ |
   | O__O |
   | /  \ |
   |/    \|
   O      O

*/
fn make_hourglass() -> Dcel<TestFlavor> {
    let (mut draw, [top_left, bottom_left]) = Draw::new(Dcel::default(), [-4., -4.], [-4., 4.]);

    let middle_left = draw.line_to([-1., 0.]);

    draw.close_path(top_left);
    draw.set_key(middle_left);

    let middle_right = draw.line_to([1., 0.]);
    let bottom_right = draw.line_to([4., -4.]);
    let top_right = draw.line_to([4., 4.]);
    draw.close_path(middle_right);

    draw.finish()
}

/*

makes this shape:

    0____0____0_____0
    |    |    |     |
    |    |    |     |
    |----0----0-----0
    |    |    |     |
    |____|____|_____|
    0    0    0     0
*/
fn make_grid() {}
