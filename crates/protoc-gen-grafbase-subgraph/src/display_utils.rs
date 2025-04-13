use std::fmt;

pub(crate) fn display_fn(f: impl Fn(&mut fmt::Formatter<'_>) -> fmt::Result) -> impl fmt::Display {
    struct DisplayFn<F>(F);

    impl<F> fmt::Display for DisplayFn<F>
    where
        F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.0)(f)
        }
    }

    DisplayFn(f)
}

pub(crate) fn grpc_path_to_graphql_name(path: &str) -> impl fmt::Display {
    display_fn(|f| {
        let mut segments = path
            .split_terminator('.')
            .filter(|segment| !segment.is_empty())
            .peekable();

        while let Some(segment) = segments.next() {
            f.write_str(segment)?;

            if segments.peek().is_some() {
                f.write_str("_")?;
            }
        }

        Ok(())
    })
}
