use core::panic;
use std::{borrow::Cow, cmp::Ordering, collections::BTreeSet, marker::PhantomData};

use crate::{
    arena::Key,
    coord::{Coordinate, Precision, sort_clockwise},
    dcel::{Dcel, EdgeKey, Traverser, VertexKey, error::Error, flavor::Flavor},
};

pub struct Linker<F: Flavor> {
    sort_buffer: Vec<Key<EdgeKey>>,
    phantom: PhantomData<F>,
}

impl<F: Flavor> Linker<F> {
    pub fn new() -> Self {
        Self {
            sort_buffer: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn follow(dcel: &mut Dcel<F>, prev: Key<EdgeKey>, next: Key<EdgeKey>) {
        dcel.edge_mut(next).prev = prev;
        dcel.edge_mut(prev).next = next;
    }

    pub(crate) fn splice_prev(dcel: &mut Dcel<F>, edge: Key<EdgeKey>, local_prev: Key<EdgeKey>) {
        let twin = dcel.edges[local_prev].twin;
        dcel.edges[twin].next = edge;
        dcel.edges[edge].prev = twin;
    }

    pub(crate) fn splice_next(dcel: &mut Dcel<F>, edge: Key<EdgeKey>, local_next: Key<EdgeKey>) {
        let twin = dcel.edges[edge].twin;
        dcel.edges[twin].next = local_next;
        dcel.edges[local_next].prev = twin;
    }

    pub(crate) fn splice_edge(
        dcel: &mut Dcel<F>,
        edge: Key<EdgeKey>,
        local_prev: Key<EdgeKey>,
        local_next: Key<EdgeKey>,
    ) {
        Self::splice_prev(dcel, edge, local_prev);
        Self::splice_next(dcel, edge, local_next);
    }

    /// Atomically link two vertices in their clockwise cyclic traversal order. THIS WILL INVALIDATE FACE INVARIANTS
    pub fn splice_edges(dcel: &mut Dcel<F>, edges: [(Key<EdgeKey>, Key<VertexKey>); 2])
    where
        F::Vertex: Coordinate,
    {
        let [(outgoing, oo), (incoming, io)] = edges;
        let [outgoing_prev, outgoing_next] = Self::find_prev_next(dcel, oo, io);
        let [incoming_prev, incoming_next] = Self::find_prev_next(dcel, io, oo);
        Self::splice_edge(dcel, outgoing, outgoing_prev, outgoing_next);
        Self::splice_edge(dcel, incoming, incoming_prev, incoming_next);
    }

    /// update points of 2 edges in prepartion of removal, update vertex origins
    pub fn unsplice_edge(dcel: &mut Dcel<F>, edges: [Key<EdgeKey>; 2]) {
        //      a
        //    / | \
        //   b  |  d
        //    \ | /
        //      c
        //
        let [e1, e2] = edges;

        // fix vertex references
        for e in edges {
            let eo = e.origin(dcel);
            if eo.edge(dcel) == Some(e) {
                let alt = e.twin(dcel).next(dcel);
                dcel.vertex_mut(eo).edge = if alt == e { None } else { Some(alt) };
            }
        }

        let a = e1.prev(dcel);
        let b = e2.next(dcel);
        let c = e2.prev(dcel);
        let d = e1.next(dcel);

        println!("{a} follows {b}");
        println!("{c} follows {d}");

        Self::follow(dcel, a, b);
        Self::follow(dcel, c, d);
    }

    pub fn sort_around(dcel: &mut Dcel<F>, [cx, cy]: [Precision; 2], buffer: &mut [Key<EdgeKey>])
    where
        F::Vertex: Coordinate,
    {
        buffer.sort_unstable_by(|a, b| {
            let at = dcel.edges[*a].twin;
            let bt = dcel.edges[*b].twin;

            let ao = dcel.edges[at].origin;
            let bo = dcel.edges[bt].origin;

            let [x1, y1] = dcel.vertices[ao].weight.xy();
            let [x2, y2] = dcel.vertices[bo].weight.xy();
            sort_clockwise([cx, cy], [x1, y1], [x2, y2])
        });
    }

    pub fn find_prev_next(
        dcel: &Dcel<F>,
        center: Key<VertexKey>,
        reference: Key<VertexKey>,
    ) -> [Key<EdgeKey>; 2]
    where
        F::Vertex: Coordinate,
    {
        let [cx, cy] = center.weight(dcel).xy();
        let [rx, ry] = reference.weight(dcel).xy();

        // Create the traverser and grab the very first edge
        let mut trav = Traverser::at(dcel, center).unwrap();
        let first = trav.edge();

        let [x1, y1] = first.twin(dcel).origin(dcel).weight(dcel).xy();
        let ordering = sort_clockwise([cx, cy], [x1, y1], [rx, ry]);

        let mut last = first;

        loop {
            trav.local_next(dcel);
            if trav.is_at_start() {
                return [last, trav.edge()];
            }

            let [x1, y1] = trav.edge().twin(dcel).origin(dcel).weight(dcel).xy();
            let new_ordering = sort_clockwise([cx, cy], [x1, y1], [rx, ry]);

            if new_ordering != Ordering::Equal && new_ordering != ordering {
                return [last, trav.edge()];
            }

            last = trav.edge();
        }
    }

    fn find_prev_next_sort(
        dcel: &mut Dcel<F>,
        c: [Precision; 2],
        origin: Key<VertexKey>,
        edge: Key<EdgeKey>,
        sort_buffer: &mut Vec<Key<EdgeKey>>,
    ) -> Result<(Key<EdgeKey>, Key<EdgeKey>), Error>
    where
        F::Vertex: Coordinate,
    {
        sort_buffer.clear();

        sort_buffer.extend(Traverser::around(dcel, origin)?);

        let origin_incident_edge = dcel.vertices[origin].edge;

        if origin_incident_edge != Some(edge) {
            sort_buffer.push(edge);
        }

        Self::sort_around(dcel, c, sort_buffer);

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

    pub(crate) fn patch_local_ordering(
        dcel: &mut Dcel<F>,
        around: Key<VertexKey>,
        edges: &mut [Key<EdgeKey>],
    ) -> Result<(), Error>
    where
        F::Vertex: Coordinate,
    {
        let n = edges.len();
        if n == 0 {
            return Ok(());
        }

        let c = dcel
            .vertices
            .get(around)
            .ok_or(Error::VertexDoesNotExist)?
            .weight
            .xy();

        Self::sort_around(dcel, c, edges);

        let n = edges.len();
        for (i, edge) in edges.iter().enumerate() {
            let local_prev = edges[(i + n - 1) % n];
            let local_next = edges[(i + 1) % n];

            Self::splice_edge(dcel, *edge, local_prev, local_next);

            dcel.edges[*edge].origin = around;
        }
        dcel.vertices[around].edge = edges.first().copied();

        Ok(())
    }

    /// reparent the edges of vertex to origin, returning the first and last edge outgoing from origin that were newly added
    /// TODO: this could probably be done without allocations if we can accept some amount of fragility in which vertices need to be moved when uncollapsing an edge
    pub(crate) fn reparent_vertex(
        &mut self,
        dcel: &mut Dcel<F>,
        origin: Key<VertexKey>,
        vertex: Key<VertexKey>,
        only: Option<Vec<Key<EdgeKey>>>,
    ) -> Vec<Key<EdgeKey>>
    where
        F::Vertex: Coordinate,
    {
        self.sort_buffer.clear();

        self.sort_buffer
            .extend(Traverser::around(dcel, origin).unwrap());

        if let Some(only) = only {
            self.sort_buffer.extend(only.iter().copied());
            let Ok(around) = Traverser::around(dcel, vertex) else {
                return vec![];
            };

            // an ordered set of edge outbound from vertex, to dedupe
            let total = BTreeSet::from_iter(around);

            // the edges that we are moving from vertex to origin
            let to_move = BTreeSet::from_iter(only.iter().copied());

            // move targets to origin and patch
            self.sort_buffer.extend(only.iter().copied());
            Self::patch_local_ordering(dcel, origin, &mut self.sort_buffer).unwrap();

            // clear the buffer and append the elements that aren't being moved to resort vertex, the difference of total and to_move
            self.sort_buffer.clear();
            self.sort_buffer.extend(total.difference(&to_move));
            Self::patch_local_ordering(dcel, vertex, &mut self.sort_buffer).unwrap();
            return only;
        } else {
            println!("reparenting {vertex} {:?}", vertex.edge(dcel));
            let mut reparented = vec![];
            let around = match Traverser::around(dcel, vertex) {
                Ok(around) => {
                    for edge in around {
                        println!("reparenting {edge}");
                        self.sort_buffer.push(edge);
                        reparented.push(edge);
                    }
                }
                Err(Error::DisconnectedVertex) => {}
                Err(err) => panic!("{err}"),
            };

            Self::patch_local_ordering(dcel, origin, &mut self.sort_buffer).unwrap();
            dcel.vertex_mut(vertex).edge = None;

            return reparented;
        }
    }
}
