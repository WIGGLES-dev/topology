use std::marker::PhantomData;

use crate::dcel::Flavor;

pub struct SpacialIndex<F: Flavor> {
    phantom: PhantomData<F>,
}

impl<F: Flavor> SpacialIndex<F> {}
