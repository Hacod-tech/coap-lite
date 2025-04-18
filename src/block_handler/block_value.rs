use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::error::{IncompatibleOptionValueFormat, InvalidBlockValue};
use crate::option_value::{OptionValueType, OptionValueU32};

/// The block option value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockValue {
    pub num: u32,
    pub more: bool,
    pub size_exponent: u8,
}

// 2^20 - 1
const MAX_BLOCK_NUMBER: u32 = 1048575;

impl BlockValue {
    pub fn new(
        num: usize,
        more: bool,
        size: usize,
    ) -> Result<Self, InvalidBlockValue> {
        let true_size_exponent = Self::largest_power_of_2_not_in_excess(size)
            .ok_or(InvalidBlockValue::SizeExponentEncodingError(size))?;

        let size_exponent = u8::try_from(true_size_exponent.saturating_sub(4))
            .map_err(InvalidBlockValue::TypeBoundsError)?;
        if size_exponent > 0x7 {
            return Err(InvalidBlockValue::SizeExponentEncodingError(size));
        }
        let num =
            u32::try_from(num).map_err(InvalidBlockValue::TypeBoundsError)?;
        if num > MAX_BLOCK_NUMBER {
            return Err(InvalidBlockValue::MaximumNumberExceeded(num));
        }
        Ok(Self {
            num,
            more,
            size_exponent,
        })
    }

    /// Finds the largest power of 2 that does not exceed `target`.
    fn largest_power_of_2_not_in_excess(target: usize) -> Option<usize> {
        if target == 0 {
            return None;
        }

        let max_power = usize::try_from(usize::BITS).unwrap();
        let power_in_excess = (0..max_power).find(|i| (1 << i) > target);

        match power_in_excess {
            Some(size) => Some(size - 1),
            None => Some(max_power),
        }
    }

    pub fn size(&self) -> usize {
        1 << (self.size_exponent + 4)
    }
}

impl From<BlockValue> for Vec<u8> {
    fn from(block_value: BlockValue) -> Vec<u8> {
        let scalar = u32::from(block_value.num) << 4
            | u32::from(block_value.more) << 3
            | u32::from(block_value.size_exponent & 0x7);
        Vec::from(OptionValueU32(scalar))
    }
}

impl TryFrom<Vec<u8>> for BlockValue {
    type Error = IncompatibleOptionValueFormat;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let scalar = OptionValueU32::try_from(value)?.0;

        let num: u32 = scalar >> 4;
        let more = scalar >> 3 & 0x1 == 0x1;
        let size_exponent: u8 = (scalar & 0x7) as u8;
        Ok(Self {
            num,
            more,
            size_exponent,
        })
    }
}

impl OptionValueType for BlockValue {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highest_containing_power_of_2() {
        assert_eq!(BlockValue::largest_power_of_2_not_in_excess(0), None);
        assert_eq!(BlockValue::largest_power_of_2_not_in_excess(256), Some(8));
        assert_eq!(BlockValue::largest_power_of_2_not_in_excess(257), Some(8));
        assert_eq!(
            BlockValue::largest_power_of_2_not_in_excess(usize::MAX),
            Some(usize::try_from(usize::BITS).unwrap())
        );
    }

    #[test]
    fn test_block_value_exponent() {
        assert!(BlockValue::new(0, false, 0).is_err());
        assert!(BlockValue::new(0, false, usize::MAX).is_err());
        assert_eq!(
            BlockValue::new(0, false, 1158).unwrap(),
            BlockValue {
                num: 0,
                more: false,
                size_exponent: 6
            }
        );
        assert_eq!(
            BlockValue::new(0, false, 256).unwrap(),
            BlockValue {
                num: 0,
                more: false,
                size_exponent: 4
            }
        );
    }

    #[test]
    fn encode_block_opt_4096() {
        let opt = BlockValue::new(4096, false, 1024).unwrap();
        let bytes = Vec::<u8>::from(opt);
        assert_eq!(bytes, vec![0x01, 0x00, 0x06]);
    }

    #[test]
    fn encode_block_opt_4095() {
        let opt = BlockValue::new(4095, false, 1024).unwrap();
        let bytes = Vec::<u8>::from(opt);
        assert_eq!(bytes, vec![0xff, 0xf6]);
    }
}
