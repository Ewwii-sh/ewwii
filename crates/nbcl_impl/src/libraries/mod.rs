mod corelib;
mod apilib;

pub use corelib::*;
pub use apilib::*;

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
