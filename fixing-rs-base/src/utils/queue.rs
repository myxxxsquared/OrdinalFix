pub struct QueueItemIndex {
    outer_idx: usize,
    inner_idx: usize,
}

impl QueueItemIndex {
    fn new() -> Self {
        Self {
            outer_idx: 0,
            inner_idx: 0,
        }
    }
}

pub struct QueueItem<T> {
    storage: Vec<Vec<T>>,
    current_index: QueueItemIndex,
}

impl<T> QueueItem<T> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            current_index: QueueItemIndex::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        if self.storage.len() == 0 || self.storage.last().unwrap().len() >= Self::chunk_max_size() {
            self.storage
                .push(Vec::with_capacity(Self::chunk_max_size()))
        }
        self.storage.last_mut().unwrap().push(value);
    }

    pub const fn chunk_max_size() -> usize {
        1024 * 1024
    }

    pub fn index_from_begin(&self) -> QueueItemIndex {
        QueueItemIndex::new()
    }
}

impl<T: Copy> QueueItem<T> {
    fn next_inner(storage: &Vec<Vec<T>>, index: &mut QueueItemIndex) -> Option<T> {
        match storage.get(index.outer_idx) {
            Some(inner_vec) => match inner_vec.get(index.inner_idx) {
                Some(v) => {
                    index.inner_idx += 1;
                    if index.inner_idx >= inner_vec.len()
                        && storage.len() > 0
                        && index.outer_idx < storage.len() - 1
                    {
                        index.outer_idx += 1;
                        index.inner_idx = 0;
                    }
                    Some(*v)
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn queue_next(&mut self) -> Option<T> {
        Self::next_inner(&self.storage, &mut self.current_index)
    }

    pub fn get_next(&self, index: &mut QueueItemIndex) -> Option<T> {
        Self::next_inner(&self.storage, index)
    }
}

pub struct Queue<T> {
    items: Vec<QueueItem<T>>,
}

impl<T> Queue<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            items: (0..=max_length).map(|_| QueueItem::new()).collect(),
        }
    }

    pub fn push(&mut self, value: T, length: usize) {
        self.items[length].push(value)
    }

    pub fn index_from_begin(&self, length: usize) -> QueueItemIndex {
        self.items[length].index_from_begin()
    }
}

impl<T: Copy> Queue<T> {
    pub fn queue_next(&mut self, length: usize) -> Option<T> {
        self.items[length].queue_next()
    }

    pub fn get_next(&self, index: &mut QueueItemIndex, length: usize) -> Option<T> {
        self.items[length].get_next(index)
    }
}
