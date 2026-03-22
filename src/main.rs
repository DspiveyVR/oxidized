mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub oxidized);

struct Obj(String);
fn main() {
    println!(
        "{}",
        oxidized::BlockParser::new()
            .parse(
                "
    {

    struct Obj(u32, u32);
    letrgn<'y, 'z> {
        let mut x = Obj(5, 4);
        let y = &'y uniq x;
        let z = &'z uniq *y;
    }

    }
    "
            )
            .unwrap()
    );

    let mut obj = Obj(String::new());
    let test1 = &mut obj;
}

#[test]
fn oxidized() {}
