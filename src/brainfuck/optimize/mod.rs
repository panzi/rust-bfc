mod fold;
mod set;
mod add_to;
mod write;
mod deadcode;
mod constexpr;

pub use fold::optimize as fold;
pub use set::optimize as set;
pub use add_to::optimize as add_to;
pub use write::optimize as write;
pub use deadcode::optimize as deadcode;
pub use constexpr::optimize as constexpr;

pub struct Options {
    pub fold:           bool,
    pub set:            bool,
    pub add_to:           bool,
    pub write:          bool,
    pub deadcode:       bool,
    pub constexpr:      bool,
    pub constexpr_echo: bool,
}

impl std::default::Default for Options {
    fn default() -> Self {
        Options {
            fold:           false,
            set:            false,
            add_to:          false,
            write:          false,
            deadcode:       false,
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
            add_to:         true,
            write:          true,
            deadcode:       true,
            constexpr:      true,
            constexpr_echo: true,
        }
    }

    pub fn none() -> Self {
        Options {
            fold:           false,
            set:            false,
            add_to:         false,
            write:          false,
            deadcode:       false,
            constexpr:      false,
            constexpr_echo: false,
        }
    }
}