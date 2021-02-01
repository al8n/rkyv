use core::fmt;
use std::error::Error;
use super::{ArchivedBox, ArchivedString, ArchivedVec};
use crate::{
    core_impl::SliceAugment,
    validation::{
        ArchiveBoundsContext,
        ArchiveMemoryContext,
    },
    ArchivePtr,
    RelPtr,
};
use bytecheck::{CheckBytes, CheckLayout};

#[derive(Debug)]
pub enum OwnedPointerError<T, R, C> {
    PointerCheckBytesError(T),
    ValueCheckBytesError(R),
    ContextError(C),
}

impl<T: fmt::Display, R: fmt::Display, C: fmt::Display> fmt::Display for OwnedPointerError<T, R, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnedPointerError::PointerCheckBytesError(e) => e.fmt(f),
            OwnedPointerError::ValueCheckBytesError(e) => e.fmt(f),
            OwnedPointerError::ContextError(e) => e.fmt(f),
        }
    }
}

impl<T: Error + 'static, R: Error + 'static, C: Error + 'static> Error for OwnedPointerError<T, R, C> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OwnedPointerError::PointerCheckBytesError(e) => Some(e as &dyn Error),
            OwnedPointerError::ValueCheckBytesError(e) => Some(e as &dyn Error),
            OwnedPointerError::ContextError(e) => Some(e as &dyn Error),
        }
    }
}

impl<C: ArchiveBoundsContext + ArchiveMemoryContext + ?Sized> CheckBytes<C> for ArchivedString
where
    C::Error: Error,
{
    type Error = OwnedPointerError<<SliceAugment as CheckBytes<C>>::Error, <str as CheckBytes<C>>::Error, C::Error>;

    unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
        let rel_ptr = RelPtr::<str>::manual_check_bytes(value.cast(), context)
            .map_err(OwnedPointerError::PointerCheckBytesError)?;
        let data = context.check_rel_ptr(rel_ptr.base(), rel_ptr.offset())
            .map_err(OwnedPointerError::ContextError)?;
        let ptr = str::augment_ptr(data, rel_ptr.augment());
        let layout = str::layout(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        context.claim_bytes(ptr.cast(), layout.size())
            .map_err(OwnedPointerError::ContextError)?;
        <str as CheckBytes<C>>::check_bytes(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        Ok(&*value)
    }
}

impl<T: ArchivePtr + CheckLayout<C> + ?Sized, C: ArchiveBoundsContext + ArchiveMemoryContext + ?Sized> CheckBytes<C> for ArchivedBox<T>
where
    T::Augment: CheckBytes<C>,
    C::Error: Error,
{
    type Error = OwnedPointerError<<T::Augment as CheckBytes<C>>::Error, T::Error, C::Error>;

    unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
        let rel_ptr = RelPtr::<T>::manual_check_bytes(value.cast(), context)
            .map_err(OwnedPointerError::PointerCheckBytesError)?;
        let data = context.check_rel_ptr(rel_ptr.base(), rel_ptr.offset())
            .map_err(OwnedPointerError::ContextError)?;
        let ptr = T::augment_ptr(data, rel_ptr.augment());
        let layout = T::layout(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        context.claim_bytes(ptr.cast(), layout.size())
            .map_err(OwnedPointerError::ContextError)?;
        T::check_bytes(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        Ok(&*value)
    }
}

impl<T: CheckLayout<C>, C: ArchiveBoundsContext + ArchiveMemoryContext + ?Sized> CheckBytes<C> for ArchivedVec<T>
where
    [T]: ArchivePtr,
    <[T] as ArchivePtr>::Augment: CheckBytes<C>,
    C::Error: Error,
{
    type Error = OwnedPointerError<<<[T] as ArchivePtr>::Augment as CheckBytes<C>>::Error, <[T] as CheckBytes<C>>::Error, C::Error>;

    unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
        let rel_ptr = RelPtr::<[T]>::manual_check_bytes(value.cast(), context)
            .map_err(OwnedPointerError::PointerCheckBytesError)?;
        let data = context.check_rel_ptr(rel_ptr.base(), rel_ptr.offset())
            .map_err(OwnedPointerError::ContextError)?;
        let ptr = <[T]>::augment_ptr(data, rel_ptr.augment());
        let layout = <[T]>::layout(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        context.claim_bytes(ptr.cast(), layout.size())
            .map_err(OwnedPointerError::ContextError)?;
        <[T]>::check_bytes(ptr, context)
            .map_err(OwnedPointerError::ValueCheckBytesError)?;
        Ok(&*value)
    }
}
