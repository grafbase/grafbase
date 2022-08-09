pub use json_dotpath::DotPaths;
#[macro_export]
macro_rules! dot_get {
    ($item: ident, $dotpath: literal) => {{
        use json_dotpath::DotPaths;
        $item.dot_get($dotpath).ok().flatten().unwrap()
    }};
}
