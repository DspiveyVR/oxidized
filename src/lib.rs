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

fn test() {
    struct A<'a> {
        a: i32,
        b: &'a i32
    }

    // let r = 5;
    // let a = A { a: 4, b: &r};
    let a = 5;
    let mut b = &a;
    let c = &mut b;
    *c;
}