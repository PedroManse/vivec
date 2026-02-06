use std::fmt::Debug;

/// Virtually Indexed Vector
/// if the feature `reverse_id_search` is enabled, it's possible to get the ID of a node from it's
/// index
#[derive(Debug)]
pub struct ViVec<T> {
    nodes: Vec<T>,
    id_to_index: Vec<usize>,
    #[cfg(feature = "reverse_id_search")]
    index_to_id: Vec<usize>,
}

impl<T> ViVec<T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            id_to_index: vec![],
            #[cfg(feature = "reverse_id_search")]
            index_to_id: vec![],
        }
    }

    pub fn used_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn unused_ids(&self) -> usize {
        self.id_to_index.len() - self.nodes.len()
    }

    #[cfg(feature = "reverse_id_search")]
    pub fn append(&mut self, data: T) {
        let unused_ids = self.unused_ids();
        let new_id = if unused_ids != 0 {
            // acquire unused id
            self.id_to_index[self.id_to_index.len() - unused_ids]
        } else {
            // push new id
            let new_id = self.id_to_index.len();
            self.id_to_index.push(new_id);
            new_id
        };
        self.index_to_id.push(new_id);
        self.nodes.push(data);
    }

    #[cfg(not(feature = "reverse_id_search"))]
    pub fn append(&mut self, data: T) {
        let unused_ids = self.unused_ids();
        if unused_ids == 0 {
            // push new id
            let new_id = self.id_to_index.len();
            self.id_to_index.push(new_id);
        }
        self.nodes.push(data);
    }

    pub fn swap(&mut self, id1: usize, id2: usize) {
        if id1 != id2 {
            assert!(id1 < self.nodes.len());
            assert!(id2 < self.nodes.len());
            self.nodes.swap(id1, id2);
            #[cfg(feature = "reverse_id_search")]
            self.index_to_id.swap(id1, id2);
            self.id_to_index.swap(id1, id2);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        // get last used id and remove it
        let pop_idx = *self.id_to_index.last()? - self.unused_ids();
        self.unlink(pop_idx)
    }

    /// # Data will be removed and id will be left dangling
    pub fn unlink(&mut self, pop_idx: usize) -> Option<T> {
        // get id of last node in data vec
        let swap_idx = *self.id_to_index.last()?;
        // send to-be-removed node to last place
        self.swap(pop_idx, swap_idx);
        // remove last node
        #[cfg(feature = "reverse_id_search")]
        self.index_to_id.pop()?;
        self.nodes.pop()
    }

    pub fn get_by_id(&self, id: usize) -> Option<&T> {
        let idx = *self.id_to_index.get(id)?;
        self.nodes.get(idx)
    }

    pub fn get_mut_by_id(&mut self, id: usize) -> Option<&mut T> {
        let idx = *self.id_to_index.get(id)?;
        self.nodes.get_mut(idx)
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&T> {
        self.nodes.get(idx)
    }

    pub fn get_mut_by_inxed(&mut self, idx: usize) -> Option<&mut T> {
        self.nodes.get_mut(idx)
    }

    pub fn get_index_by_id(&self, id: usize) -> Option<&usize> {
        self.id_to_index.get(id)
    }

    pub fn into_straight_vec(self) -> Vec<T> {
        self.nodes
    }
    pub fn into_ordered_vec(self) -> Vec<T> {
        todo!()
    }
    pub fn straight_iter(&self) -> std::slice::Iter<'_, T> {
        self.nodes.iter()
    }
    pub fn into_straight_iter(self) -> std::vec::IntoIter<T> {
        self.nodes.into_iter()
    }
    pub fn ordered_iter<'v>(&'v self) -> OrderedIter<'v, T> {
        OrderedIter {
            current_id: 0,
            list: &self,
        }
    }
    pub fn into_ordered_iter(self) -> OrderedIterator<T> {
        OrderedIterator { list: self }
    }
}

impl<T> From<Vec<T>> for ViVec<T> {
    fn from(value: Vec<T>) -> Self {
        let p: Vec<_> = vec![(); value.len()]
            .into_iter()
            .enumerate()
            .map(|(a, ())| a)
            .collect();
        ViVec {
            nodes: value,
            id_to_index: p.clone(),
            #[cfg(feature = "reverse_id_search")]
            index_to_id: p,
        }
    }
}

#[doc(alias = "GayIterator")]
pub struct OrderedIter<'v, T> {
    pub current_id: usize,
    pub list: &'v ViVec<T>,
}

impl<'v, T> Iterator for OrderedIter<'v, T> {
    type Item = &'v T;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = *self.list.get_index_by_id(self.current_id)?;
        let node = self.list.get_by_index(idx);
        self.current_id += 1;
        node.or_else(|| self.next())
    }
}

pub struct OrderedIterator<T> {
    pub list: ViVec<T>,
}

impl<T> Iterator for OrderedIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.unlink(0)
    }
}

#[doc(alias = "GayIterator")]
pub struct OrderedMutIter<'v, T> {
    pub current_id: usize,
    pub list: &'v mut ViVec<T>,
}
