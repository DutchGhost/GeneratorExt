mod tests {
    use gen::{Gen, Callable};
    use iter::{ReturnIterExt, ReturnIterator, YieldIterExt};

    #[test]
    fn test_generator_into_iterator() {
        let mut g = Callable::new(|| {
            yield_from!(|| {
                for i in (0..5u8) {
                    yield i;
                }
            });

            return '\n';
        }).iter_yield_only();

        assert_eq!(g.next(), Some(0));
        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), Some(3));
        assert_eq!(g.next(), Some(4));
        assert_eq!(g.next(), Some(b'\n'));
    }
}