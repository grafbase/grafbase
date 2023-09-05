use std::{fmt::Display, ops::Rem};

use num_traits::{AsPrimitive, Zero};

use crate::{InputValueError, LegacyInputType};

pub fn multiple_of<T, N>(value: &T, n: N) -> Result<(), InputValueError<T>>
where
    T: AsPrimitive<N> + LegacyInputType,
    N: Rem<Output = N> + Zero + Display + Copy + PartialEq + 'static,
{
    let value = value.as_();
    if !value.is_zero() && value % n == N::zero() {
        Ok(())
    } else {
        Err(format!("the value must be a multiple of {n}.").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_of() {
        assert!(multiple_of(&5, 3).is_err());
        assert!(multiple_of(&6, 3).is_ok());
        assert!(multiple_of(&0, 3).is_err());
    }
}
