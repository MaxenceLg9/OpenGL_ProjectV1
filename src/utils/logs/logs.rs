
#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {{
        if false {
            print_base!($($arg)*)
        }
    }};
}

#[macro_export]
macro_rules! print_base {
    ($($arg:tt)*) => {{
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unamed");
            print!("[Thread: {}] ", name);
            println!($($arg)*);
    }};
}