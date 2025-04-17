use cynic_parser::common::WrappingType;

pub(crate) fn convert_wrappers(wrappers: impl IntoIterator<Item = WrappingType>) -> wrapping::Wrapping {
    let wrappers = wrappers.into_iter().collect::<Vec<_>>();
    let mut wrappers = wrappers.into_iter().rev().peekable();

    let mut wrapping = if wrappers.next_if(|w| matches!(w, WrappingType::NonNull)).is_some() {
        wrapping::Wrapping::required()
    } else {
        wrapping::Wrapping::nullable()
    };

    while let Some(next) = wrappers.next() {
        debug_assert_eq!(next, WrappingType::List, "double non-null wrapping type not possible");

        wrapping = if wrappers.next_if(|w| matches!(w, WrappingType::NonNull)).is_some() {
            wrapping.list_non_null()
        } else {
            wrapping.list()
        }
    }

    wrapping
}
