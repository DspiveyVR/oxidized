use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);
pub mod ast;
pub mod debugging;
pub mod static_analysis;
pub mod testlib;

#[macro_export]
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    };
}

// fn test() {
//     let mut x: u32 = 5;
//     let mut y: &mut u32 = &mut x;
//     let mut z: &mut u32 = y;
//     let mut w: &mut &mut u32 = &mut y;
// }
