use std::mem::{self, MaybeUninit};

/// Stack allocated first in last out list
pub struct Stack<T, const S: usize = 64> {
    items: [MaybeUninit<T>;S],
    len: usize,
}

impl<T> Stack<T, 64> {
    /// Constructs a new, empty [`Stack<T>`] with default max capacity.
    pub fn new() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() };64],
            len: 0,
        }
    }
}

impl<T, const S: usize> Stack<T, S> {
    /// Constructs a new, empty [`Stack<T>`] with specified max capacity.
    pub fn with_size() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() };S],
            len: 0,
        }
    }

    /// Returns `true` if the stack contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if no capacity left.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len == S
    }

    /// Returns the number of items in the stack, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Appends an element to the back of a stack, return it back if size reach max capacity
    pub fn push(&mut self, item: T) -> Option<T> {
        if self.len == S {
            return Some(item);
        }
        self.items[self.len].write(item);
        self.len += 1;
        None
    }

    /// Removes the last item from a stack and returns it, or [`None`] if it is empty.
    #[must_use]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.len -= 1;
        let item = mem::replace(&mut self.items[self.len], MaybeUninit::uninit());
        Some(unsafe { item.assume_init() })
    }

    /// Extracts a slice containing the entire stack.
    pub fn as_slice(&self) -> &[T] {
        unsafe { mem::transmute(&self.items[..self.len]) }
    }

    /// Extracts a mutable slice of the entire stack.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { mem::transmute(&mut self.items[..self.len]) }
    }
}

impl<T, const S: usize> Drop for Stack<T,S> {
    fn drop(&mut self) {
        for item in &mut self.items[..self.len] {
            unsafe { item.assume_init_drop() };
        }
    }
}

impl<T, const S: usize> IntoIterator for Stack<T,S> {
    type Item = T;

    type IntoIter = IntoIter<T, S>;

    /// Consume [`Stack<T>`] and return an [`Iterator`]
    ///
    /// The order of returned items from iterator is back to front
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len;
        let items = mem::replace(&mut self.items, [const { MaybeUninit::uninit() };S]);

        // prevent `Drop` deallocating
        self.len = 0;

        IntoIter { items, initial_size: len, len }
    }
}

impl<T, const S: usize> std::ops::Deref for Stack<T, S> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const S: usize> std::ops::DerefMut for Stack<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}


/// A consuming iterator from [`Stack<T>`]
///
/// The order of returned items is back to front
pub struct IntoIter<T, const S: usize> {
    items: [MaybeUninit<T>;S],
    initial_size: usize,
    len: usize,
}

impl<T, const S: usize> Drop for IntoIter<T,S> {
    fn drop(&mut self) {
        for item in &mut self.items[..self.len] {
            unsafe { item.assume_init_drop() };
        }
    }
}

impl<T, const S: usize> ExactSizeIterator for IntoIter<T, S> { }

impl<T, const S: usize> Iterator for IntoIter<T, S> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let item = mem::replace(&mut self.items[self.initial_size], MaybeUninit::uninit());
        Some(unsafe { item.assume_init() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len,Some(self.len))
    }
}

