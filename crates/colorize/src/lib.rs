pub use colored;
pub use colored::control::ShouldColorize;
pub use colored::Color;
pub use hex_literal;
use once_cell::sync::Lazy;

pub static TRUECOLOR_SUPPORTED: Lazy<bool> = Lazy::new(|| {
    let colorterm = std::env::var("COLORTERM").ok();
    colorterm.as_deref() == Some("truecolor") || colorterm.as_deref() == Some("24bit")
});

/// an ergonomic string coloring macro
///
/// ```
/// use colorize::{colorize, Color, ShouldColorize};
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// colorize!("CLI v{}", version, hex("4A9C6D"), Color::BrightGreen);
/// colorize!("CLI v{}", version, hex("4A9C6D"), Color::BrightGreen);
/// colorize!("{}", "CLI", hex("4A9C6D"), Color::BrightGreen);
/// colorize!("CLI v{}", version, Color::BrightGreen);
/// colorize!("{}", "CLI", Color::BrightGreen);
/// colorize!("{}", version, Color::BrightGreen);
/// ```
#[macro_export]
macro_rules! colorize {
    ($fmt:literal, $($args:expr)*, $color: expr) => {{
        use $crate::colored::Colorize;
        format!($fmt, $($args)*).color($color)
    }};
    ($fmt:literal, $($args:expr)*, rgb($r:literal,$g:literal,$b: literal), $fallback: expr) => {{
        use $crate::colored::Colorize;
        let color = if *$crate::TRUECOLOR_SUPPORTED {
            $crate::colored::Color::TrueColor{ r: $r, g: $g, b: $b }
        } else {
            $fallback
        };

        format!($fmt, $($args)*).color(color)
    }};
    ($fmt:literal, $($args:expr)*, hex($hex:literal), $fallback: expr) => {{
        use $crate::colored::Colorize;
        let color = if *$crate::TRUECOLOR_SUPPORTED {
            let rgb = $crate::hex_literal::hex!($hex);
            $crate::colored::Color::TrueColor{ r: rgb[0], g: rgb[1], b: rgb[2] }
        } else {
            $fallback
        };

        format!($fmt, $($args)*).color(color)
    }};
}

/// a macro calling [`colorize`] and then [`println`]

/// ```
/// use colorize::{Color, ShouldColorize};
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// colorize::println!("CLI v{}", version, hex("4A9C6D"), Color::BrightGreen);
/// colorize::println!("{}", "CLI", hex("4A9C6D"), Color::BrightGreen);
/// colorize::println!("CLI v{}", version, rgb(74, 156, 109), Color::BrightGreen);
/// colorize::println!("CLI v{}", version, Color::BrightGreen);
/// colorize::println!("{}", "CLI", Color::BrightGreen);
/// ```
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {{
       let colored_string = $crate::colorize!($($args)*);
       std::println!("{}", colored_string);
    }};
}

/// a macro calling [`colorize`] and then [`eprintln`]
///
/// ```ignore
/// use colorize::{self, Color, ShouldColorize};
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// colorize::eprintln!("CLI v{}", version, hex("4A9C6D"), Color::BrightGreen);
/// colorize::eprintln!("CLI v{}", version, rgb(74, 156, 109), Color::BrightGreen);
/// colorize::eprintln!("CLI v{}", version, Color::BrightGreen);
/// colorize::eprintln!("{}", "CLI", Color::BrightGreen);
/// ```
#[macro_export]
macro_rules! eprintln {
    ($($args:tt)*) => {{
       let colored_string = $crate::colorize!($($args)*);
       std::eprintln!("{}", colored_string);
    }};
}
