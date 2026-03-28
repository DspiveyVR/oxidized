mod ast;
mod static_analysis;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

fn main() {
    println!(
        "{}",
        oxidized::ExprParser::new()
            .parse(
                "

    letrgn<'y, 'z> {
        let mut x = (5, 4);
        let y = &'y uniq x;
        let z = &'z uniq *y; ()
    }

    "
            )
            .unwrap()
    );
}

#[test]
fn oxidized() {}
