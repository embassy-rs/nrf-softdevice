use core::mem;

pub struct DropBomb {
    _private: (),
}

impl DropBomb {
    #[allow(unused)]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[allow(unused)]
    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        panic!("boom")
    }
}
