use crate::arena::Key;

enum Seed {
    Vertex(Key),
    Edge(Key),
    Face(Key),
}

pub struct ComponentSeed {
    seed: Seed,
    bounds: [f32; 4],
}
