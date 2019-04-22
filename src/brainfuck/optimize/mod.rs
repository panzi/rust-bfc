mod fold;
mod set;
mod write;
mod constexpr;

pub use fold::optimize as fold;
pub use set::optimize as set;
pub use write::optimize as write;
pub use constexpr::optimize as constexpr;

pub struct Options {
    pub fold:           bool,
    pub set:            bool,
    pub write:          bool,
    pub constexpr:      bool,
    pub constexpr_echo: bool,
}

impl std::default::Default for Options {
    fn default() -> Self {
        Options {
            fold:           false,
            set:            false,
            write:          false,
            constexpr:      false,
            constexpr_echo: false,
        }
    }
}

impl Options {
    pub fn all() -> Self {
        Options {
            fold:           true,
            set:            true,
            write:          true,
            constexpr:      true,
            constexpr_echo: true,
        }
    }

    pub fn none() -> Self {
        Options {
            fold:           false,
            set:            false,
            write:          false,
            constexpr:      false,
            constexpr_echo: false,
        }
    }
}