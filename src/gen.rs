use std::ops::Generator;
use std::ops::GeneratorState;

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

/// Returns the Yield variant of [State](gen/enum.State.html) containing a `()`, to indicate the Generator has yielded.
/// Only returns a Return<R> if the Generator has returned.
/// Any further calls to [`resume`](trait.GenOnce.html#method.resume) should return None.
pub trait GenOnce {
    type Return;
    
    fn resume(&mut self) -> ResumeOnce<Self::Return>;
}

pub type Resume<Y, R> = Option<State<Y, R>>;

/// Returns the Yield variant of [State](gen/enum.State.html) with the yielded items of the Generator,
/// and the Return variant of [State](gen/enum.State.html) when the Generator returns, with the returned item.
/// Any further calls to [`resume_with_yield`](trait.Gen.html#method.resume_with_yield) should return None.
pub trait Gen: GenOnce {
    type Yield;
    
    fn resume_with_yield(&mut self) -> Resume<Self::Yield, Self::Return>;
}

/// A safe wrapper around a Generator.
/// Once the Generator is returned, it's guaranteed that [`resume`](https://doc.rust-lang.org/1.23.0/std/ops/trait.Generator.html#tymethod.resume) is never called again on the Generator.
pub struct Callable<G>(Option<G>);

impl<G> Callable<G> {
    
    #[inline]
    pub fn new(g: G) -> Self {
        Callable(Some(g))
    }

    /// chains a new Callable. this function takes a closure that takes the return value of the underlying Generator and returns a new Generator,
    /// The newly created Callable has a generator under the hood that first yields all the items of the old generator, once that returns it passes the returned value into the closure,
    /// so a new generator is pulled out of the closure, and that generator will resume from there on.
    /// Returns None if the underlying Generator already has been exhausted.
    pub fn chain<O>(self, g: impl FnOnce(G::Return) -> O) -> Option<Callable<impl Generator<Yield = G::Yield, Return = G::Return>>>
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

    /// Takes out the underlying Generator, and calls the closure with it. The closure should return a new Generator.
    /// Returns None if the underlying Generator already has been exhausted.
    #[inline]
    pub fn move_into<O>(self, func: impl FnOnce(G) -> O) -> Option<Callable<impl Generator<Yield = O::Yield, Return = O::Return>>>
    where
        G: Generator,
        O: Generator,
    {
        let generator = self.into_inner()?;
        Some(Callable::new(func(generator)))
    }

    /// Calls the closure with self. Because `Self` can be turned into an Iterator, it makes iterating over the underlying Generator of self easy to do in the new generator.
    /// Returns None if the underlying Generator already has been exhausted
    #[inline]
    pub fn make_new<O>(self, func: impl FnOnce(Self) -> O) -> Option<Callable<impl Generator<Yield = O::Yield, Return = O::Return>>>
    where
        G: Generator,
        O: Generator
    {
        if self.0.is_some() {
            return Some(Callable::new(func(self)))
        }
        None
    }

    /// Calls the closure, borrowing `Self`. This still allows Iteration inside of the new Generator, except that `Self` is not moved into the closure.
    /// Returns None if the underlying Generator already has been exhausted
    #[inline]
    pub fn borrow_mut<'a, 's: 'a, O>(&'s mut self, func: impl FnOnce(&'a mut Self) -> O) -> Option<Callable<impl Generator<Yield = O::Yield, Return = O::Return>>>
    where
        G: Generator,
        O: Generator
    {
        if self.0.is_some() {
            return Some(Callable::new(func(self)))
        }
        None
    }

    /// Takes the underlying Generator out of self, consuming self.
    /// Returns None if the underlying Generator already has been exhausted
    #[inline]
    pub fn into_inner(self) -> Option<G> {
        self.0
    }

    /// Takes out the underlying Generator, replacing it with None.
    /// This does not consume `Self`.
    /// Returns None if the underlying Generator already has been exhausted
    #[inline]
    pub fn take(&mut self) -> Option<G> {
        self.0.take()   
    }

    /// Returns a mutable reference to the underlying Generator.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut G> {
        self.0.as_mut()
    }
}

impl <G> GenOnce for Callable<G>
where
    G: Generator
{
    type Return = G::Return;
    
    #[inline]
    fn resume(&mut self) -> ResumeOnce<Self::Return> {
        let r = return_from_yield!(self.as_mut()?);
        self.take();
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
        let r = return_yielded!(self.as_mut()?);
        self.take();
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