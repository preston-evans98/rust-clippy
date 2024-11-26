use std::ops::{Deref, DerefMut};
// We really do want a Drop bound here because we lint by searching for
// drop(T) in the MIR. If at type is not `Drop`, these calls may be absent
// leading to false negatives.
#[allow(drop_bounds)]
pub trait NearlyLinear: Drop {
    type Inner;
    fn done(self) -> Self::Inner;
}

union DropGuardHelper<T> {
    drop_guard: std::mem::ManuallyDrop<DropGuard<T>>,
    contents: std::mem::ManuallyDrop<T>,
}

#[repr(transparent)]
pub struct DropGuard<T>(T);

impl<T> DropGuard<T> {
    pub fn new(item: T) -> Self {
        Self(item)
    }
}

impl<T> Deref for DropGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for DropGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Drop for DropGuard<T> {
    fn drop(&mut self) {
        // Do nothing
    }
}

impl<T: Sized> NearlyLinear for DropGuard<T> {
    type Inner = T;
    fn done(self) -> T {
        // Return the item without dropping the DropGuard
        // Safety: Since we use repr(transparent), this type is
        // guaranteed to be bit-for-bit identical to the underlying.
        // let inner = &self as *const _ as *const T;
        // unsafe { *inner }
        // <T as std::mem::TransmuteFrom<Self>>::transmute_from(self)
        // unsafe { std::mem::transmute::<Self, T>(self) }
        let as_union = DropGuardHelper {
            drop_guard: std::mem::ManuallyDrop::new(self),
        };
        std::mem::ManuallyDrop::into_inner(unsafe { as_union.contents })
        // let Self(inner) = self;
        // inner
    }
}
