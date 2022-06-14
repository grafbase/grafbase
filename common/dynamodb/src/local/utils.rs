pub fn joined_repeating(repeated: &'static str, times: usize, delimiter: &'static str) -> String {
    std::iter::repeat(repeated.to_owned())
        .take(times)
        .collect::<Vec<String>>()
        .join(delimiter)
}
