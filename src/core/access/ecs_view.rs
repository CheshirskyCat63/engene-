use std::any::TypeId;
use std::marker::PhantomData;

pub struct Read<T>(PhantomData<T>);
pub struct Write<T>(PhantomData<T>);

pub trait ComponentAccess {
    fn type_id() -> TypeId;
    fn is_write() -> bool;
}

impl<T: 'static> ComponentAccess for Read<T> {
    fn type_id() -> TypeId { TypeId::of::<T>() }
    fn is_write() -> bool { false }
}

impl<T: 'static> ComponentAccess for Write<T> {
    fn type_id() -> TypeId { TypeId::of::<T>() }
    fn is_write() -> bool { true }
}
