use core::fmt::Debug;
use core::mem;
use core::task::Waker;

use parking_lot::const_mutex;
use parking_lot::Mutex;

#[derive(Debug)]
enum InnerState<Input, Output> {
    Ongoing(Option<Input>, Option<Waker>),

    /// The awaitable is done
    Done(Output),

    Consumed,
}
impl<Input, Output> InnerState<Input, Output> {
    const fn new(input: Option<Input>) -> Self {
        InnerState::Ongoing(input, None)
    }
}

#[derive(Debug)]
pub struct Awaitable<Input, Output>(Mutex<InnerState<Input, Output>>);

impl<Input, Output> Awaitable<Input, Output> {
    pub const fn new(input: Option<Input>) -> Self {
        Self(const_mutex(InnerState::new(input)))
    }
}

impl<Input: Debug, Output: Debug> Awaitable<Input, Output> {
    pub fn reset(&self, input: Option<Input>) {
        *self.0.lock() = InnerState::Ongoing(input, None);
    }

    /// Return true if the task is already done.
    ///
    /// **
    /// `install_waker` must not be registered twice.
    /// `install_waker` must not be called after `take_output` is called.
    /// **
    pub fn install_waker(&self, waker: Waker) -> bool {
        use InnerState::*;

        let mut guard = self.0.lock();

        match &mut *guard {
            Ongoing(_input, stored_waker) => {
                if stored_waker.is_some() {
                    panic!("Waker is installed twice before the awaitable is done");
                }
                *stored_waker = Some(waker);
                false
            }
            Done(_) => true,
            Consumed => {
                panic!("Waker is installed after the awaitable is done and its result consumed")
            }
        }
    }

    /// **`take_input` must not be called after `take_output` is called.
    pub fn take_input(&self) -> Option<Input> {
        use InnerState::*;

        let mut guard = self.0.lock();

        match &mut *guard {
            Ongoing(input, _stored_waker) => input.take(),
            Done(_) => None,
            Consumed => {
                panic!("Task attempts to retrieve input after the awaitable is done and its result consumed")
            }
        }
    }

    /// `done` must be only called once on one instance of `Awaitable`.
    ///
    /// **`done` must not be called after `take_output` is called.**
    pub fn done(&self, value: Output) {
        use InnerState::*;

        let prev_state = mem::replace(&mut *self.0.lock(), Done(value));

        match prev_state {
            Done(_) => panic!("Awaitable is marked as done twice"),
            Ongoing(_input, stored_waker) => {
                if let Some(waker) = stored_waker {
                    waker.wake();
                }
            }
            Consumed => {
                panic!("Awaitable is marked as done again after its result consumed")
            }
        };
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
}
