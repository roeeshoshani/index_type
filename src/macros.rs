use core::marker::PhantomData;

use crate::{IndexType, typed_slice::TypedSlice, typed_vec::TypedVec};

#[doc(hidden)]
pub const fn __const_assert_len_in_bounds<I: IndexType, const N: usize>() {
    struct AssertLenInBounds<I: IndexType, const N: usize>(PhantomData<I>);
    impl<I: IndexType, const N: usize> AssertLenInBounds<I, N> {
        pub const OK: () = {
            if N > I::MAX_RAW_INDEX {
                panic!("length exceeds maximum index type capacity");
            }
        };
    }
    let _ = AssertLenInBounds::<I, N>::OK;
}

#[doc(hidden)]
pub const fn __const_assert_vec_in_bounds<I: IndexType, T, const N: usize>(_: &TypedVec<I, T>) {
    __const_assert_len_in_bounds::<I, N>();
}

#[doc(hidden)]
pub const fn __const_assert_slice_in_bounds<I: IndexType, T, const N: usize>(_: &TypedSlice<I, T>) {
    __const_assert_len_in_bounds::<I, N>();
}

#[doc(hidden)]
pub const fn __unsafe_wrap_slice_mut<I: IndexType, T, const N: usize>(
    slice: &mut [T],
) -> &mut TypedSlice<I, T> {
    let res = unsafe { TypedSlice::from_slice_unchecked_mut(slice) };
    __const_assert_slice_in_bounds::<I, T, N>(res);
    res
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ignore {
    ($($any:tt)*) => {
        ()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __count {
    ($($x:expr),*) => {
        <[()]>::len(&[ $( $crate::__ignore!($x) ),* ])
    };
}

/// Creates a [`TypedVec`](crate::typed_vec::TypedVec) containing the arguments.
///
/// `typed_vec!` allows `TypedVec` to be defined with the same syntax as the standard library's `vec!` macro.
///
/// # Usage Example
///
/// ```rust
/// use index_type::{IndexType, typed_vec};
/// use index_type::typed_vec::TypedVec;
///
/// #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
///
/// let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];
/// assert_eq!(v.len_usize(), 3);
/// ```
#[macro_export]
macro_rules! typed_vec {
    ($elem:expr; $n:expr) => {{
        $crate::typed_vec::TypedVec::<_, _>::try_from_vec($crate::alloc::vec![($elem); ($n)]).unwrap()
    }};
    ($($x:expr),* $(,)?) => {{
            // SAFETY: The length is checked at compile time by __const_assert_vec_in_bounds.
            let res = unsafe { $crate::typed_vec::TypedVec::<_, _>::from_vec_unchecked($crate::alloc::vec![$($x),*]) };

            const __LEN: usize = $crate::__count!($($x),*);
            $crate::macros::__const_assert_vec_in_bounds::<_, _, __LEN>(&res);

            res
    }};
}

/// Creates a [`TypedArray`](crate::typed_array::TypedArray) containing the arguments.
///
/// # Usage Example
///
/// ```rust
/// use index_type::{IndexType, typed_array};
/// use index_type::typed_array::TypedArray;
///
/// #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
///
/// let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];
/// assert_eq!(a.len_usize(), 3);
/// ```
#[macro_export]
macro_rules! typed_array {
    ($elem:expr; $n:expr) => {{
        $crate::typed_array::TypedArray::<_, _, $n>::from_array([$elem; $n])
    }};
    ($($x:expr),* $(,)?) => {{
        $crate::typed_array::TypedArray::from_array([$($x),*])
    }};
}

/// Creates a reference to a [`TypedSlice`](crate::typed_slice::TypedSlice) containing the arguments.
///
/// This macro creates a temporary array and returns a reference to it as a `TypedSlice`.
/// Note that due to how temporary lifetimes work in Rust, the returned reference is only valid
/// for the duration of the statement it is in, unless it is immediately bound to a `let` variable.
///
/// # Usage Example
///
/// ```rust
/// use index_type::{IndexType, typed_slice};
/// use index_type::typed_slice::TypedSlice;
///
/// #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
///
/// let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
/// assert_eq!(s.len_usize(), 3);
/// ```
#[macro_export]
macro_rules! typed_slice {
    ($($x:expr),* $(,)?) => {{
        // SAFETY: The length is checked at compile time by __const_assert_slice_in_bounds.
        let res = unsafe { $crate::typed_slice::TypedSlice::<_, _>::from_slice_unchecked(&[$($x),*]) };

        const __LEN: usize = $crate::__count!($($x),*);
        $crate::macros::__const_assert_slice_in_bounds::<_, _, __LEN>(res);

        res
    }};
}

/// Creates a mutable reference to a [`TypedSlice`](crate::typed_slice::TypedSlice) containing the arguments.
///
/// This macro creates a temporary array and returns a mutable reference to it as a `TypedSlice`.
/// Note that due to how temporary lifetimes work in Rust, the returned reference is only valid
/// for the duration of the statement it is in.
///
/// Binding the result of this macro to a variable will produce a value that can't be use to due
/// the slice's temporary lifetime.
///
/// But, the result of this macro can properly be used in other situations, for example it can be
/// passed as a function argument.
///
/// # Usage Example
///
/// ```rust
/// use index_type::{IndexType, typed_slice_mut};
/// use index_type::typed_slice::TypedSlice;
///
/// #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
///
/// fn print_mut_slice(slice: &mut TypedSlice<MyIndex, i32>) {
///     println!("{:?}", slice);
/// }
///
/// print_mut_slice(typed_slice_mut![1, 2, 3]);
/// ```
#[macro_export]
macro_rules! typed_slice_mut {
    ($($x:expr),* $(,)?) => {
        // We must avoid creating an extra scope here so that the temporary `&mut [...]` value is created with the lifetime of the scope
        // in which this macro is evaluated.
        //
        // So, we put all of our code inside a function instead of creating an extra scope inside the macro body itself.
        //
        // Also note that we can't mark the function as unsafe, since that would require wrapping it in an unsafe block, but an unsafe
        // block is an extra scope, and we must avoid that.
        $crate::macros::__unsafe_wrap_slice_mut::<_, _, { $crate::__count!($($x),*) }>(&mut [$($x),*])
    };
}
