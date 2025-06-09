mod component;
pub mod error;
mod flavor;
mod mutation;
mod spacial_index;
mod traverser;
mod util;
pub mod vis;

use crate::util::ShoeString;

use error::Error::{self, EdgeDoesNotExist, FaceDoesNotExist, VertexDoesNotExist};

pub use mutation::*;
pub use traverser::*;

use crate::{
    arena::{Arena, Key},
    coord::{Coordinate, FromCoordinate, Precision, UpdateCoordinate, sort_clockwise},
};

pub struct Vertex<T> {
    pub edge: Option<Key<EdgeKey>>,
    pub weight: T,
}

pub struct Edge<T> {
    pub origin: Key<VertexKey>,
    pub twin: Key<EdgeKey>,
    pub prev: Key<EdgeKey>,
    pub next: Key<EdgeKey>,
    pub face: Option<Key<FaceKey>>,
    pub weight: T,
}

pub struct Face<T> {
    pub edge: Key<EdgeKey>,
    pub signed_area: f32,
    pub holes: Vec<Key<FaceKey>>,
    pub weight: T,
}

impl<T> Face<T> {
    pub fn default_at_edge(edge: Key<EdgeKey>, signed_area: f32) -> Self
    where
        T: Default,
    {
        Self {
            edge,
            signed_area,
            holes: vec![],
            weight: T::default(),
        }
    }

    pub fn is_bounding(&self) -> bool {
        self.signed_area < 0.
    }
}

pub struct VertexKey;
pub struct EdgeKey;
pub struct FaceKey;

pub struct Dcel<V, E, F>
where
    F: Default,
{
    vertices: Arena<Vertex<V>, VertexKey>,
    edges: Arena<Edge<E>, EdgeKey>,
    faces: Arena<Face<F>, FaceKey>,
    vertex_index: spacial_index::VertexIndex,
    face_index: spacial_index::FaceIndex,
}

impl<V, E, F> Dcel<V, E, F>
where
    F: Default,
{
    pub fn vertices(&self) -> &Arena<Vertex<V>, VertexKey> {
        &self.vertices
    }

    pub fn vertex(&self, key: Key<VertexKey>) -> &Vertex<V> {
        &self.vertices[key]
    }

    pub fn edges(&self) -> &Arena<Edge<E>, EdgeKey> {
        &self.edges
    }

    pub fn edge(&self, key: Key<EdgeKey>) -> &Edge<E> {
        &self.edges[key]
    }

    pub fn faces(&self) -> &Arena<Face<F>, FaceKey> {
        &self.faces
    }

    pub fn face(&self, key: Key<FaceKey>) -> &Face<F> {
        &self.faces[key]
    }

    pub fn bounding_face(&self) -> Option<Key<FaceKey>> {
        self.faces.key(1)
    }
}

impl<V, E, F> Default for Dcel<V, E, F>
where
    F: Default,
{
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            edges: Default::default(),
            faces: Default::default(),
            vertex_index: Default::default(),
            face_index: Default::default(),
        }
    }
}

pub struct DcelDelta {
    pub edges_created: Vec<Key<EdgeKey>>,
    pub faces_created: Vec<Key<FaceKey>>,
    pub faces_removed: Vec<Key<FaceKey>>,
    pub edges_removed: Vec<Key<EdgeKey>>,
    pub vertices_created: Vec<Key<VertexKey>>,
    pub vertices_removed: Vec<Key<VertexKey>>,
}

impl DcelDelta {
    pub fn extend(&mut self, rhs: &DcelDelta) {
        self.edges_created.extend(rhs.edges_created.iter());
        self.faces_created.extend(rhs.faces_created.iter());
        self.faces_removed.extend(rhs.faces_removed.iter());
        self.edges_removed.extend(rhs.edges_removed.iter());
        self.vertices_created.extend(rhs.vertices_created.iter());
        self.vertices_removed.extend(rhs.vertices_removed.iter());
    }
}

