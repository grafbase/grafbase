use itertools::join;

pub fn joined_repeating(repeated: &'static str, times: usize, delimiter: &'static str) -> String {
    join(std::iter::repeat(repeated.to_owned()).take(times), delimiter)
}
