use serde::{Deserialize, Serialize};

use crate::coord::{Coordinate, Precision, UpdateCoordinate};

use super::{Dcel, EdgeKey, Key, VertexKey, error::Error};

pub enum NewOrExisting<V> {
    New(V),
    Existing(Key<VertexKey>),
}

pub enum Mutation<V, E> {
    CreateVertex(V),
    MoveVertex {
        vertex: Key<VertexKey>,
        translation: [Precision; 3],
    },
    LinkVertices {
        v1: NewOrExisting<V>,
        v2: NewOrExisting<V>,
        w1: E,
        w2: E,
    },
    MergeVertices {
        left: Key<VertexKey>,
        right: Key<VertexKey>,
    },
}

#[derive(Deserialize, Serialize)]
pub enum AppliedMutation {
    CreateVertex(Key<VertexKey>),
    MoveVertex {
        vertex: Key<VertexKey>,
        translation: [Precision; 3],
    },
    LinkVertices(Key<EdgeKey>),
    MergeVertices {
        left: Key<VertexKey>,
        right: Key<VertexKey>,
    },
}

pub struct Mutations<V, E> {
    mutations: Vec<Mutation<V, E>>,
}

impl<V, E> Default for Mutations<V, E> {
    fn default() -> Self {
        Self { mutations: vec![] }
    }
}

impl<V, E> Mutations<V, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_vertex(&mut self, value: V) {
        self.mutations.push(Mutation::CreateVertex(value));
    }

    pub fn move_vertex(&mut self, vertex: Key<VertexKey>, translation: [Precision; 3]) {
        self.mutations.push(Mutation::MoveVertex {
            vertex,
            translation,
        })
    }

    pub fn link_vertices(&mut self, v1: NewOrExisting<V>, v2: NewOrExisting<V>, w1: E, w2: E) {
        self.mutations
            .push(Mutation::LinkVertices { v1, v2, w1, w2 })
    }

    pub fn merge_vertices(&mut self, left: Key<VertexKey>, right: Key<VertexKey>) {
        self.mutations.push(Mutation::MergeVertices { left, right })
    }

    pub fn apply<F>(
        &mut self,
        dcel: &mut Dcel<V, E, F>,
        history: &mut Vec<AppliedMutation>,
    ) -> Result<(), Error>
    where
        V: Coordinate + UpdateCoordinate,
        F: Default,
    {
        for mutation in self.mutations.drain(0..) {
            match mutation {
                Mutation::CreateVertex(weight) => {
                    let key = dcel.insert_vertex(weight, None);
                    history.push(AppliedMutation::CreateVertex(key))
                }
                Mutation::LinkVertices { v1, v2, w1, w2 } => {
                    let v1 = match v1 {
                        NewOrExisting::Existing(key) => key,
                        NewOrExisting::New(weight) => dcel.insert_vertex(weight, None),
                    };
                    let v2 = match v2 {
                        NewOrExisting::Existing(key) => key,
                        NewOrExisting::New(weight) => dcel.insert_vertex(weight, None),
                    };
                    let delta = dcel.link_vertices(v1, v2, w1, w2)?;
                    let key = delta.edges_created[0];
                    history.push(AppliedMutation::LinkVertices(key));
                }
                Mutation::MergeVertices { left, right } => {}
                Mutation::MoveVertex {
                    vertex,
                    translation,
                } => {
                    let v = &mut dcel
                        .vertices
                        .get_mut(vertex)
                        .ok_or(Error::VertexDoesNotExist)?
                        .weight;
                    let [x, y, z] = v.xyz();
                    let [x1, y1, z1] = translation;

                    v.set_xyz([x + x1, y + y1, z + z1]);

                    history.push(AppliedMutation::MoveVertex {
                        vertex,
                        translation,
                    });
                }
            }
        }
        Ok(())
    }
}
