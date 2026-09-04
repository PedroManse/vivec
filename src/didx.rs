pub trait DenslyIndexable {
    type Key: std::cmp::Ord;
    fn get_key(&self) -> Self::Key;
}

pub struct DenseIndex<T: DenslyIndexable> {
    data: Box<[T]>,
    keys: Box<[T::Key]>,
}

impl<T: DenslyIndexable> DenseIndex<T> {
    pub fn new(mut data: Box<[T]>) -> Self {
        data.sort_by_key(T::get_key);
        let keys = data.iter().map(T::get_key).collect();
        Self { data, keys }
    }

    pub fn get(&self, key: &T::Key) -> Option<&T> {
        let idx = self.keys.binary_search(key).ok()?;
        self.data.get(idx)
    }

    pub fn unordered_iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}