impl Default for DcelDelta {
    fn default() -> Self {
        Self {
            edges_created: vec![],
            faces_created: vec![],
            faces_removed: vec![],
            edges_removed: vec![],
            vertices_created: vec![],
            vertices_removed: vec![],
        }
    }
}

impl<V, E, F> Dcel<V, E, F>
where
    F: Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_raw(
        vertices: Arena<Vertex<V>, VertexKey>,
        edges: Arena<Edge<E>, EdgeKey>,
        faces: Arena<Face<F>, FaceKey>,
    ) -> Self {
        Self {
            vertices,
            edges,
            faces,
            face_index: Default::default(),
            vertex_index: Default::default(),
        }
    }

    pub fn through<Cb>(&mut self, edge: Key<EdgeKey>, mut cb: Cb) -> Result<(), Error>
    where
        Cb: FnMut(&mut Self, Key<EdgeKey>),
    {
        let mut traverser = Traverser::new(&self, edge)?;

        loop {
            cb(self, traverser.edge());
            traverser.next(&self);
            if traverser.is_at_start() {
                break;
            }
        }
        Ok(())
    }

    pub fn around<Cb>(&mut self, vertex: Key<VertexKey>, mut cb: Cb) -> Result<(), Error>
    where
        Cb: FnMut(&mut Self, Key<EdgeKey>),
    {
        let mut traverser = Traverser::at(&self, vertex)?;
        loop {
            let edge = traverser.edge();
            // super dangerous to mutate the dcel while you're traversing it.
            traverser.local_next(&self);
            cb(self, edge);
            if traverser.is_at_start() {
                break;
            }
        }
        Ok(())
    }

    pub fn face_path(&self, key: Key<FaceKey>) -> Result<Vec<f32>, Error>
    where
        V: Coordinate,
    {
        let incident = self.faces.get(key).ok_or(FaceDoesNotExist)?.edge;
        let mut traverser = Traverser::new(self, incident)?;
        traverser.next(&self);

        let mut path = vec![];
        path.extend(self.vertices[self.edges[incident].origin].weight.xy());

        while !traverser.is_at_start() {
            let e = &self.edges[traverser.edge()];
            let eo = &self.vertices[e.origin];

            path.extend(eo.weight.xy());
            traverser.next(&self);
        }

        Ok(path)
    }

    pub fn reverse_loop(&mut self, face: Key<FaceKey>) {}

    pub fn update_vertex_coord(
        &mut self,
        key: Key<VertexKey>,
        pos: [Precision; 3],
    ) -> Result<(), Error>
    where
        V: UpdateCoordinate,
    {
        self.vertices
            .get_mut(key)
            .ok_or(VertexDoesNotExist)?
            .weight
            .set_xyz(pos);
        Ok(())
    }

    pub fn translate_vertex_coord(
        &mut self,
        key: Key<VertexKey>,
        translation: [Precision; 3],
    ) -> Result<(), Error>
    where
        V: Coordinate + UpdateCoordinate,
    {
        let v = &mut self.vertices.get_mut(key).ok_or(VertexDoesNotExist)?.weight;
        let [x, y, z] = v.xyz();
        let [x1, y1, z1] = translation;
        v.set_xyz([x + x1, y + y1, z + z1]);
        Ok(())
    }

    pub fn delete_edge(&mut self, e: Key<EdgeKey>) -> Result<(), Error> {
        let edge = self.edges.remove(e).ok_or(EdgeDoesNotExist)?;
        let twin = self.edges.remove(edge.twin).ok_or(EdgeDoesNotExist)?;

        if let Some(f1) = edge.face {
            let is_bounding = self.faces[f1].is_bounding();
            if !is_bounding {
                self.faces.remove(f1);
            }
        }
        if let Some(f2) = twin.face {
            let is_bounding = self.faces[f2].is_bounding();
            if !is_bounding {
                self.faces.remove(f2);
            }
        }

        Ok(())
    }

    pub fn delete_vertex(&mut self, v: Key<VertexKey>) -> Result<(), Error> {
        let around = Traverser::around(self, v)?;
        let to_delete: Vec<Key<EdgeKey>> = around.collect();

        for key in &to_delete {
            self.delete_edge(*key)?;
        }

        Ok(())
    }

    pub fn insert_vertex(&mut self, weight: V, edge: Option<Key<EdgeKey>>) -> Key<VertexKey> {
        self.vertices.insert(Vertex { edge, weight })
    }

    fn make_half_edges(
        &mut self,
        v1: Key<VertexKey>,
        v2: Key<VertexKey>,
        w1: E,
        w2: E,
    ) -> (Key<EdgeKey>, Key<EdgeKey>) {
        let he1_key = self.edges.reserve();
        let he2_key = self.edges.reserve();

        self.vertices[v1].edge.get_or_insert(he1_key);
        self.vertices[v2].edge.get_or_insert(he2_key);

        let face = None;

        self.edges.set(
            he1_key,
            Edge {
                origin: v1,
                twin: he2_key,
                prev: he2_key,
                next: he2_key,
                face,
                weight: w2,
            },
        );
        self.edges.set(
            he2_key,
            Edge {
                origin: v2,
                twin: he1_key,
                prev: he1_key,
                next: he1_key,
                face,
                weight: w1,
            },
        );

        (he1_key, he2_key)
    }

    pub fn sort_around(&self, [cx, cy]: [Precision; 2], buffer: &mut [Key<EdgeKey>])
    where
        V: Coordinate,
    {
        buffer.sort_unstable_by(|a, b| {
            let at = self.edges[*a].twin;
            let bt = self.edges[*b].twin;

            let ao = self.edges[at].origin;
            let bo = self.edges[bt].origin;

            let [x1, y1] = self.vertices[ao].weight.xy();
            let [x2, y2] = self.vertices[bo].weight.xy();
            sort_clockwise([cx, cy], [x1, y1], [x2, y2])
        });
    }

    fn splice_prev(&mut self, edge: Key<EdgeKey>, local_prev: Key<EdgeKey>) {
        let twin = self.edges[local_prev].twin;
        self.edges[twin].next = edge;
        self.edges[edge].prev = twin;
    }

    fn splice_next(&mut self, edge: Key<EdgeKey>, local_next: Key<EdgeKey>) {
        let twin = self.edges[edge].twin;
        self.edges[twin].next = local_next;
        self.edges[local_next].prev = twin;
    }

    fn splice_edge(
        &mut self,
        edge: Key<EdgeKey>,
        local_prev: Key<EdgeKey>,
        local_next: Key<EdgeKey>,
    ) {
        self.splice_prev(edge, local_prev);
        self.splice_next(edge, local_next);
    }

    fn find_prev_next(
        &self,
        c: [Precision; 2],
        origin: Key<VertexKey>,
        edge: Key<EdgeKey>,
        sort_buffer: &mut Vec<Key<EdgeKey>>,
    ) -> Result<(Key<EdgeKey>, Key<EdgeKey>), Error>
    where
        V: Coordinate,
    {
        sort_buffer.clear();

        sort_buffer.extend(Traverser::around(&self, origin)?);

        if let Some(origin_incident_edge) = self.vertices[origin].edge {
            if origin_incident_edge != edge {
                sort_buffer.push(edge);
            }
        }

        self.sort_around(c, sort_buffer);

        let i = sort_buffer
            .iter()
            .enumerate()
            .find(|(_, v)| **v == edge)
            .unwrap()
            .0;

        let prev = if i == 0 {
            sort_buffer[sort_buffer.len() - 1]
        } else {
            sort_buffer[i - 1]
        };

        let next = sort_buffer[(i + 1) % sort_buffer.len()];

        Ok((prev, next))
    }

    pub fn propagate_face(
        &mut self,
        edge: Key<EdgeKey>,
        face: Option<Key<FaceKey>>,
    ) -> Result<(), Error> {
        let face = face.or_else(|| self.edges[edge].face);
        self.edges[edge].face = face;

        let mut traverser = Traverser::new(&self, edge)?;

        traverser.next(&self);

        while !traverser.is_at_start() {
            self.edges[traverser.edge()].face = face;
            traverser.next(&self);
        }

        Ok(())
    }

    fn rebuild_faces(
        &mut self,
        dirty_vertices: &[Key<VertexKey>],
        dirty_edges: &[Key<EdgeKey>],
        removed_edges: &[Key<EdgeKey>],
        delta: &mut DcelDelta,
    ) -> Result<(), Error>
    where
        V: Coordinate,
    {
        // Algorithm is as follow
        // give dirty vertices, dirty_edges and removed_edges repair faces local around the changes
        // if the dirty edge is a spur, its not part of a face
        // if the dirty edge is already part of a face and isn't a spur we need to check the loop orientation of those faces

        for root_edge in dirty_edges.iter().copied() {
            let mut traverser = Traverser::new(&self, root_edge)?;
            traverser.next(&self);

            let Edge {
                face: root_face,
                prev: root_prev,
                next: root_next,
                twin: root_twin,
                ..
            } = self.edges[root_edge];

            let root_next_face = self.edges[root_next].face;
            let root_prev_face = self.edges[root_prev].face;
            let root_twin_face = self.edges[root_twin].face;

            let mut num_spurs = 0;

            let mut area = ShoeString::default();

            let mut traverser = Traverser::new(&self, root_edge)?;
            traverser.next(&self);
            let mut num_edges = 1;
            loop {
                let edge = traverser.edge();
                let Edge {
                    face,
                    prev,
                    next,
                    twin,
                    ..
                } = &self.edges[edge];

                num_edges += 1;

                let v1 = self.vertex(self.edge(edge).origin);
                let v2 = self.vertex(self.edge(*twin).origin);

                area.add(&v1.weight, &v2.weight);

                if next == twin || prev == twin {
                    num_spurs += 1;
                }

                traverser.next(&self);
                if traverser.is_at_start() {
                    break;
                }
            }

            if root_face.is_none() {
                //
            }

            match (root_face, root_next_face, root_prev_face) {
                (Some(_root_face), None, None) => {}
                (maybe_root_face, Some(next_face), Some(prev_face)) if next_face == prev_face => {
                    let root_face = match maybe_root_face {
                        Some(root_face) => &self.faces[root_face],
                        None => {
                            self.edges[root_edge].face = Some(next_face);
                            &self.faces[next_face]
                        }
                    };

                    let root_area = root_face.signed_area;

                    let loop_area = area.area();

                    // create a new face if the loop area is different and its going ccw (area > 0.)
                    if root_area.abs() != loop_area.abs() && loop_area > 0. {
                        let face = self
                            .faces
                            .insert(Face::default_at_edge(root_edge, root_area));
                        self.propagate_face(root_edge, Some(face))?;
                        delta.faces_created.push(face);
                        self.faces[face].signed_area = loop_area;
                    }
                }
                (Some(root_face), Some(next_face), Some(prev_face)) => {
                    let root_area = area.area();
                    let other_face = if root_face == next_face {
                        prev_face
                    } else if root_face == prev_face {
                        next_face
                    } else {
                        panic!("something odd happened")
                    };

                    let other_area = self.face(other_face).signed_area;

                    let (remove, prop) = if root_area.abs() > other_area {
                        (prev_face, root_face)
                    } else {
                        (root_face, prev_face)
                    };
                    self.faces.remove(remove);
                    self.propagate_face(root_edge, Some(prop))?;
                    delta.faces_removed.push(remove);
                }
                (None, None, None) => {
                    let face = self
                        .faces
                        .insert(Face::default_at_edge(root_edge, area.area()));
                    self.propagate_face(root_edge, Some(face))?;
                    delta.faces_created.push(face);
                }

                (None, Some(next_face), Some(prev_face)) => {}
                (None, Some(next_face), None) => {
                    self.edges[root_edge].face = Some(next_face);
                }
                (None, None, Some(prev_face)) => {
                    self.edges[root_edge].face = Some(prev_face);
                }
                _ => {
                    todo!()
                }
            }
        }

        Ok(())
    }

    // patch a local ordering, assumes that twin references edges are valid (which they should be)
    fn patch_local_ordering(
        &mut self,
        around: Key<VertexKey>,
        edges: &mut [Key<EdgeKey>],
    ) -> Result<(), Error>
    where
        V: Coordinate,
    {
        let n = edges.len();
        if n == 0 {
            return Ok(());
        }

        let c = self
            .vertices
            .get(around)
            .ok_or(Error::VertexDoesNotExist)?
            .weight
            .xy();

        self.sort_around(c, edges);

        let n = edges.len();
        for (i, edge) in edges.iter().enumerate() {
            let local_prev = edges[(i + n - 1) % n];
            let local_next = edges[(i + 1) % n];

            self.splice_edge(*edge, local_prev, local_next);

            self.edges[*edge].origin = around;
        }
        self.vertices[around].edge = edges.first().copied();

        Ok(())
    }

    pub fn merge_vertices(
        &mut self,
        keep: Key<VertexKey>,
        delete: Key<VertexKey>,
    ) -> Result<DcelDelta, Error>
    where
        V: Coordinate,
    {
        let mut sort_buffer: Vec<Key<EdgeKey>> = vec![];

        let c = self.vertices[keep].weight.xy();

        let mut dirty_edges = vec![];

        for edge in Traverser::around(self, delete)? {
            let twin = self.edges[edge].twin;
            sort_buffer.push(edge);
            dirty_edges.extend_from_slice(&[edge, twin]);
        }

        sort_buffer.extend(Traverser::around(self, keep)?);

        self.patch_local_ordering(keep, &mut sort_buffer)?;

        self.vertices.remove(delete);

        let mut delta = DcelDelta {
            vertices_removed: vec![delete],
            ..Default::default()
        };

        sort_buffer.clear();

        self.rebuild_faces(&[keep], &dirty_edges, &[], &mut delta)?;

        Ok(delta)
    }

    pub fn link_vertices_default(
        &mut self,
        v1: Key<VertexKey>,
        v2: Key<VertexKey>,
    ) -> Result<DcelDelta, Error>
    where
        E: Default,
        V: Coordinate,
    {
        self.link_vertices(v1, v2, E::default(), E::default())
    }

    pub fn link_vertices(
        &mut self,
        v1: Key<VertexKey>,
        v2: Key<VertexKey>,
        w1: E,
        w2: E,
    ) -> Result<DcelDelta, Error>
    where
        V: Coordinate,
    {
        let (oc, dc) = {
            let origin = self.vertices.get(v1).ok_or(VertexDoesNotExist)?;
            let destination = self.vertices.get(v2).ok_or(VertexDoesNotExist)?;
            (origin.weight.xy(), destination.weight.xy())
        };

        let (he1, he2) = self.make_half_edges(v1, v2, w1, w2);

        let mut sort_buffer: Vec<Key<EdgeKey>> = vec![];

        let (he1_prev, he1_next) = self.find_prev_next(oc, v1, he1, &mut sort_buffer)?;

        let (he2_prev, he2_next) = self.find_prev_next(dc, v2, he2, &mut sort_buffer)?;

        self.splice_edge(he1, he1_prev, he1_next);
        self.splice_edge(he2, he2_prev, he2_next);

        sort_buffer.extend(Traverser::around(&self, v1)?.chain(Traverser::around(&self, v2)?));

        let mut delta = DcelDelta {
            edges_created: vec![he1, he2],
            ..Default::default()
        };

        self.rebuild_faces(&[v1, v2], &[he1, he2], &[], &mut delta)?;

        Ok(delta)
    }

    pub fn split_edge(&mut self, edge: Key<EdgeKey>) {}

    pub fn merge_create_edge(
        &mut self,
        other: Dcel<V, E, F>,
        from: Key<VertexKey>,
        to: Key<VertexKey>,
    ) {
    }

    pub fn merge_share_edge(&mut self, other: Dcel<V, E, F>, edge: Key<EdgeKey>) {}

    pub fn from_edge_pairs(pairs: Vec<u32>, positions: Vec<Precision>) -> Self
    where
        V: Coordinate + FromCoordinate,
        E: Default,
        F: Default,
    {
        // - TODO pass pairs in the format of [{origin_idx},{num_edges},...{destination_idx}] ideally presorted on the client
        // - TODO sort edges pairs in place without allocating another buffer
        // - TODO save an iteration of all vertices, do it in the main pair loop
        // - TODO save an iteration find minx boudning vertex in the main loop

        let mut dcel = Dcel::default();

        let num_vertices = positions.len() / 3;

        for i in 0..num_vertices {
            let offset = i * 3;
            let x = positions[offset];
            let y = positions[offset + 1];
            let z = positions[offset + 2];
            let v = V::from_xyz([x, y, z]);

            dcel.vertices.insert(Vertex {
                edge: None,
                weight: v,
            });
        }

        for pair in pairs.chunks_exact(2) {
            let v1_idx = pair[0] as usize;
            let v2_idx = pair[1] as usize;

            let v1 = dcel.vertices.key((v1_idx + 1) as u32).unwrap();
            let v2 = dcel.vertices.key((v2_idx + 1) as u32).unwrap();

            dcel.make_half_edges(v1, v2, E::default(), E::default());
        }

        let mut sorted_edges: Vec<(Key<VertexKey>, Key<EdgeKey>)> = dcel
            .edges
            .iter()
            .map(|(edge, key)| (edge.origin, key))
            .collect();
        sorted_edges.sort_unstable_by_key(|(origin, _)| *origin);

        // let mut local_sort = std::collections::HashMap::<Key, (Key, Key)>::new();

        let mut sort_buffer: Vec<Key<EdgeKey>> = vec![];
        for edges in sorted_edges.chunk_by_mut(|a, b| a.0 == b.0) {
            let [cx, cy] = dcel.vertices.get(edges[0].0).unwrap().weight.xy();
            sort_buffer.clear();
            sort_buffer.extend(edges.iter().map(|(_, edge)| *edge));
            dcel.sort_around([cx, cy], &mut sort_buffer);

            for (i, &(origin_key, edge_key)) in edges.iter().enumerate() {
                if i == 0 {
                    // the incident edge is always the one sorted clockwise
                    dcel.vertices[origin_key].edge = Some(edge_key);
                };

                let next_index = (i + 1) % edges.len();

                let (_, local_next_edge_key) = edges[next_index];

                dcel.splice_next(edge_key, local_next_edge_key);
            }
        }

        let mut min_x = std::f32::INFINITY;
        let mut min_x_key: Option<Key<VertexKey>> = None;
        for (vertex, key) in dcel.vertices.iter() {
            if vertex.edge.is_none() {
                continue;
            }
            let x = vertex.weight.x();
            if x < min_x {
                min_x = x;
                min_x_key = Some(key);
            }
        }

        let min_x_key = match min_x_key {
            Some(v) => v,
            None => {
                // easy, there are no faces because there are no edges.
                return dcel;
            }
        };

        let min_x_bounding_edge = dcel.vertices[min_x_key].edge.unwrap();

        // all edges have a prev
        let bounding_face_start = min_x_bounding_edge;

        dcel.faces.insert(Face {
            signed_area: 0.,
            holes: vec![],
            edge: bounding_face_start,
            weight: F::default(),
        });

        dcel
    }
}
