mod tests {
    use gen::Callable;
    use iter::ReturnIterExt;

    // #[test]
    // fn __test_generator_into_iterator() {
    //     let mut g = Callable::new(|| {
    //         yield_from!(|| {
    //             for i in (0..5u8) {
    //                 yield i as char;
    //             }
    //         });

    //         return 99
    //     }).iter_with_return();

    //     assert_eq!(g.next(), Some(0 as char));
    //     assert_eq!(g.next(), Some(1 as char));
    //     assert_eq!(g.next(), Some(2 as char));
    //     assert_eq!(g.next(), Some(3 as char));
    //     assert_eq!(g.next(), Some(4 as char));
    //     assert_eq!(g.next(), Some('c'));
    // }

    #[test]
    fn test_generator_into_iterator() {
        let mut char_yielder = || {
            for i in 0..5u8 {
                yield i as char;
            }
        };

        let mut callable = Callable::new(|| {
            yield_from!(char_yielder);

            return 99
        });
        {
            let mut g = (&mut callable).iter_with_return().take(4);
            assert_eq!(g.next(), Some(0 as char));
            assert_eq!(g.next(), Some(1 as char));
            assert_eq!(g.next(), Some(2 as char));
            assert_eq!(g.next(), Some(3 as char));
            assert_eq!(g.next(), None);
        }

        let mut resumed = callable.iter_with_return();
        assert_eq!(resumed.next(), Some(4 as char));
        assert_eq!(resumed.next(), Some('c'));
        assert_eq!(resumed.next(), None);
    }
}