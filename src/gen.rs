use std::ops::Generator;
use std::ops::GeneratorState;

#[macro_export]
macro_rules! yield_from {

    ($g:expr) => (
        unsafe {
            loop {
                match $g.resume() {
                     GeneratorState::Yielded(y) => yield y,
                    GeneratorState::Complete(ret) => break ret,
                }
            }
        }
    );
}

/// This macro is used for the implementation of the `GenOnce` trait.
/// It advances a Generator, but returning the Yield variant of [State](gen/enum.State.html), containing the Unit type if the Generator yielded.
/// On return, you can bind the value to a value, like ```let ret = return_from_yield!(generator)```.
#[macro_export]
macro_rules! return_from_yield {
    ($g:expr) => {
        unsafe {
            match $g.resume() {
                GeneratorState::Yielded(_) => return Some(State::Yield(())),
                GeneratorState::Complete(ret) => ret
            }
        }
    }
}

/// This macro is used for the implementation of the `Gen` trait.
/// It advances a Generator, but returning the Yield variant of [State](gen/enum.State.html), with the yielded value if the Generator yielded.
/// On return, you can bind the value to a value, like ```let ret = return_yielded!(generator)```.
#[macro_export]
macro_rules! return_yielded {
    ($g:expr) => {
        unsafe {
            match $g.resume() {
                GeneratorState::Yielded(y) => return Some(State::Yield(y)),
                GeneratorState::Complete(ret) => ret
            }
        }
    }
}

/// Indicates the State of Generator.
/// This Enum is used by functions and methods that advance a Generator.
#[derive(Debug)]
pub enum State<Y, R> {
    Yield(Y),
    Return(R)
}

impl <Y, R: Into<Y>> Into<Option<Y>> for State<Y, R> {
    
    #[inline]
    fn into(self) -> Option<Y> {
        match self {
            State::Yield(value) => Some(value),
            State::Return(value) => Some(value.into())
        }
    }
}

pub type ResumeOnce<R> = Option<State<(), R>>;

/// Returns the Yield variant with a Unit type of [State](gen/enum.State.html) to indicate the Generator has yielded.
/// Only returns a Return<R> if the Generator has returned.
/// Any further calls to resume should return None.
pub trait GenOnce {
    type Return;
    
    fn resume(&mut self) -> ResumeOnce<Self::Return>;
}

pub type Resume<Y, R> = Option<State<Y, R>>;

/// Returns the Yield variant of [State](gen/enum.State.html) when the Generator yields with the yielded item,
/// and the Return variant of [State](gen/enum.State.html) when the Generator returns, with the returned item.
/// Any further calls to resume_with_yield should return None.
pub trait Gen: GenOnce {
    type Yield;
    
    fn resume_with_yield(&mut self) -> Resume<Self::Yield, Self::Return>;
}

/// A safe wrapper around a Generator.
/// Once the Generator is returned, it's guaranteed that resume() is never called again on the Generator.
pub struct Callable<G>(Option<G>);

impl<G> Callable<G> {
    
    #[inline]
    pub fn new(g: G) -> Self {
        Callable(Some(g))
    }

    /// Composes a new Callable. this function takes a closure that takes the return value of the underlying Generator and returns a new Generator,
    /// This function consumes the underlying generator of `self`, calls the function with the returned item, and returns a new callable containing the new
    /// generator.
    pub fn compose<O>(self, g: impl FnOnce(G::Return) -> O) -> Option<Callable<impl Generator<Yield = G::Yield, Return = G::Return>>>
    where
        G: Generator,
        O: Generator<Yield = G::Yield, Return = G::Return>,
    {
        let mut generator = self.into_inner()?;
        
        Some(Callable::new(move || {
            let ret = yield_from!(generator);

            let mut provided_gen = g(ret);

            return yield_from!(provided_gen)
        }))
    }

    // pub fn chain<Y, R, O>(mut self, g: impl FnOnce(G::Return) -> O) -> Option<Callable<impl Generator<Yield = O::Yield, Return = O::Return >>>
    // where
    //     G: Generator,
    //     O: Generator,
    // {
    //     let ret = loop {
    //         match self.resume_with_yield() {
    //             Some(State::Yield(_)) => continue,
    //             Some(State::Return(ret)) => break ret,
    //             None => return None,
    //         };
    //     };

    //     Some(Callable::new(move || {

    //         let mut provided_gen = g(ret);

    //         return yield_from!(provided_gen)
    //     }))
    // }

    #[inline]
    pub fn into_inner(self) -> Option<G> {
        self.0
    }

    pub fn take(&mut self) -> Option<G> {
        self.0.take()   
    }
}

impl <G> GenOnce for Callable<G>
where
    G: Generator
{
    type Return = G::Return;
    
    #[inline]
    fn resume(&mut self) -> ResumeOnce<Self::Return> {
        let r = return_from_yield!(self.0.as_mut()?);
        
        self.0.take();
        return Some(State::Return(r));
    }
}

impl <'a, G> GenOnce for &'a mut G
where
    G: GenOnce
{
    type Return = G::Return;

    #[inline]
    fn resume(&mut self) -> ResumeOnce<Self::Return> {
        (*self).resume()
    }
}

impl <G> Gen for Callable<G>
where
    G: Generator
{
    type Yield = G::Yield;
    
    #[inline]
    fn resume_with_yield(&mut self) -> Resume<Self::Yield, Self::Return> {
        let r = return_yielded!(self.0.as_mut()?);
        self.0.take();
        return Some(State::Return(r));
    }
}

impl <'a, G> Gen for &'a mut G
where
    G: Gen
{
    type Yield = G::Yield;

    #[inline]
    fn resume_with_yield(&mut self) -> Resume<Self::Yield, Self::Return> {
        (*self).resume_with_yield()
    }
}