use std::any::TypeId;
use std::marker::PhantomData;

pub struct ReadResource<T>(PhantomData<T>);
pub struct WriteResource<T>(PhantomData<T>);
pub struct Emit<E>(PhantomData<E>);

pub trait ResourceAccess {
    fn type_id() -> TypeId;
    fn is_write() -> bool;
}

impl<T: 'static> ResourceAccess for ReadResource<T> {
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
    fn is_write() -> bool {
        false
    }
}

impl<T: 'static> ResourceAccess for WriteResource<T> {
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
    fn is_write() -> bool {
        true
    }
}
