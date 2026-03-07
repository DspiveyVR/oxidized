mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

fn main() {
    // println!("{:?}", oxidized::StatementsParser::new().parse("let test = 123; let test2 = false;").unwrap());
    println!("{:?}", oxidized::PlaceExprParser::new().parse("*test.2.3.4").unwrap());
}


#[test]
fn oxidized() {
}
