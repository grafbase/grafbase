use tokio::sync::watch;

/// Creates a watch::Receiver that never closes with a value that never changes
///
/// Note that this intentionally leaks memory so you should _not_ call it regularly
pub fn constant_watch_receiver<T>(value: T) -> watch::Receiver<T> {
    let (sender, receiver) = watch::channel(value);
    std::mem::forget(sender);
    receiver
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constant_watch_receiver_should_not_close() {
        let mut test = constant_watch_receiver(100);
        assert_eq!(*test.borrow_and_update(), 100);
        assert_eq!(*test.borrow(), 100);
        assert!(!test.has_changed().expect("the receiver should not close"));
    }
}
