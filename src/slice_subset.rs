use bitset_core::BitSet;

// SliceSubset is basically an immutable slice with the ability to retain only
// the elements matching a given predicate. It does this by maintaining a bit
// mask alongside the slice to keep track of which indexes are still "present".
//
// This is primarily useful for cases where one needs to repeatedly filter the
// same list of elements over and over and would rather not pay for full copies
// each time.
type BitMask = u128;

#[derive(Clone)]
pub struct SliceSubset<'a, T> {
    slice: &'a [T],
    bit_mask: BitMask,
}

impl<'a, T> SliceSubset<'a, T> {
    const MAX_CAPACITY: usize = std::mem::size_of::<BitMask>() * 8;

    pub fn from_slice(slice: &'a [T]) -> Option<SliceSubset<'a, T>> {
        if slice.len() <= Self::MAX_CAPACITY {
            // Set the slice.len() lowest bits to 1.
            let bit_mask = (1 << slice.len()) - 1;
            Some(Self { slice, bit_mask })
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bit_mask.bit_none()
    }

    pub fn len(&self) -> usize {
        self.bit_mask.count_ones() as usize
    }

    #[must_use]
    pub fn retained<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&T) -> bool,
    {
        let slice = self.slice;
        let mut bit_mask = self.bit_mask;
        let mut bits_to_visit = self.bit_mask;
        while bits_to_visit.bit_any() {
            let index = bits_to_visit.trailing_zeros() as usize;
            if !f(&slice[index]) {
                bit_mask.bit_reset(index);
            }
            bits_to_visit.bit_reset(index);
        }

        if bit_mask.bit_none() {
            Self {
                slice: &[],
                bit_mask,
            }
        } else {
            let first_occupied = bit_mask.trailing_zeros() as usize;
            let last_occupied = Self::MAX_CAPACITY - bit_mask.leading_zeros() as usize - 1;
            Self {
                slice: &slice[first_occupied..=last_occupied],
                bit_mask: bit_mask >> first_occupied,
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            slice_subset: self,
            bits_to_visit: self.bit_mask,
        }
    }
}

impl<'outer, 'inner, T> IntoIterator for &'outer SliceSubset<'inner, T>
where
    'outer: 'inner,
{
    type Item = &'inner T;
    type IntoIter = Iter<'inner, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, T> {
    slice_subset: &'a SliceSubset<'a, T>,
    bits_to_visit: BitMask,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.bits_to_visit.trailing_zeros() as usize;
        if index < SliceSubset::<'a, T>::MAX_CAPACITY {
            let v = &self.slice_subset.slice[index];
            self.bits_to_visit.bit_reset(index);
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_slice() {
        {
            let small_array = [1i32, 2, 3, 4, 5];
            let subset = SliceSubset::from_slice(&small_array);
            assert!(subset.is_some());
            assert_eq!(subset.unwrap().bit_mask, 0b00011111);
        }
        {
            let big_array = [42i32; 130];
            let subset = SliceSubset::from_slice(&big_array);
            assert!(subset.is_none());
        }
        {
            let empty_array = [0; 0];
            let subset = SliceSubset::from_slice(&empty_array);
            assert!(subset.is_some());
            let subset = subset.unwrap();
            assert!(subset.is_empty());
            assert_eq!(subset.len(), 0);
        }
    }

    #[test]
    fn test_iter() {
        let ar = [1i32, 2, 3, 4, 5];
        let subset = SliceSubset::from_slice(&ar).unwrap();
        assert_eq!(subset.len(), 5);
        assert!(!subset.is_empty());
        let evens = subset.retained(|i| i % 2 == 0);
        let mut it = evens.iter();
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&4));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_retained() {
        let ar = [0i32, 0, 0, 0, 2, 3, 0, 0, 4, 5, 0, 0, 0];
        let subset = SliceSubset::from_slice(&ar).unwrap();
        let non_zeroes = subset.retained(|i| *i != 0);
        assert_eq!(non_zeroes.len(), 4);
        let mut it = non_zeroes.iter();
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&3));
        assert_eq!(it.next(), Some(&4));
        assert_eq!(it.next(), Some(&5));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);

        assert_eq!(non_zeroes.slice.len(), 6);
    }

    #[test]
    fn test_retained_none() {
        let ar = [0i32, 0, 0, 0, 2, 3, 0, 0, 4, 5, 0, 0, 0];
        let subset = SliceSubset::from_slice(&ar).unwrap();
        let double_digits = subset.retained(|i| *i >= 10);
        assert_eq!(double_digits.len(), 0);
        assert_eq!(double_digits.slice.len(), 0);
        let mut it = double_digits.into_iter();
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_large() {
        let ar: Vec<_> = { 1..124 }.collect();
        let subset = SliceSubset::from_slice(&ar).unwrap();

        let multiples_of_7 = subset.retained(|i| *i % 7 == 0);
        assert_eq!(multiples_of_7.len(), 17);
        assert_eq!(multiples_of_7.slice.len(), 119 - 7 + 1);

        let mut it = multiples_of_7.iter();
        for i in { 7..=119 }.step_by(7) {
            assert_eq!(it.next(), Some(&i));
        }
        assert_eq!(it.next(), None);
    }
}
