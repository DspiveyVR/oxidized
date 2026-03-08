mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

fn main() {
    // println!("{:?}", oxidized::StatementsParser::new().parse("let test = 123; let test2 = false;").unwrap());
    println!("{:?}", oxidized::ExprParser::new().parse("(id.1)").unwrap());
}


#[test]
fn oxidized() {
}
