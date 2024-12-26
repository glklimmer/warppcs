pub use enum_mappable::Mappable;

/// A generic EnumMap for any enum that implements `EnumIter`.
#[derive(Clone)]
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
