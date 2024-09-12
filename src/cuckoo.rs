use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher, RandomState};

const MAX_LOOP: u8 = 100;

/// `CuckooHashTable` consists of two sets of buckets where an item `x`
/// can go to any of two buckets as long as there is an empty slot. The
/// downside as compared to standard hash table is that it requires two
/// independent hash functions.
pub struct CuckooHashTable<T> {
    buckets: [Vec<Option<T>>; 2],
    size: usize,
    capacity: usize,
    load_factor: f64,
    hash1: DefaultHasher,
    hash2: DefaultHasher,
}

impl<T: Hash + Clone + Eq> CuckooHashTable<T> {
    pub fn new() -> Self {
        let init_capacity: usize = 16;
        let rs1 = RandomState::new();
        let rs2 = RandomState::new();
        let h1 = rs1.build_hasher();
        let h2 = rs2.build_hasher();
        CuckooHashTable {
            buckets: [vec![None; init_capacity], vec![None; init_capacity]],
            capacity: init_capacity,
            size: 0,
            load_factor: 0.2,
            hash1: h1,
            hash2: h2,
        }
    }

    fn h1(&self, x: &T) -> usize {
        let mut hasher1 = self.hash1.clone();
        x.hash(&mut hasher1);
        let h1 = hasher1.finish() as usize;
        h1 % self.buckets[0].len()
    }

    fn h2(&self, x: &T) -> usize {
        let mut hasher2 = self.hash2.clone();
        x.hash(&mut hasher2);
        let h2 = hasher2.finish() as usize;
        h2 % self.buckets[1].len()
    }

    pub fn contains(&self, x: &T) -> bool {
        let b1 = self.h1(x);
        let b2 = self.h2(x);
        self.buckets[0][b1].as_ref() == Some(x) ||
            self.buckets[1][b2].as_ref() == Some(x)
    }

    pub fn remove(&mut self, x: &T) -> bool {
        let b1 = self.h1(x);
        if self.buckets[0][b1].as_ref() == Some(x) {
            self.buckets[0][b1] = None;
            self.size -= 1;
            return true;
        }
        let b2 = self.h2(x);
        if self.buckets[1][b2].as_ref() == Some(x) {
            self.buckets[1][b2] = None;
            self.size -= 1;
            return true;
        }
        return false;
    }

    pub fn insert(&mut self, x: T) -> bool {
        if self.contains(&x) {
            return false;
        }
        let b0 = self.h1(&x);
        if self.buckets[0][b0].is_none() {
            self.insert_into_slot(0, b0, x);
            return true;
        }
        let b1 = self.h2(&x);
        if self.buckets[1][b1].is_none() {
            self.insert_into_slot(1, b1, x);
            return true;
        }
        // We reach here when we cannot insert the
        // key straightaway to either of the slots.
        // In this case, we have to move things around
        // a bit to make space for it until we find some
        // space or rehash the elements with a larger table.
        let mut current = x;
        for _ in 0..MAX_LOOP {
            let b1 = self.h1(&current);
            if self.buckets[0][b1].is_none() {
                self.insert_into_slot(0, b1, current);
                return true;
            }
            // It is safe to expect this to be Some(x) because we
            // have already performed the None check in the previous
            // step, and we will never reach here in that case.
            current = self.buckets[0][b1].replace(current).expect("must not be None");
            let b2 = self.h2(&current);
            if self.buckets[1][b2].is_none() {
                self.insert_into_slot(1, b2, current);
                return true;
            }
        }
        // If we are here, it means that we don't have enough
        // slots to insert. Hence, we need to rehash and retry
        // inserting into the table.
        self.resize_and_rehash();
        self.insert(current);
        return true;
    }

    #[inline]
    fn insert_into_slot(&mut self, bucket_group: usize, bucket: usize, elem: T) {
        self.buckets[bucket_group][bucket] = Some(elem);
        self.size += 1;
    }

    fn resize_and_rehash(&mut self) {
        let new_capacity = self.capacity * 2;
        let mut resized = CuckooHashTable{
            buckets: [vec![None; new_capacity], vec![None; new_capacity]],
            size: 0,
            capacity: new_capacity,
            hash1: self.hash1.clone(),
            hash2: self.hash2.clone(),
        };
        for bucket in &mut self.buckets {
            for item in bucket.iter_mut().filter(|x| x.is_some()) {
                resized.insert(item.take().expect("unexpectedly none"));
            }
        }
        *self = resized;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use crate::cuckoo::CuckooHashTable;

    #[test]
    fn test_insert_and_contains() {
        let mut table = CuckooHashTable::new();
        assert!(table.insert(1));
        assert!(table.insert(2));
        assert!(table.insert(3));
        assert!(!table.insert(3));
        assert!(table.contains(&1));
        assert!(table.contains(&2));
        assert!(table.contains(&3));
        assert!(!table.contains(&4));
    }

    #[test]
    fn test_delete() {
        let mut table = CuckooHashTable::new();
        table.insert(1);
        table.insert(2);
        assert!(table.remove(&1));
        assert!(!table.contains(&1));
        assert!(table.contains(&2));
        assert!(!table.remove(&3));
    }

    #[quickcheck]
    fn prop_insert_and_delete_are_consistent_with_contains_and_std_hashmap(xs: Vec<i32>) -> TestResult {
        let mut table = CuckooHashTable::new();
        let mut set = HashSet::new();
        for &x in &xs {
            assert_eq!(table.insert(x), set.insert(x));
        }
        for &x in &xs {
            assert_eq!(table.contains(&x), set.contains(&x));
        }
        for &x in &xs {
            assert_eq!(table.remove(&x), set.remove(&x));
        }
        TestResult::passed()
    }
}