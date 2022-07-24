// macro for hack this problem: https://github.com/rust-lang/rust/issues/99675
// also can use `cfg_if` crate
macro_rules! cfg_fix {
    (#[cfg($meta:meta)] { $($tokens:tt)* }) => {
        #[cfg($meta)] $crate::cfg_fix! { @identity $($tokens)* }
    };

    (@identity $($tokens:tt)*) => {
        $($tokens)*
    };
}

pub(crate) use cfg_fix;
