use duplicate::duplicate_item;

/// Fixed-capacity, stack-allocated bit array backed by `[T; N]`.
///
/// `T` — storage word type (u8/u16/u32/u64/u128).  
/// `N` — number of words.
///
/// All operations are `const`, enabling compile-time bit-vector
/// computations.  Not intended for large or dynamic bit sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BinStore<T, const N: usize> {
    pub(self) bits: [T; N],
}

#[duplicate_item( raw_type; [u8]; [u16]; [u32]; [u64]; [u128]; )]
impl<const N: usize> BinStore<raw_type, N> {
    /// All-zero store.
    pub const ZERO: Self = Self { bits: [0; N] };

    /// Store with only the least-significant bit set.
    pub const ONE: Self = const {
        let mut rv = Self::ZERO;
        rv.set_bit(0);
        rv
    };

    /// Wrap a raw word array as a `BinStore`.
    pub const fn new(bits: [raw_type; N]) -> Self {
        Self { bits }
    }

    /// Build a store whose set bits are given by `bits` (element indices).
    pub const fn with_elements<const NN: usize>(bits: [usize; NN]) -> Self {
        let mut rv = Self::ZERO;
        let mut i = 0;
        while i < NN {
            rv.set_bit(bits[i]);
            i += 1;
        }
        rv
    }

    /// Set the bit at `index` to 1.
    pub const fn set_bit(&mut self, index: usize) {
        let word_index = index / (raw_type::BITS as usize);
        let bit_index = index % (raw_type::BITS as usize);
        self.bits[word_index] |= (1 as raw_type) << bit_index;
    }

    /// Clear the bit at `index` (set to 0).
    pub const fn clear_bit(&mut self, index: usize) {
        let word_index = index / (raw_type::BITS as usize);
        let bit_index = index % (raw_type::BITS as usize);
        self.bits[word_index] &= !((1 as raw_type) << bit_index);
    }

    /// Return `true` if the bit at `index` is set.
    pub const fn get_bit(&self, index: usize) -> bool {
        let word_index = index / (raw_type::BITS as usize);
        let bit_index = index % (raw_type::BITS as usize);
        (self.bits[word_index] & ((1 as raw_type) << bit_index)) != 0
    }

    /// Bitwise left shift (all words).  Equivalent to `<<` on a flat integer.
    pub const fn shift_left(mut self, amount: usize) -> Self {
        self.shift_left_assign(amount);
        self
    }

