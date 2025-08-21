use rhai::plugin::*;
use rhai::EvalAltResult;

#[export_module]
pub mod math {
    // math constants
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const TAU: f64 = std::f64::consts::TAU;

    // basics
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }

    #[rhai_fn(return_raw)]
    pub fn sqrt(x: f64) -> Result<f64, Box<EvalAltResult>> {
        if x < 0.0 {
            Err("Math error: sqrt of negative number".into())
        } else {
            Ok(x.sqrt())
        }
    }

    pub fn pow(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }

    pub fn sin(x: f64) -> f64 {
        x.sin()
    }
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }
    pub fn tan(x: f64) -> f64 {
        x.tan()
    }

    // exponentiation
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }

    #[rhai_fn(return_raw)]
    pub fn ln(x: f64) -> Result<f64, Box<EvalAltResult>> {
        if x <= 0.0 {
            Err("Math error: ln of non-positive number".into())
        } else {
            Ok(x.ln())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn log10(x: f64) -> Result<f64, Box<EvalAltResult>> {
        if x <= 0.0 {
            Err("Math error: log10 of non-positive number".into())
        } else {
            Ok(x.log10())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn log(base: f64, x: f64) -> Result<f64, Box<EvalAltResult>> {
        if base <= 0.0 || base == 1.0 {
            Err("Math error: invalid base for log".into())
        } else if x <= 0.0 {
            Err("Math error: log of non-positive number".into())
        } else {
            Ok(x.log(base))
        }
    }

    // advanced
    #[rhai_fn(return_raw)]
    pub fn asin(x: f64) -> Result<f64, Box<EvalAltResult>> {
        if x < -1.0 || x > 1.0 {
            Err("Math error: asin input out of range [-1,1]".into())
        } else {
            Ok(x.asin())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn acos(x: f64) -> Result<f64, Box<EvalAltResult>> {
        if x < -1.0 || x > 1.0 {
            Err("Math error: acos input out of range [-1,1]".into())
        } else {
            Ok(x.acos())
        }
    }

    pub fn atan(x: f64) -> f64 {
        x.atan()
    }
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    pub fn sinh(x: f64) -> f64 {
        x.sinh()
    }
    pub fn cosh(x: f64) -> f64 {
        x.cosh()
    }
    pub fn tanh(x: f64) -> f64 {
        x.tanh()
    }

    pub fn floor(x: f64) -> f64 {
        x.floor()
    }
    pub fn ceil(x: f64) -> f64 {
        x.ceil()
    }
    pub fn round(x: f64) -> f64 {
        x.round()
    }
    pub fn trunc(x: f64) -> f64 {
        x.trunc()
    }
    pub fn fract(x: f64) -> f64 {
        x.fract()
    }

    pub fn min(a: f64, b: f64) -> f64 {
        a.min(b)
    }
    pub fn max(a: f64, b: f64) -> f64 {
        a.max(b)
    }
    pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
        x.clamp(min, max)
    }

    // other api's
    #[rhai_fn(return_raw)]
    pub fn to_float(x: Dynamic) -> Result<f64, Box<EvalAltResult>> {
        if let Some(f) = x.clone().try_cast::<f64>() {
            Ok(f)
        } else if let Some(i) = x.try_cast::<i64>() {
            Ok(i as f64)
        } else {
            Err("Expected a number".into())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn to_int(x: Dynamic) -> Result<i64, Box<EvalAltResult>> {
        if let Some(i) = x.clone().try_cast::<i64>() {
            Ok(i)
        } else if let Some(f) = x.try_cast::<f64>() {
            Ok(f as i64)
        } else {
            Err("Expected a number".into())
        }
    }
}
