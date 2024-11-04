#[macro_export]
macro_rules! time {
    ($expr:expr) => {{
        let instant = std::time::Instant::now();
        match $expr {
            tmp => {
                let elapsed = instant.elapsed();
                println!("[{}:{}] `{}` took {:?}", file!(), line!(), stringify!($expr), elapsed);
                tmp
            }
        }
    }};
}
