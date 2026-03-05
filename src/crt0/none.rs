// Hosted 模式下的打印宏：直接绕过微内核服务，使用宿主机的标准输出
// 这符合 "hutch使用系统自身的stdio" 的设计要求
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ( {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_fmt(format_args!($($arg)*));
        let _ = stdout.flush();
    } );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
