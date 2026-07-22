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
pub(crate) fn double_ref2() -> &'static str {
    "
    letrgn<'y, 'x> {
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

#[fixture]
pub(crate) fn ref_rr() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x shrd u32 = &'y shrd x; ()
    }
    "
}

#[fixture]
pub(crate) fn ref_rr_scopes() -> &'static str {
    "
    letrgn<'x> {
        let mut x: u32 = 5;
        letrgn<'z> {
            let y: &'z shrd u32 = &'x shrd x; ()
        }
    }
    "
}

#[fixture]
pub(crate) fn ref_rr_scopes_bad() -> &'static str {
    "
    letrgn<'x> {
        let mut x: u32 = 5;
        letrgn<'z> {
            let y: &'x shrd u32 = &'z shrd x; ()
        }
    }
    "
}

#[fixture]
pub(crate) fn unsafe_place() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = &'y uniq x; ()
    }
    "
}

#[fixture]
pub(crate) fn gc_test() -> &'static str {
    "
    letrgn<'x> {
        let mut x: u32 = 5;
        letrgn<'z> {
            let y: &'x shrd u32 = &'x shrd x; ()
        };
        let z: &'x uniq u32 = &'x uniq x; ()
    }
    "
}

#[fixture]
pub(crate) fn gc_test2() -> &'static str {
    "
    letrgn<'x> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y;
        let z: &'x uniq u32 = &'x uniq x; ()
    }
    "
}

#[fixture]
pub(crate) fn move_ref() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = y; ()
    }
    "
}

#[fixture]
pub(crate) fn move_ref_err() -> &'static str {
    "
    letrgn<'x,  'y, 'z> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = y;
        let w: &'z uniq &'x uniq u32 = &'z uniq y; ()
    }
    "
}

#[fixture]
pub(crate) fn copy_ref() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        let z: &'y shrd u32 = y; ()
    }
    "
}

#[fixture]
pub(crate) fn copy_ref_usage() -> &'static str {
    "
    letrgn<'x,  'y, 'z> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        let z: &'y shrd u32 = y;
        let w: &'z shrd &'x shrd u32 = &'z shrd y; ()
    }
    "
}

#[fixture]
pub(crate) fn move_to_place() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y; ()
    }
    "
}

#[fixture]
pub(crate) fn move_to_place2() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        letrgn<'z> {
            y; ()
        };
        letrgn<'w> {
            let z: u32 = 3; ()
        }  
    }
    "
}

#[fixture]
pub(crate) fn move_to_place_err() -> &'static str {
    "
    letrgn<'x,  'y, 'z> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y;
        let w: &'z uniq &'x uniq u32 = &'z uniq y; ()
    }
    "
}

#[fixture]
pub(crate) fn move_to_place_err2() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y; y; ()
    }
    "
}

#[fixture]
pub(crate) fn copy_to_place() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; ()
    }
    "
}

#[fixture]
pub(crate) fn copy_to_place2() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; y; ()
    }
    "
}

#[fixture]
pub(crate) fn copy_to_place3() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; y; ()
    }; ()
    "
}

#[fixture]
pub(crate) fn copy_to_place_usage() -> &'static str {
    "
    letrgn<'x,  'y, 'z> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y;
        let w: &'z shrd &'x shrd u32 = &'z shrd y; ()
    }
    "
}
