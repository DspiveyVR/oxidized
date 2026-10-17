use rstest::fixture;

#[fixture]
pub fn double_ref() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
    }
    "
}

#[fixture]
pub fn double_ref2() -> &'static str {
    "
    letrgn<'y, 'x> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
    }
    "
}

#[fixture]
pub fn double_ref_bad() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x; 
        let z: &'y uniq u32 = &'y uniq y; ()
    }
    "
}

#[fixture]
pub fn triple_ref() -> &'static str {
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
pub fn triple_ref2() -> &'static str {
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
pub fn ref_rr() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x shrd u32 = &'y shrd x; ()
    }
    "
}

#[fixture]
pub fn ref_rr_scopes() -> &'static str {
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
pub fn ref_rr_scopes_bad() -> &'static str {
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
pub fn unsafe_place() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = &'y uniq x; ()
    }
    "
}

#[fixture]
pub fn gc_test() -> &'static str {
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
pub fn gc_test2() -> &'static str {
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
pub fn move_ref() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = y; ()
    }
    "
}

#[fixture]
pub fn move_ref_err() -> &'static str {
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
pub fn copy_ref() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        let z: &'y shrd u32 = y; ()
    }
    "
}

#[fixture]
pub fn copy_ref_usage() -> &'static str {
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
pub fn move_to_place() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y; ()
    }
    "
}

#[fixture]
pub fn move_to_place2() -> &'static str {
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
pub fn move_to_place3() -> &'static str {
    "
    letrgn<'x> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        letrgn<'y> {
            y
        };
        letrgn<'y> {
            let z: u32 = 3;
            let w: &'y uniq u32 = &'y uniq z;
            letrgn<'z> { 
                let a: &'x uniq u32 = &'x uniq x;
                w
            };
            let b: &'y uniq u32 = &'y uniq z
        }  
    }
    "
}

#[fixture]
pub fn move_to_place3_err() -> &'static str {
    "
    letrgn<'x> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        letrgn<'y> {
            y
        };
        letrgn<'y> {
            let z: u32 = 3;
            let w: &'y uniq u32 = &'y uniq z;
            letrgn<'z> { 
                let a: &'x uniq u32 = &'x uniq x;
                w
            };
            let b: &'y uniq u32 = w
        }  
    }
    "
}

#[fixture]
pub fn complex_rewrite() -> &'static str {
    "
    letrgn<'x> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        letrgn<'y> {
            y
        };
        letrgn<'y> {
            let z: u32 = 3;
            let w: &'y uniq u32 = &'y uniq z;
            letrgn<'z> { 
                let a: &'x uniq u32 = &'x uniq x;
                let b: &'y uniq u32 = w
            }
        }  
    }
    "
}

#[fixture]
pub fn move_to_place_err() -> &'static str {
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
pub fn move_to_place_err2() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        y; y; ()
    }
    "
}

#[fixture]
pub fn copy_to_place() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; ()
    }
    "
}

#[fixture]
pub fn copy_to_place2() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; y; ()
    }
    "
}

#[fixture]
pub fn copy_to_place3() -> &'static str {
    "
    letrgn<'x,  'y> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y; y; ()
    }; ()
    "
}

#[fixture]
pub fn copy_to_place_usage() -> &'static str {
    "
    letrgn<'x,  'y, 'z> {
        let x: u32 = 5;
        let y: &'x shrd u32 = &'x shrd x;
        y;
        let w: &'z shrd &'x shrd u32 = &'z shrd y; ()
    }
    "
}

#[fixture]
pub fn uniq_lifetime() -> &'static str {
    "
    letrgn<'x> {
        let x: u32 = 5;
        letrgn<'y> {
            let z: &'y uniq u32 = &'y uniq x
        };
        let y: &'x uniq u32 = &'x uniq x
    }
    "
}

#[fixture]
pub fn move_inner_ref() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let a: &'x uniq u32 = &'x uniq x;
        let b: &'y uniq &'x uniq u32 = &'y uniq a;
        a
    }
    "
}

#[fixture]
pub fn complex_nesting() -> &'static str {
    "
    letrgn<'x> {
        let x: u32 = 5;
        letrgn<'y> {
            letrgn<'z> {
                let a: &'z uniq u32 = &'y uniq x;
                let b: &'z uniq u32 = a
            };
            let c: &'y uniq u32 = &'x uniq x
        };
        let a: &'x uniq u32 = &'x uniq x;
        letrgn<'y> {
            let b: &'y shrd &'x uniq u32 = &'y shrd a;
            letrgn<'z> {
                let c: &'z shrd &'x uniq u32 = &'z shrd a;
                letrgn<'w, 'a> {
                    let d: &'w shrd &'x uniq u32 = c;
                    let e: &'a uniq &'w shrd &'x uniq u32 = &'a uniq d
                }
            }
        };
        let b: &'x uniq u32 = a 
    }
    "
}

#[fixture]
pub fn primitive_assignment() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        x = 6
    }
    "
}

#[fixture]
pub fn primitive_assignment_err() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        x = 6
    }
    "
}

#[fixture]
pub fn assign_under_ref_err() -> &'static str {
    "
    letrgn<'x, 'y> {
        let mut x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        x = 6
    }
    "
}

#[fixture]
pub fn reassign_ref_err() -> &'static str { // I think this would need NLL to pass
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let mut y: &'x uniq u32 = &'x uniq x;
        y = &'y uniq x
    }
    "
}

#[fixture]
pub fn reassign_ref() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let z: u32 = 5;
        let mut y: &'x uniq u32 = &'x uniq x;
        y = &'y uniq z;
        let w: &'x uniq u32 = &'x uniq x
    }
    "
}

#[fixture]
pub fn dangling_ref_err() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let mut y: &'x uniq u32 = &'x uniq x;
        letrgn<'z> {
            let z: u32 = 5;
            y = &'y uniq z; ()
        }; ()
    }
    "
}

#[fixture]
pub fn dangling_ref_err2() -> &'static str {
    "
    letrgn<'x, 'y, 'w> {
        let x: u32 = 5;
        let w: u32 = 5;
        let mut y: &'x uniq u32 = &'x uniq x;
        letrgn<'z> {
            let z: u32 = 5;
            y = &'y uniq z; 
            y = &'w uniq w
        }; ()
    }
    "
}

// #[fixture]
// pub fn weird_rr() -> &'static str {
//     "
//     letrgn<'x, 'y> {
//         let x: u32 = 5;
//         let mut y: &'x uniq &'y uniq u32 = &'y uniq &'x uniq x
//     "
// }

#[fixture]
pub fn basic_deref() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: u32 = *y
    }
    "
}

#[fixture]
pub fn reborrow() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = &'y uniq *y;
        z
    }
    "
}

#[fixture]
pub fn reborrow_err() -> &'static str {
    "
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq u32 = &'y uniq *y;
        y
    }
    "
}

#[fixture]
pub fn reborrow_move_ref_err() -> &'static str {
    "
    letrgn<'x, 'y, 'z> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq &'x uniq u32 = &'y uniq y;
        let w: &'z uniq &'x uniq u32 = &'z uniq *z;
        *w
    }
    "
}

#[fixture]
pub fn deref2x() -> &'static str {
    "
    letrgn<'x, 'y, 'z> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq &'x uniq u32 = &'y uniq y;
        let w: u32 = **z
    }
    "
}