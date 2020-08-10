#![cfg(test)]

use crate::prelude::*;

use core::ptr;

use std::panic::catch_unwind;

#[test]
fn push() {
	let mut bvm08 = BitVec::<Msb0, u8>::new();
	assert!(bvm08.is_empty());

	bvm08.push(false);
	assert_eq!(bvm08.len(), 1);
	assert!(!bvm08[0]);

	bvm08.push(true);
	assert_eq!(bvm08.len(), 2);
	assert!(bvm08[1]);

	bvm08.extend(&[true; 3]);
	bvm08.extend(&[false; 3]);
	assert_eq!(bvm08, bits![0, 1, 1, 1, 1, 0, 0, 0]);
}

#[test]
fn inspect() {
	let bv = bitvec![Local, u16; 0; 40];
	assert_eq!(bv.elements(), 3);
}

#[test]
fn force_align() {
	let data = 0xA5u8;
	let bits = data.view_bits::<Msb0>();

	let mut bv = bits[2 ..].to_bitvec();
	assert_eq!(bv.as_slice(), &[0xA5u8]);
	bv.force_align();
	assert_eq!(bv.as_slice(), &[0b1001_0101]);
	bv.force_align();
	assert_eq!(bv.as_slice(), &[0b1001_0101]);
}

#[test]
#[should_panic(expected = "Vector capacity exceeded")]
fn overcommit() {
	BitVec::<Local, usize>::with_capacity(
		BitSlice::<Local, usize>::MAX_BITS + 1,
	);
}

#[test]
#[should_panic(
	expected = "Attempted to reconstruct a `BitVec` from a null pointer"
)]
fn from_null() {
	unsafe {
		BitVec::from_raw_parts(
			ptr::slice_from_raw_parts_mut(ptr::null_mut::<u8>(), 64)
				as *mut BitSlice<Local, usize>,
			0,
		);
	}
}

#[test]
fn reservations() {
	let mut bv = bitvec![1; 40];
	assert!(bv.capacity() >= 40);
	bv.reserve(100);
	assert!(bv.capacity() >= 140, "{}", bv.capacity());
	bv.shrink_to_fit();
	assert!(bv.capacity() >= 40);

	//  Trip the first assertion by wrapping around the `usize` boundary.
	assert!(
		catch_unwind(|| {
			let mut bv = bitvec![1; 100];
			bv.reserve(!0 - 50);
		})
		.is_err()
	);

	//  Trip the second assertion by exceeding `MAX_BITS` without wrapping.
	assert!(
		catch_unwind(|| {
			let mut bv = bitvec![1; 100];
			bv.reserve(BitSlice::<Local, usize>::MAX_BITS - 50);
		})
		.is_err()
	);

	let mut bv = bitvec![1; 40];
	assert!(bv.capacity() >= 40);
	bv.reserve_exact(100);
	assert!(bv.capacity() >= 140, "{}", bv.capacity());

	//  Trip the first assertion by wrapping around the `usize` boundary.
	assert!(
		catch_unwind(|| {
			let mut bv = bitvec![1; 100];
			bv.reserve_exact(!0 - 50);
		})
		.is_err()
	);

	//  Trip the second assertion by exceeding `MAX_BITS` without wrapping.
	assert!(
		catch_unwind(|| {
			let mut bv = bitvec![1; 100];
			bv.reserve_exact(BitSlice::<Local, usize>::MAX_BITS - 50);
		})
		.is_err()
	);
}

#[test]
fn iterators() {
	let data = 0x35u8.view_bits::<Msb0>();
	let bv: BitVec = data.iter().collect();
	assert_eq!(bv.count_ones(), 4);

	for (l, r) in (&bv).into_iter().zip(bits![0, 0, 1, 1, 0, 1, 0, 1]) {
		/* Unimportant but interesting implementation detail: all accessors in
		the crate that return an `&bool` use the same two addresses, because
		the compiler unifies `&literal` references into secret statics. You
		could argue that, much like YAML accepting yes/no as boolean literals,
		there are now four valid boolean values in the crate: `true`, `false`,
		and the addresses of their secret statics.

		Switch to a by-value comparison instead of by-ref if this test fails.
		*/
		assert_eq!(l as *const _, r as *const _);
	}

	let mut iter = bv.clone().into_iter();
	assert!(!iter.next().unwrap());
	assert_eq!(iter.as_bitslice(), data[1 ..]);
	assert_eq!(iter.as_slice(), &[0x35]);
}

#[test]
fn misc() {
	let mut bv = bitvec![1; 10];
	bv.truncate(20);
	assert_eq!(bv, bits![1; 10]);
	bv.truncate(5);
	assert_eq!(bv, bits![1; 5]);

	let mut bv = bitvec![0, 0, 1, 0, 0];
	assert!(bv.swap_remove(2));
	assert!(bv.not_any());

	bv.insert(2, true);
	assert_eq!(bv, bits![0, 0, 1, 0, 0]);

	bv.remove(2);
	assert!(bv.not_any());

	let mut bv = bitvec![0, 0, 1, 1, 0, 1, 0, 1, 0, 0];
	bv.retain(|idx, bit| !(idx & 1 == 0 && *bit));
	//                                         ^^^ even ^^^    prime
	assert_eq!(bv, bits![0, 0, 1, 0, 1, 0, 1, 0, 0]);
	//                        ^ 2 is the only even prime

	let mut bv_1 = bitvec![Lsb0, u8; 0; 5];
	let mut bv_2 = bitvec![Msb0, u16; 1; 5];
	bv_1.append(&mut bv_2);

	assert_eq!(bv_1, bits![0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
	assert!(bv_2.is_empty());

	let bv_3 = bv_1.split_off(5);
	assert!(bv_1.not_any());
	assert!(bv_3.all());

	let mut last = false;
	bv_1.resize_with(10, || {
		last = !last;
		last
	});
	assert_eq!(bv_1, bits![0, 0, 0, 0, 0, 1, 0, 1, 0, 1]);

	let mut bv = bitvec![];
	bv.extend_from_slice(&[false, false, true, true, false, true]);
	assert_eq!(bv, bits![0, 0, 1, 1, 0, 1]);
}