pub struct Defer<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> Defer<F> {
    #[inline(always)]
    pub fn new(f: F) -> Defer<F> {
        Defer { f: Some(f) }
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}

mod test {
    #[test]
    fn test() {
        fn defer() {
            let a = 4u32;
            crate::defer::Defer::new(|| {
                assert_eq!(1u32, a);
            });
        }

        let result = std::panic::catch_unwind(|| {
            defer();
        });
        
        assert!(result.is_err());
    }
}