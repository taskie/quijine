#[macro_export]
macro_rules! qj_slice {
    [ $($v:expr),* ] => {
        &[$(Into::<$crate::Value>::into($v)),*]
    };
}

#[macro_export]
macro_rules! qj_vec {
    [ $($v:expr),* ] => {
        vec![$(Into::<$crate::Value>::into($v)),*]
    };
}
