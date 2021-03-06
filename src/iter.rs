use gen::{Senerator, State};

/// This trait converts any type implementing Gen to an Iterator.
/// The Iterator should only return the Yield variants of [State](../gen/enum.State.html), and ignore the Return variant.
pub trait YieldIterExt: Senerator {
    /// The Iterator returned.
    type Iter: Iterator;

    /// Returns the Iterator.
    /// The Iterator should only return the Yielded items from the underlying Generator
    fn iter_yielded(self) -> Self::Iter;
}

impl<G> YieldIterExt for G
where
    G: Senerator,
{
    type Iter = YieldIterator<Self>;

    fn iter_yielded(self) -> Self::Iter {
        YieldIterator(self)
    }
}

pub struct YieldIterator<G>(G);

impl<G> Iterator for YieldIterator<G>
where
    G: Senerator,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.resume_with_yield() {
            Some(State::Yield(y)) => Some(y),
            _ => None,
        }
    }
}

/// This traits converts any type Implementing Gen<Yield = T, Return = R> into an Iterator, where R: Into<T>.
/// This Iterator also returns the returned item from Gen.
/// This is only possible if the Yield type and Return type are the same, or when the Return type can be transformed into the Yield type.
pub trait ReturnIterExt<Y, R>: Senerator<Yield = Y, Return = R>
where
    R: Into<Y>,
{
    /// The Iterator returned.
    type Iter: Iterator;

    /// Returns the Iterator.
    /// The Iterator should return both the yielded items, as well as the returned item from the underlying Generator.
    fn iter_all(self) -> Self::Iter;
}

impl<Y, R, G> ReturnIterExt<Y, R> for G
where
    G: Senerator<Yield = Y, Return = R>,
    R: Into<Y>,
{
    type Iter = ReturnIterator<Self>;

    fn iter_all(self) -> Self::Iter {
        ReturnIterator(self)
    }
}

pub struct ReturnIterator<G>(G);

impl<Y, R, G> Iterator for ReturnIterator<G>
where
    G: Senerator<Yield = Y, Return = R>,
    R: Into<Y>,
{
    type Item = Y;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.resume_with_yield() {
            Some(state) => state.into(),
            None => None,
        }
    }
}
