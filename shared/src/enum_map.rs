pub use enum_mappable::Mappable;
use serde::{Deserialize, Serialize};

/// A generic EnumMap for any enum that implements `EnumIter`.
#[derive(Clone, Serialize, Deserialize)]
pub struct EnumMap<E, T>
where
    E: EnumIter,
{
    data: Vec<T>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E, T> EnumMap<E, T>
where
    E: EnumIter,
{
    /// Create the map by calling `mapping` on each variant.
    pub fn new<F>(mapping: F) -> Self
    where
        F: Fn(E) -> T,
    {
        let data = E::all_variants().iter().map(|&e| mapping(e)).collect();

        Self {
            data,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Retrieve a reference to the value stored for variant `e`.
    pub fn get(&self, e: E) -> &T {
        &self.data[e.as_index()]
    }
}

/// A trait to represent "an enum you can iterate over".
pub trait EnumIter: Copy + 'static {
    /// The total number of variants in this enum.
    const COUNT: usize;

    /// Return an array of all variants.
    fn all_variants() -> &'static [Self];

    /// Maps `Self` into an index (0..COUNT).
    fn as_index(&self) -> usize;
}

impl<E, T> EnumMap<E, T>
where
    E: EnumIter,
{
    /// Iterate over `&T`
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    /// Iterate over `&mut T`
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    /// Iterate over `(E, &T)`
    pub fn iter_enums(&self) -> impl Iterator<Item = (E, &T)> {
        E::all_variants().iter().copied().zip(self.data.iter())
    }

    /// Iterate over `(E, &mut T)`
    pub fn iter_enums_mut(&mut self) -> impl Iterator<Item = (E, &mut T)> {
        E::all_variants().iter().copied().zip(self.data.iter_mut())
    }
}

// `for value in enum_map` moves out the values
impl<E, T> IntoIterator for EnumMap<E, T>
where
    E: EnumIter,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// `for &value in &enum_map`
impl<'a, E, T> IntoIterator for &'a EnumMap<E, T>
where
    E: EnumIter,
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

// `for &mut value in &mut enum_map`
impl<'a, E, T> IntoIterator for &'a mut EnumMap<E, T>
where
    E: EnumIter,
{
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}
