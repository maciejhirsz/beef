use core::ptr::slice_from_raw_parts_mut;
use crate::traits::Capacity;

pub type Cow<'a, T> = crate::generic::Cow<'a, T, Cursed>;

#[derive(Clone, Copy)]
pub struct Cursed;

const MASK_LO: usize = u32::max_value() as usize;
const MASK_HI: usize = !u32::max_value() as usize;

impl Capacity for Cursed {
    type NonZero = Cursed;

    #[inline]
    fn empty<T>(ptr: *mut T, len: usize) -> (*mut [T], Cursed) {
        (slice_from_raw_parts_mut(ptr, len & MASK_LO), Cursed)
    }

    #[inline]
    fn store<T>(ptr: *mut T, len: usize, capacity: usize) -> (*mut [T], Cursed) {
    	if capacity > MASK_LO {
    		panic!("beef::skinny::Cow: Capacity out of bounds");
    	}

        (
        	slice_from_raw_parts_mut(
        		ptr,
        		(len & MASK_LO) | ((capacity & MASK_HI) << 32),
        	),
        	Cursed,
        )
    }

    #[inline]
    fn unpack(len: usize, _: Cursed) -> (usize, usize) {
        (len & MASK_LO, (len & MASK_HI) >> 32)
    }

    #[inline]
    fn maybe(len: usize, _: Cursed) -> Option<Cursed> {
        if len & MASK_HI != 0 {
        	Some(Cursed)
        } else {
        	None
        }
    }
}