    /// In-place left shift.
    pub const fn shift_left_assign(&mut self, amount: usize) {
        if amount == 0 {
            return;
        }

        let word_shift = amount / (raw_type::BITS as usize);
        let bit_shift = amount % (raw_type::BITS as usize);

        if word_shift >= N {
            let mut i = 0;
            while i < N {
                self.bits[i] = 0;
                i += 1;
            }
            return;
        }

        let mut i = N - 1;
        loop {
            if i < word_shift {
                break;
            }
            self.bits[i] = self.bits[i - word_shift] << bit_shift;
            if bit_shift > 0 && i > word_shift {
                let carry_shift = (raw_type::BITS as usize) - bit_shift;
                self.bits[i] |= self.bits[i - word_shift - 1] >> carry_shift;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        let mut i = 0;
        while i < word_shift {
            self.bits[i] = 0;
            i += 1;
        }
    }

    /// Bitwise right shift (all words).  Equivalent to `>>` on a flat integer.
    pub const fn shift_right(mut self, amount: usize) -> Self {
        self.shift_right_assign(amount);
        self
    }

    /// In-place right shift.
    pub const fn shift_right_assign(&mut self, amount: usize) {
        if amount == 0 {
            return;
        }

        let word_shift = amount / (raw_type::BITS as usize);
        let bit_shift = amount % (raw_type::BITS as usize);

        if word_shift >= N {
            let mut i = 0;
            while i < N {
                self.bits[i] = 0;
                i += 1;
            }
            return;
        }

        let mut i = 0;
        while i < N - word_shift {
            self.bits[i] = self.bits[i + word_shift] >> bit_shift;
            if bit_shift > 0 && i + word_shift + 1 < N {
                let carry_shift = (raw_type::BITS as usize) - bit_shift;
                self.bits[i] |= self.bits[i + word_shift + 1] << carry_shift;
            }
            i += 1;
        }
        let mut i = N - word_shift;
        while i < N {
            self.bits[i] = 0;
            i += 1;
        }
    }

    /// Bitwise NOT (all words).
    pub const fn not(self) -> Self {
        let mut rv = Self::ZERO;
        let mut i = 0;
        while i < N {
            rv.bits[i] = !self.bits[i];
            i += 1;
        }
        rv
    }

    /// In-place bitwise NOT.
    pub const fn negate(&mut self) {
        let mut i = 0;
        while i < N {
            self.bits[i] = !self.bits[i];
            i += 1;
        }
    }

    /// Bitwise AND with another store (word by word).
    pub const fn bitand(self, other: &Self) -> Self {
        let mut rv = Self::ZERO;
        let mut i = 0;
        while i < N {
            rv.bits[i] = self.bits[i] & other.bits[i];
            i += 1;
        }
        rv
    }

    /// Bitwise OR with another store (word by word).
    pub const fn bitor(self, other: &Self) -> Self {
        let mut rv = Self::ZERO;
        let mut i = 0;
        while i < N {
            rv.bits[i] = self.bits[i] | other.bits[i];
            i += 1;
        }
        rv
    }

    /// In-place bitwise AND.
    pub const fn bitand_assign(&mut self, other: &Self) {
        let mut i = 0;
        while i < N {
            self.bits[i] &= other.bits[i];
            i += 1;
        }
    }

    /// In-place bitwise OR.
    pub const fn bitor_assign(&mut self, other: &Self) {
        let mut i = 0;
        while i < N {
            self.bits[i] |= other.bits[i];
            i += 1;
        }
    }

    /// Interpret the store as a flat integer and try to fit it into `usize`.
    /// Returns `None` if any bit beyond `usize::BITS` is set.
    pub const fn try_into_usize(self) -> Option<usize> {
        let mut result: usize = 0;
        let mut i = 0;
        while i < N {
            let word = self.bits[i];
            let shift = i * (raw_type::BITS as usize);
            if shift >= usize::BITS as usize {
                return None;
            }
            let remaining_bits = (usize::BITS as usize) - shift;
            let word_bits = raw_type::BITS as usize;
            if word_bits > remaining_bits && (word >> remaining_bits) != 0 {
                return None;
            }
            result |= (word as usize) << shift;
            i += 1;
        }
        Some(result)
    }

    /// Number of trailing zero bits; returns total capacity if no bit is set.
    pub const fn trailing_zeros(&self) -> usize {
        let mut total_zeros = 0;
        let mut i = 0;

        while i < N {
            let word = self.bits[i];
            let word_zeros = word.trailing_zeros() as usize;

            total_zeros += word_zeros;

            if word_zeros < (raw_type::BITS as usize) {
                break;
            }

            i += 1;
        }

        total_zeros
    }

    /// Population count (number of set bits).
    pub const fn count_ones(&self) -> usize {
        let mut total_ones = 0;
        let mut i = 0;

        while i < N {
            total_ones += self.bits[i].count_ones() as usize;
            i += 1;
        }

        total_ones
    }

    /// Alias for `count_ones`.
    pub const fn count(&self) -> usize {
        self.count_ones()
    }

    /// Zero out all bits at position ≥ `count`.
    pub const fn retain_lsb(&mut self, count: usize) {
        let raw_bits = raw_type::BITS as usize;
        let total_bits = N * raw_bits;

        if count >= total_bits {
            return;
        }

        let target_word = count / raw_bits;
        let bit_offset = count % raw_bits;

        if bit_offset == 0 {
            self.bits[target_word] = 0;
        } else {
            let mask = ((1 as raw_type) << bit_offset) - 1;
            self.bits[target_word] &= mask;
        }

        let mut i = target_word + 1;
        while i < N {
            self.bits[i] = 0;
            i += 1;
        }
    }

    /// `true` iff every bit is zero.
    pub const fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < N {
            if self.bits[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Index of the least-significant set bit; returns total bit capacity if none.
    pub const fn last(&self) -> usize {
        self.trailing_zeros()
    }

    /// Remove and return the least-significant set bit.
    /// Returns total bit capacity if the store is empty.
    pub const fn pop(&mut self) -> usize {
        let index = self.last();
        if index < N * (raw_type::BITS as usize) {
            self.clear_bit(index);
        }
        index
    }

    /// Interpret the store as a flat binary number and add a small word to it.
    /// The word is propagated through carries across all words.
    pub const fn add_raw(&self, rhs: raw_type) -> Self {
        let mut rv = *self;
        rv.add_assign_raw(rhs);
        rv
    }

    /// In-place version of `add_raw`.
    pub const fn add_assign_raw(&mut self, rhs: raw_type) {
        let mut carry = rhs;
        let mut i = 0;
        while i < N {
            let sum = self.bits[i].wrapping_add(carry);
            let overflowed = sum < self.bits[i];
            self.bits[i] = sum;
            if !overflowed {
                break;
            }
            carry = 1;
            i += 1;
        }
    }
    /// Interpret the store as a flat binary number and subtract a small word from it.
    /// Borrow is propagated across words if underflow occurs.
    pub const fn sub_raw(&self, rhs: raw_type) -> Self {
        let mut rv = *self;
        rv.sub_assign_raw(rhs);
        rv
    }

    /// In-place version of `sub_raw`.
    pub const fn sub_assign_raw(&mut self, rhs: raw_type) {
        let mut borrow = rhs;
        let mut i = 0;
        while i < N {
            let diff = self.bits[i].wrapping_sub(borrow);
            // Underflow occurs if borrowing caused the result to wrap around (be larger than the initial word).
            let underflowed = diff > self.bits[i];
            self.bits[i] = diff;
            if !underflowed {
                break;
            }
            borrow = 1;
            i += 1;
        }
    }

    pub const fn take_raw_bit(&self, index: usize) -> raw_type {
        self.bits[index]
    }

    pub const fn raw_bit(&self, index: usize) -> &raw_type {
        &self.bits[index]
    }

    pub const fn raw_bit_mut(&mut self, index: usize) -> &mut raw_type {
        &mut self.bits[index]
    }
}
