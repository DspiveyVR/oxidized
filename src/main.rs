mod ast;
mod static_analysis;

use lalrpop_util::lalrpop_mod;

use crate::static_analysis::data_structures::*;

lalrpop_mod!(pub oxidized);

fn main() {
    let ast = oxidized::ExprParser::new()
        .parse(
            "
        letrgn<'x, 'y> {
            let mut x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; ()
        }
    ",
        )
        .unwrap();

    println!("{}", ast.clone());
    let tc = ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new());
    match tc {
        Ok(tc) => {
            println!(
                "Typecheck result:\n env: {:#?},\n return type: {:#?}",
                tc.0, tc.1
            );
        }
        Err(err) => {
            println!("ERROR: {}", err);
        }
    };
}

#[test]
fn oxidized() {}
