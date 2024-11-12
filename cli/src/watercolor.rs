pub use colored;
pub use colored::control::ShouldColorize;

use std::sync::LazyLock;

#[allow(dead_code)]
pub static TRUECOLOR_SUPPORTED: LazyLock<bool> = LazyLock::new(|| {
    let colorterm = std::env::var("COLORTERM").ok();
    colorterm.as_deref() == Some("truecolor") || colorterm.as_deref() == Some("24bit")
});

/// an ergonomic macro for printing colored strings in a terminal environment
///
/// ```
/// use watercolor::{watercolor, ShouldColorize};
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// watercolor!("CLI v{version}", @hex("4A9C6D"), @@BrightGreen);
/// watercolor!("CLI v{}", version, @hex("4A9C6D"), @@BrightGreen);
/// watercolor!("CLI", @hex("4A9C6D"), @@BrightGreen);
/// watercolor!("CLI v{version}", @BrightGreen);
/// watercolor!("CLI", @BrightGreen);
/// watercolor!("{version}", @BrightGreen);
/// ```
macro_rules! watercolor {
    ($fmt:expr, $($args:expr,)* @$color:ident$(,)?) => {{
        use $crate::watercolor::colored::Colorize;
        format!($fmt, $($args,)*).color($crate::watercolor::colored::Color::$color)
    }};
    ($fmt:expr, $($args:expr,)* @rgb($r:literal,$g:literal,$b:literal), @@$fallback:ident$(,)?) => {{
        use $crate::colored::Colorize;
        let color = if *$crate::TRUECOLOR_SUPPORTED {
            $crate::colored::Color::TrueColor{ r: $r, g: $g, b: $b }
        } else {
            $crate::colored::Color::$fallback
        };

        format!($fmt, $($args,)*).color(color)
    }};
    ($fmt:expr, $($args:expr,)* @hex($hex:literal), @@$fallback:ident$(,)?) => {{
        use $crate::watercolor::colored::Colorize;
        let color = if *$crate::watercolor::TRUECOLOR_SUPPORTED {
            let rgb = $crate::watercolor::hex_literal::hex!($hex);
            $crate::watercolor::colored::Color::TrueColor{ r: rgb[0], g: rgb[1], b: rgb[2] }
        } else {
            $crate::watercolor::colored::Color::$fallback
        };

        format!($fmt, $($args)*).color(color)
    }};
}

/// a macro calling [`watercolor`] and then [`println`]
///
/// ```
/// use watercolor::ShouldColorize;
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// watercolor::println!("CLI v{version}", @hex("4A9C6D"), @@BrightGreen);
/// watercolor::println!("CLI", @hex("4A9C6D"), @@BrightGreen);
/// watercolor::println!("CLI v{}", version, @rgb(74, 156, 109), @@BrightGreen);
/// watercolor::println!("CLI v{version}", @BrightGreen);
/// watercolor::println!("CLI", @BrightGreen);
/// ```
macro_rules! output {
    ($($args:tt)*) => {{
       let colored_string = $crate::watercolor::watercolor!($($args)*);
       std::println!("{}", colored_string);
    }};
}

/// a macro calling [`watercolor`] and then [`eprintln`]
///
/// ```ignore
/// use watercolor::ShouldColorize;
///
/// ShouldColorize::from_env();
///
/// let version = "0.1.0";
/// watercolor::eprintln!("CLI v{version}", @hex("4A9C6D"), @@BrightGreen);
/// watercolor::eprintln!("CLI v{}", version, @rgb(74, 156, 109), @@BrightGreen);
/// watercolor::eprintln!("CLI v{version}", @BrightGreen);
/// watercolor::eprintln!("CLI", @BrightGreen);
/// ```
macro_rules! output_error {
    ($($args:tt)*) => {{
       let colored_string = $crate::watercolor::watercolor!($($args)*);
       std::eprintln!("{}", colored_string);
    }};
}

pub(crate) use output;
pub(crate) use output_error;
pub(crate) use watercolor;
