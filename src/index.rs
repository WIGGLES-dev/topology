pub trait SpatialIndex {
    fn locate_all_at_point(&mut self);
    fn locate_all_at_point_int(&mut self);
    fn locate_all_at_point_int_mut(&mut self);
    fn locate_in_envelope(&mut self);
    fn locate_in_envelope_mut(&mut self);
    fn locate_in_envelope_int(&mut self);
    fn locate_in_envelope_int_mut(&mut self);
    fn drain_in_envelope(&mut self);

    fn locate_in_envelope_intersecting(&mut self);
    fn locate_in_envelope_intersecting_mut(&mut self);
    fn locate_in_envelope_intersecting_int(&mut self);
    fn locate_in_envelope_intersecting_int_mut(&mut self);
    fn drain_in_envelope_intersecting(&mut self);

    fn remove(&mut self);
    fn remove_at_point(&mut self);

    fn contains(&mut self);

    fn nearest_neighbor(&mut self);
    fn nearest_neighbors(&mut self);

    fn locate_within_distance(&mut self);
    fn drain_within_distance(&mut self);

    fn nearest_neighbors_iter(&mut self);
    fn nearest_neighbor_iter_with_distance(&mut self);
    fn pop_nearest_neighbor(&mut self);

    fn remove_with_selection_function(&mut self);
    fn drain_with_selection_function(&mut self);
}
