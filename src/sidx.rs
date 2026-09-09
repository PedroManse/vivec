pub trait SparselyIndexable {
    fn get_index(&self) -> usize;
}

#[derive(Debug)]
pub struct SparseIndex<T: SparselyIndexable> {
    data: Box<[Option<T>]>,
}

impl<T: SparselyIndexable + std::fmt::Debug> SparseIndex<T> {
    pub fn new(biggest_index: usize, reader: impl Iterator<Item = T>) -> Self {
        let mut sparse_data = Self::prepare(biggest_index);
        Self::populate(&mut sparse_data, reader);
        Self {
            data: sparse_data.into_boxed_slice(),
        }
    }
    pub unsafe fn new_big_last(data: Vec<T>) -> Self {
        Self::new(data.last().unwrap().get_index(), data.into_iter().rev())
    }
    pub unsafe fn new_big_first(data: Vec<T>) -> Self {
        Self::new(data.first().unwrap().get_index(), data.into_iter())
    }
    fn prepare(biggest_index: usize) -> Vec<Option<T>> {
        let mut sparse_data = Vec::with_capacity(biggest_index + 1);
        sparse_data.extend((0..=biggest_index).map(|_| None));
        sparse_data
    }
    fn populate(sparse_data: &mut Vec<Option<T>>, reader: impl Iterator<Item = T>) {
        for next_back in reader {
            let idx = next_back.get_index();
            sparse_data[idx] = Some(next_back);
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)?.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::{SparseIndex, SparselyIndexable};
    #[derive(Debug)]
    struct Entry(usize, String);
    impl Entry {
        pub fn new(i: usize) -> Self {
            Self(i, i.to_string())
        }
    }
    impl SparselyIndexable for Entry {
        fn get_index(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn sparse() {
        let si =
            unsafe { SparseIndex::new_big_last((0..32).map(|i| i * 2).map(Entry::new).collect()) };
        eprintln!("{si:?}");
    }
}
