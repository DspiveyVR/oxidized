use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);
pub mod static_analysis;
pub mod ast;
pub mod debugging;
pub mod testlib;

#[macro_export]
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    };
}