use super::mvec::{MVec, MVecIterMut};
use dashmap::{mapref::entry::Entry, DashMap};
use futures_util::task::AtomicWaker;
use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::task::{Context, Poll};

/// A registry of channels.
#[derive(Debug)]
pub(crate) struct Registry<K, V>
where
    K: Eq + Hash,
{
    map: DashMap<K, MVec<V>>,
    key_set_change_waker: AtomicWaker,
    key_set_change_ready: AtomicBool,
}

/// Newtype to protect access to the registry ID.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ID(usize);

#[derive(Debug)]
pub enum RegistrationEffect {
    NewName,
    ExistingName,
}

#[derive(Debug)]
pub enum UnregistrationEffect {
    NameErased,
    NamePreserved,
}

impl<K, V> Registry<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
            key_set_change_waker: AtomicWaker::new(),
            key_set_change_ready: AtomicBool::new(false),
        }
    }

    pub fn register(&self, name: K, value: V) -> (ID, RegistrationEffect) {
        let entry = self.map.entry(name);

        let effect = match &entry {
            Entry::Vacant(_) => RegistrationEffect::NewName,
            Entry::Occupied(_) => RegistrationEffect::ExistingName,
        };

        let mut val = entry.or_default();

        let mvec = val.value_mut();
        let id = mvec.counter();
        mvec.push(value);

        (ID(id), effect)
    }

    pub fn unregister<Q: ?Sized>(&mut self, name: &Q, id: ID) -> Option<(V, UnregistrationEffect)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let ID(id) = id;

        let mut value = self.map.get_mut(name)?;

        let mvec = value.value_mut();
        let removed = mvec.remove(id)?;

        let effect = if mvec.is_empty() {
            self.map.remove(name);
            UnregistrationEffect::NameErased
        } else {
            UnregistrationEffect::NamePreserved
        };

        Some((removed, effect))
    }

    pub fn get_iter_mut<'a, Q: ?Sized>(&'a mut self, name: &Q) -> Option<MVecIterMut<'a, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map
            .get_mut(name)
            .map(|mut v| MVec::iter_mut(v.value().clone()))
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.iter().map(|v| v.key())
    }

    fn poll_key_set_change(&self, cx: &mut Context) -> Poll<()> {
        self.key_set_change_waker.register(cx.waker());

        if self
            .key_set_change_ready
            .compare_and_swap(true, false, Relaxed)
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn key_set_change_notify(&self) {
        self.key_set_change_ready.store(true, Relaxed);
        self.key_set_change_waker.wake();
    }
}
