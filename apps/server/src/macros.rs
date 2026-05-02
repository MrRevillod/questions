#[macro_export]
macro_rules! attempt_filter {
    ($($field:ident $(: $value:expr)?),* $(,)?) => {
        ::filterstruct::filter!(AttemptFilter, { $($field $(: $value)?),* })
    };
}
