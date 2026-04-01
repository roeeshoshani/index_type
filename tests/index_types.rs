use core::num::NonZeroU8;

use index_type::{GenericIndexTooBigError, IndexScalarType, IndexTooBigError, IndexType};

#[test]
fn test_error_and_index_primitives_cover_basic_contracts() {
    let err = GenericIndexTooBigError::new();
    assert_eq!(err.to_string(), "index too big");

    fn assert_error<E: std::error::Error>(_err: &E) {}
    assert_error(&err);

    assert_eq!(u8::try_from_usize(255), Some(255));
    assert_eq!(u8::try_from_usize(256), None);
    assert_eq!(IndexScalarType::checked_add_scalar(5u8, 4), Some(9));
    assert_eq!(IndexScalarType::checked_sub_scalar(5u8, 6), None);
    assert_eq!(5u8.to_usize(), 5);
    assert_eq!(unsafe { u8::from_usize_unchecked(7) }, 7);

    assert_eq!(u16::try_from_raw_index(42).unwrap(), 42);
    assert_eq!(u16::BIAS, 0);
    assert_eq!(u16::MAX_INDEX.to_raw_index(), u16::MAX as usize);
    assert_eq!(u16::MAX_INDEX.to_raw_index_biased(), u16::MAX as usize);
    assert_eq!(u16::try_from_scalar(9).unwrap().to_scalar(), 9);
    assert_eq!(u16::checked_mul_scalar(7, 6).unwrap(), 42);
    assert_eq!(IndexType::checked_sub_scalar(3u16, 5), None);
    assert_eq!(u16::checked_sub_index(9, 4), Some(5));
    assert_eq!(unsafe { u16::from_raw_index_unchecked(11) }, 11);
    assert_eq!(unsafe { IndexType::unchecked_add_scalar(4u16, 5) }, 9);
    assert_eq!(unsafe { IndexType::unchecked_sub_scalar(9u16, 5) }, 4);
    assert_eq!(unsafe { u16::unchecked_sub_index(9, 5) }, 4);

    let nz = NonZeroU8::try_from_raw_index(4).unwrap();
    assert_eq!(NonZeroU8::BIAS, 1);
    assert_eq!(nz.to_raw_index(), 4);
    assert_eq!(nz.to_raw_index_biased(), 5);
    assert_eq!(NonZeroU8::ZERO.to_raw_index(), 0);
    assert_eq!(NonZeroU8::ZERO.to_raw_index_biased(), 1);
    assert_eq!(NonZeroU8::MAX_INDEX.to_raw_index(), 254);
    assert_eq!(NonZeroU8::MAX_INDEX.to_raw_index_biased(), 255);
    assert_eq!(NonZeroU8::try_from_scalar(6).unwrap().to_scalar(), 6);
    assert_eq!(nz.checked_add_scalar(3).unwrap().to_raw_index(), 7);
    assert_eq!(nz.checked_mul_scalar(3).unwrap().to_raw_index(), 12);
    assert_eq!(nz.checked_sub_scalar(2).unwrap().to_raw_index(), 2);
    assert_eq!(nz.checked_sub_index(NonZeroU8::ZERO), Some(4));
    assert_eq!(
        unsafe { NonZeroU8::from_scalar_unchecked(8) }.to_raw_index(),
        8
    );
    assert_eq!(
        unsafe { NonZeroU8::unchecked_add_scalar(nz, 2) }.to_raw_index(),
        6
    );
    assert_eq!(
        unsafe { NonZeroU8::unchecked_sub_scalar(nz, 2) }.to_raw_index(),
        2
    );
    assert_eq!(
        unsafe { NonZeroU8::unchecked_sub_index(nz, NonZeroU8::ZERO) },
        4
    );
}
