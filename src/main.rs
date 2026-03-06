mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

fn main() {
    println!("{:?}", oxidized::ExprParser::new().parse("let test = true; let test2 = false").unwrap());
}


#[test]
fn oxidized() {
}
