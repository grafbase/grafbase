

#[macro_export]
macro_rules! dot_get_opt {
    ($item: expr, $dotpath: expr, $ty: ty) => {{
        use json_dotpath::DotPaths;
        $item.dot_get::<$ty>($dotpath).ok().flatten()
    }};
    ($item: expr, $dotpath: expr) => {{
        use json_dotpath::DotPaths;
        $item.dot_get::<_>($dotpath).ok().flatten()
    }};
}

#[macro_export]
macro_rules! dot_get {
    ($item: expr, $dotpath: expr, $ty: ty) => {{
        let path = $dotpath;
        let item = &$item;
        $crate::dot_get_opt!(item, path, $ty).expect(&format!("path `{path}` cannot be resolved in {item:#?}"))
    }};
    ($item: expr, $dotpath: expr) => {{
        let path = $dotpath;
        let item = &$item;
        $crate::dot_get_opt!(item, path).expect(&format!("path `{path}` cannot be resolved in {item:#?}"))
    }};
}
