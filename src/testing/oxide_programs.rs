pub const EXAMPLE_PROGRAM: &'static str = "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
    }
";