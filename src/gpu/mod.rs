use std::sync::Arc;

use crate::geometry::Hittable;
pub mod upload;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectKey(usize);
impl ObjectKey {
    pub fn of(obj: &Arc<dyn Hittable>) -> Self {
        Self(Arc::as_ptr(obj) as *const () as usize)
    }
}
