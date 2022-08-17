pub use json_dotpath::DotPaths;

#[macro_export]
macro_rules! dot_get_opt {
    ($item: ident, $dotpath: literal, $ty: ty) => {{
        use json_dotpath::DotPaths;
        $item.dot_get::<$ty>($dotpath).ok().flatten()
    }};
    ($item: ident, $dotpath: literal) => {{
        use json_dotpath::DotPaths;
        $item.dot_get::<_>($dotpath).ok().flatten()
    }};
}

#[macro_export]
macro_rules! dot_get {
    ($item: ident, $dotpath: literal, $ty: ty) => {{
        $crate::dot_get_opt!($item, $dotpath, $ty).expect(concat!("path `", $dotpath, "` cannot be resolved"))
    }};
    ($item: ident, $dotpath: literal) => {{
        $crate::dot_get_opt!($item, $dotpath).expect(concat!("path `", $dotpath, "` cannot be resolved"))
    }};
}
