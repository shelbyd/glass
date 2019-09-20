use core::marker::PhantomPinned;
use core::pin::Pin;

pub struct Pinned<T> {
    t: T,
    pinned_to: Option<*const Self>,
    _pinned: PhantomPinned,
}

impl<T> Pinned<T> {
    pub fn new(t: T) -> Self {
        Pinned {
            t,
            pinned_to: None,
            _pinned: PhantomPinned,
        }
    }

    pub fn check(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        match this.pinned_to {
            Some(ptr) => {
                if ptr != this {
                    panic!("Pinned moved after pinned");
                }
            }
            None => this.pinned_to = Some(this),
        }
    }

    pub fn pin(self) -> Pin<Box<Self>> {
        let mut pinned = Box::pin(self);
        let location = Pin::get_ref(pinned.as_ref());
        unsafe { Pin::get_unchecked_mut(pinned.as_mut()) }.pinned_to = Some(location);
        pinned
    }
}

impl<T> Drop for Pinned<T> {
    fn drop(&mut self) {
        match self.pinned_to {
            Some(ptr) => {
                if ptr != self {
                    if !::std::thread::panicking() {
                        panic!("Pinned moved before drop");
                    }
                }
            }
            None => {}
        }
    }
}

impl<T: Default> Default for Pinned<T> {
    fn default() -> Self {
        Pinned {
            _pinned: PhantomPinned,
            pinned_to: None,
            t: T::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Pinned moved after pinned")]
    fn move_after_pin_panics() {
        let mut p = Pinned::new(42);

        unsafe { core::pin::Pin::new_unchecked(&mut p) }.check();

        let mut moved = p;

        // Should panic because moved.
        unsafe { core::pin::Pin::new_unchecked(&mut moved) }.check();
    }

    #[test]
    fn no_move_check_is_fine() {
        let mut p = Pinned::new(42);

        unsafe { core::pin::Pin::new_unchecked(&mut p) }.check();
        unsafe { core::pin::Pin::new_unchecked(&mut p) }.check();
        unsafe { core::pin::Pin::new_unchecked(&mut p) }.check();
    }

    #[test]
    #[should_panic(expected = "Pinned moved before drop")]
    fn move_before_drop_panics() {
        let mut p = Pinned::new(42);

        unsafe { core::pin::Pin::new_unchecked(&mut p) }.check();

        let _moved = p;
        // Panics at end of scope because moved.
    }

    #[test]
    fn drop_never_pinned_is_fine() {
        let p = Pinned::new(42);

        let _moved = p;
    }

    #[test]
    #[should_panic(expected = "Pinned moved before drop")]
    fn mem_swap_fails() {
        fn naughty<T: Default>(pinned: Pin<&mut T>) {
            let unchecked_pin = unsafe { pinned.get_unchecked_mut() };
            core::mem::swap(unchecked_pin, &mut T::default());
        }

        let p = Pinned::new(42);
        naughty(p.pin().as_mut());
    }
}
