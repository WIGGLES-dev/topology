pub trait Traverser: DoubleEndedIterator {
    fn next(&mut self);
    fn prev(&mut self);
}

pub trait OrderedTraverser {
    fn local_next(&mut self);
    fn local_prev(&mut self);
}
