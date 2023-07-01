use std::ops::Deref;

use crate::{InputValueError, LegacyInputType};

pub fn max_items<T: Deref<Target = [E]> + LegacyInputType, E>(
    value: &T,
    len: usize,
) -> Result<(), InputValueError<T>> {
    if value.deref().len() <= len {
        Ok(())
    } else {
        Err(format!(
            "the value length is {}, must be less than or equal to {}",
            value.deref().len(),
            len
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_items() {
        assert!(max_items(&vec![1, 2], 3).is_ok());
        assert!(max_items(&vec![1, 2, 3], 3).is_ok());
        assert!(max_items(&vec![1, 2, 3, 4], 3).is_err());
    }
}
