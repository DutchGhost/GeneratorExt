
#![feature(generator_trait, generators)]

/// A macro that first yields all items in the provided Generator, gives the ability to bind the return value of the Generator to a variable.

#[macro_export]
macro_rules! yield_from {

    ($g:expr) => (
        unsafe {
            use ::std::ops::{GeneratorState, Generator};
            loop {
                match $g.resume() {
                    GeneratorState::Yielded(y) => yield y,
                    GeneratorState::Complete(ret) => break ret,
                }
            }
        }
    );
}

pub mod gen;
pub mod iter;


#[cfg(test)]
mod tests;