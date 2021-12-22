mod error;

use core::fmt::Debug;
use core::mem;
use core::task::Waker;

use parking_lot::const_mutex;
use parking_lot::Mutex;

pub use error::Error;

#[derive(Debug)]
enum InnerState<Input, Output> {
    Uninitialized,

    Ongoing(Option<Input>, Option<Waker>),

    /// The awaitable is done
    Done(Output),

    Consumed,
}

#[derive(Debug)]
pub struct Awaitable<Input, Output>(Mutex<InnerState<Input, Output>>);

impl<Input, Output> Awaitable<Input, Output> {
    /// Create an uninitialized `Awaitable`.
    ///
    /// Must be `reset` before it can be used.
    pub const fn new() -> Self {
        Self(const_mutex(InnerState::Uninitialized))
    }
}

impl<Input: Debug, Output: Debug> Awaitable<Input, Output> {
    /// Reset `Awaitable` to its initial state, equivalent to
    /// calling `Self::new(input)` again.
    ///
    /// After this call, `install_waker`, `take_input` and `done`
    /// can be called again.
    pub fn reset(&self, input: Option<Input>) {
        *self.0.lock() = InnerState::Ongoing(input, None);
    }

    /// Return true if the task is already done.
    ///
    /// **
    /// `install_waker` must not be registered twice.
    /// `install_waker` must not be called after `take_output` is called.
    /// **
    pub fn install_waker(&self, waker: Waker) -> Result<bool, Error> {
        use InnerState::*;

        let mut guard = self.0.lock();

        match &mut *guard {
            Uninitialized => Err(Error::Uninitialized),

            Ongoing(_input, stored_waker) => {
                if stored_waker.is_some() {
                    Err(Error::WakerAlreadyInstalled)
                } else {
                    *stored_waker = Some(waker);
                    Ok(false)
                }
            }
            Done(_) => Ok(true),
            Consumed => Err(Error::AlreadyConsumed),
        }
    }

    /// **`take_input` must not be called after `take_output` is called.
    pub fn take_input(&self) -> Result<Option<Input>, Error> {
        use InnerState::*;

        let mut guard = self.0.lock();

        match &mut *guard {
            Uninitialized => Err(Error::Uninitialized),

            Ongoing(input, _stored_waker) => Ok(input.take()),
            Done(_) => Ok(None),
            Consumed => Err(Error::AlreadyConsumed),
        }
    }

    /// `done` must be only called once on one instance of `Awaitable`.
    ///
    /// **`done` must not be called after `take_output` is called.**
    pub fn done(&self, value: Output) -> Result<(), Error> {
        use InnerState::*;

        let prev_state = mem::replace(&mut *self.0.lock(), Done(value));

        match prev_state {
            Uninitialized => Err(Error::Uninitialized),

            Done(_) => Err(Error::AlreadyDone),
            Ongoing(_input, stored_waker) => {
                if let Some(waker) = stored_waker {
                    waker.wake();
                }

                Ok(())
            }
            Consumed => Err(Error::AlreadyConsumed),
        }
    }

    /// Return `Some(output)` if the awaitable is done.
    pub fn take_output(&self) -> Option<Output> {
        use InnerState::*;

        let prev_state = mem::replace(&mut *self.0.lock(), Consumed);

        match prev_state {
            Done(value) => Some(value),
            _ => None,
        }
    }

    /// Return true if current state is `Done`.
    pub fn is_done(&self) -> bool {
        matches!(&*self.0.lock(), InnerState::Done(_))
    }
}
