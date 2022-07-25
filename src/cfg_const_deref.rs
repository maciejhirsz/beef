macro_rules! cfg_const_deref {
    ( $($tt:tt)* ) => { crate::cfg_const_deref_munch! { [] [$($tt)*] } };
}

#[cfg(not(feature = "const_deref"))]
macro_rules! cfg_const_deref_munch {
    ( [$($done:tt)*] [] ) => { $($done)* };

    ( [$($done:tt)*] [*const      $($todo:tt)*] ) => { crate::cfg_const_deref_munch! { [$($done)* *const] [$($todo)*] } };
    ( [$($done:tt)*] [~const      $($todo:tt)*] ) => { crate::cfg_const_deref_munch! { [$($done)*       ] [$($todo)*] } };
    ( [$($done:tt)*] [ const      $($todo:tt)*] ) => { crate::cfg_const_deref_munch! { [$($done)*       ] [$($todo)*] } };
    ( [$($done:tt)*] [{$($a:tt)*} $($todo:tt)*] ) => { crate::cfg_const_deref_munch! { [$($done)* { crate::cfg_const_deref!{$($a)*} }] [$($todo)*] } };
    ( [$($done:tt)*] [   $a:tt    $($todo:tt)*] ) => { crate::cfg_const_deref_munch! { [$($done)* $a    ] [$($todo)*] } };
}

#[cfg(feature = "const_deref")]
macro_rules! cfg_const_deref_munch {
    ( [$($done:tt)*] [$($todo:tt)*] ) => { $($done)* $($todo)* };
}

pub(crate) use cfg_const_deref;
pub(crate) use cfg_const_deref_munch;
