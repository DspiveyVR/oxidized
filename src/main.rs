mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

fn main() {
    println!("{}", oxidized::BlockParser::new().parse("
    {

    struct Obj(u32);
    letrgn<'y, 'z> {
        let mut x = Obj(5);
        let y = &'y uniq x;
        let z = &'z uniq *y;
    }

    }
    ").unwrap());
}


#[test]
fn oxidized() {
}
