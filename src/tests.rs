mod tests {
    use gen::Callable;
    use iter::ReturnIterExt;
    use ::std::ops::{GeneratorState, Generator};
  
    // #[test]
    // fn __test_generator_into_iterator() {
    //     let mut g = Callable::new(|| {
    //         yield_from!(|| {
    //             for i in (0..5u8) {
    //                 yield i as char;
    //             }
    //         });

    //         return 99
    //     }).iter_all();

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
            let mut g = (&mut callable).iter_all().take(4);
            assert_eq!(g.next(), Some(0 as char));
            assert_eq!(g.next(), Some(1 as char));
            assert_eq!(g.next(), Some(2 as char));
            assert_eq!(g.next(), Some(3 as char));
            assert_eq!(g.next(), None);
        }

        let mut resumed = callable.iter_all();
        assert_eq!(resumed.next(), Some(4 as char));
        assert_eq!(resumed.next(), Some('c'));
        assert_eq!(resumed.next(), None);
    }

    #[test]
    fn test_chain() {
        let generator = Callable::new(move || {
            yield 1;
            yield 2;
            return 3;
        });

        let chain_once = generator.chain(|input| {
            move || {
                yield input * 2;
                return input;
            }
        }).unwrap();
        
        let chain_twice = chain_once.chain(|mut input| {
            move || {
                yield input * 10;
                input *= 10;
                input - 1
            }
        }).unwrap();

        let mut iter = chain_twice.iter_all();

        assert_eq!(iter.next(), Some(1)); // yield 1;
        assert_eq!(iter.next(), Some(2)); // yield 2
        assert_eq!(iter.next(), Some(6)); // yield input * 2;

        assert_eq!(iter.next(), Some(30)); // yield input * 10;
        assert_eq!(iter.next(), Some(29)); // in the last generator we do input *= 10 (results in input being 30), and then we return 'input - 1'
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn make_new() {
        let mut generator = Callable::new(|| {
            for i in 0..10 {
                yield i;
            }
            return 2;
        });

        {
            let mut iter = generator.borrow_mut(|gen| {
                move || {
                    let sum = gen.iter_all().take(3).sum();
                    for i in 0..sum {
                        yield i;
                    }
                    return 0;
                }
            }).unwrap().iter_all();

            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(1));
            assert_eq!(iter.next(), Some(2));
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), None);
        }

        let mut iter = generator.iter_all();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), Some(8));
        assert_eq!(iter.next(), Some(9));
        assert_eq!(iter.next(), Some(2));
    }
}