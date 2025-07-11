//! Some helpful combo operators for when the primitive operators don't do what you want easily.

use crate::{
    arena::Key,
    coord::Coordinate,
    dcel::{
        Dcel, EdgeKey, Flavor, Op, Operator, OperatorErr, Traverser, VertexKey,
        ops::{Kef, Kve, Mef, Mekh, Mve},
    },
};

pub struct CollapseEdge {
    kill_adjacent_faces: [Option<Kef>; 2],
    kill_vertex_edge: Kve,
}

impl CollapseEdge {
    pub fn new<F: Flavor>(
        dcel: &Dcel<F>,
        origin: Key<VertexKey>,
        edges: [Key<EdgeKey>; 2],
        vertex: Key<VertexKey>,
    ) -> Self
    where
        F::Vertex: Coordinate,
    {
        let last: Option<Key<EdgeKey>> = None;
        let kill_adjacent_faces = edges.map(|edge| {
            let edge_origin = edge.origin(dcel);

            let twin = edge.twin(dcel);
            let is_cyclic = edge_origin != origin && twin.next(dcel) == edge.prev(dcel).twin(dcel);
            if is_cyclic {
                return None;
            };

            let face = edge.face(dcel);
            let face_edges = Traverser::through(dcel, edge).unwrap().count();

            if face_edges == 3 {
                let kill = if edge.origin(dcel) == origin {
                    edge.next(dcel)
                } else {
                    edge.prev(dcel)
                };
                let kill_twin = kill.twin(dcel);

                Some(Kef {
                    face,
                    edges: [kill, kill_twin],
                })
            } else {
                None
            }
        });

        Self {
            kill_adjacent_faces,
            kill_vertex_edge: Kve {
                edges,
                origin,
                vertex,
            },
        }
    }
    pub fn apply<F: Flavor>(self, dcel: &mut Dcel<F>) -> UncollapseEdge<F>
    where
        F::Vertex: Coordinate,
    {
        let make_adjacent_faces = self
            .kill_adjacent_faces
            .map(|op| op.and_then(|op| dcel.check_apply(op).ok()));
        let make_vertex_edge = dcel.check_apply(self.kill_vertex_edge).unwrap();
        UncollapseEdge {
            make_adjacent_faces,
            make_vertex_edge,
        }
    }
}

pub struct UncollapseEdge<F: Flavor> {
    make_adjacent_faces: [Option<Mef<F>>; 2],
    make_vertex_edge: Mve<F>,
}

impl<F: Flavor> UncollapseEdge<F>
where
    F::Vertex: Coordinate,
{
    pub fn apply(self, dcel: &mut Dcel<F>) -> CollapseEdge {
        let kill_vertex_edge = dcel.check_apply(self.make_vertex_edge).unwrap();
        let kill_adjacent_faces = self
            .make_adjacent_faces
            .map(|op| op.and_then(|op| dcel.check_apply(op).ok()));
        CollapseEdge {
            kill_adjacent_faces,
            kill_vertex_edge,
        }
    }
}

pub struct WeldVertex<F: Flavor> {
    bridge: Option<Mef<F>>,
    collapse: CollapseEdge,
}

pub struct UnweldVertex<F: Flavor> {
    unbridge: Option<Kef>,
    uncollapse: UncollapseEdge<F>,
}

pub enum LinkVertices<F: Flavor> {
    Mef(Mef<F>),
    Mekh(Mekh),
}

/// You probably want Op::Euler::Mve

impl<F: Flavor> LinkVertices<F>
where
    F::Vertex: Coordinate,
    F::Edge: Default,
    F::Face: Default,
{
    pub fn new(dcel: &mut Dcel<F>, v1: Key<VertexKey>, v2: Key<VertexKey>) -> Self {
        let mef = Mef {
            vertices: [v1, v2],
            data: (Default::default(), Default::default(), Default::default()),
        };

        match mef.check(dcel) {
            Ok(_) => LinkVertices::Mef(mef),
            Err(_) => LinkVertices::Mekh(Mekh {}),
        }
    }

    pub fn apply(self, dcel: &mut Dcel<F>) -> UnlinkVertices {
        match self {
            LinkVertices::Mef(mef) => UnlinkVertices::Kef(dcel.check_apply(mef).unwrap()),
            LinkVertices::Mekh(mekh) => todo!(),
        }
    }
}

pub enum UnlinkVertices {
    Kef(Kef),
}
