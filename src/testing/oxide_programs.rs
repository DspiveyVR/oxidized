use rstest::fixture;

#[fixture]
pub(crate) fn double_ref() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
    }
    "
}

#[fixture]
pub(crate) fn double_ref_bad() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq u32 = &'y uniq y; ()
    }
    "
}

#[fixture]
pub(crate) fn triple_ref() -> &'static str {
    "
    letrgn<'x, 'y, 'z> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y;
        let w: &'z uniq &'y uniq &'x uniq u32 = &'z uniq z; ()
    }
    "
}

#[fixture]
pub(crate) fn triple_ref2() -> &'static str {
    "
    letrgn<'x, 'y, 'z> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y;
        let w: &'z shrd &'y uniq &'x uniq u32 = &'z shrd z; ()
    }
    "
}