use std::ptr::NonNull;

pub struct Tru;

pub struct Fls;

pub struct Carrier<T: ?Sized>(NonNull<T>);

unsafe impl<T: ?Sized> Send for Carrier<T> {}

#[allow(clippy::wrong_self_convention)]
impl<T: ?Sized> Carrier<T> {
    pub const unsafe fn new(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }
    pub const fn as_ptr(self) -> *const T {
        self.0.as_ptr()
    }
    pub const fn as_mut_ptr(self) -> *mut T {
        self.0.as_ptr()
    }
    #[allow(clippy::missing_const_for_fn)]
    pub unsafe fn as_ref<'a>(self) -> &'a T {
        &*self.0.as_ptr()
    }
    pub unsafe fn as_mut<'a>(self) -> &'a mut T {
        &mut *self.0.as_ptr()
    }
}
