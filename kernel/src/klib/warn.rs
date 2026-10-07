#[macro_export]
macro_rules! warn {
    ($cond:expr, $($arg:tt)*) => {{
        let cond: bool = $cond;
        if cond {
            $crate::log!(
                $crate::tty::log::LogLevel::Warn,
                "WARNING at {}:{}: {}\n",
                file!(),
                line!(),
                format_args!($($arg)*)
            );
        }
        cond
    }};
}
