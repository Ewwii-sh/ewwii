mod apilib;
mod corelib;

pub use apilib::*;
pub use corelib::*;

#[macro_export]
macro_rules! runtime_err {
    ($($arg:tt)*) => {
        nbcl::error::NbclError::Runtime {
            message: format!($($arg)*),
            hint: None,
            span: None,
        }
    };
}
