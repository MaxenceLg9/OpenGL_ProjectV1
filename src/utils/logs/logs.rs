#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {{
        {
            if *DEBUG.get().unwrap() {
                println!($($arg)*);
            }
        }
    }};
}