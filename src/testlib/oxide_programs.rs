use rstest::fixture;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

pub fn check_rust_code(code: &str) -> Result<(), String> {
    let dir = tempdir().map_err(|e| e.to_string())?;
    let file_path = dir.path().join("test_code.rs");

    fs::write(&file_path, code).map_err(|e| e.to_string())?;

    let output = Command::new("rustc")
        .arg("--crate-type=lib")
        .arg("--crate-name=test_crate")
        .arg("--emit=metadata")
        .arg("-A")
        .arg("warnings")
        .arg("--out-dir")
        .arg(dir.path())
        .arg(&file_path)
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        let temp_prefix = format!("{}/", dir.path().to_string_lossy());
        let normalized_stderr = stderr.replace(&temp_prefix, "");

        Err(normalized_stderr)
    }
}

pub struct TestProgram {
    pub oxide: &'static str,
    pub rust: &'static str,
    pub program_name: &'static str,
}

#[fixture]
pub fn double_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; 
            let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            ()
        }",
        program_name: "double_ref",
    }
}

#[fixture]
pub fn double_ref2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'y, 'x> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; 
            let z: &'y uniq &'x uniq u32 = &'y uniq y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            ()
        }",
        program_name: "double_ref2",
    }
}

#[fixture]
pub fn double_ref_bad() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; 
            let z: &'y uniq u32 = &'y uniq y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = &mut y;
            ()
        }",
        program_name: "double_ref_bad",
    }
}

#[fixture]
pub fn triple_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; 
            let z: &'y uniq &'x uniq u32 = &'y uniq y;
            let w: &'z uniq &'y uniq &'x uniq u32 = &'z uniq z; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            let mut w: &mut &mut &mut u32 = &mut z;
            ()
        }",
        program_name: "triple_ref",
    }
}

#[fixture]
pub fn triple_ref2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x; 
            let z: &'y uniq &'x uniq u32 = &'y uniq y;
            let w: &'z shrd &'y uniq &'x uniq u32 = &'z shrd z; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            let mut w: &&mut &mut u32 = &z;
            ()
        }",
        program_name: "triple_ref2",
    }
}

#[fixture]
pub fn ref_rr() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'y shrd x; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            ()
        }",
        program_name: "ref_rr",
    }
}

#[fixture]
pub fn ref_rr_scopes() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x> {
            let x: u32 = 5;
            letrgn<'z> {
                let y: &'z shrd u32 = &'x shrd x; ()
            }
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            {
                let mut y: &u32 = &x;
                ()
            }
        }",
        program_name: "ref_rr_scopes",
    }
}

#[fixture]
pub fn ref_rr_scopes_bad() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x> {
            let x: u32 = 5;
            letrgn<'z> {
                let y: &'x shrd u32 = &'z shrd x; ()
            }
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            {
                let mut y: &u32 = &x;
                ()
            }
        }",
        program_name: "ref_rr_scopes_bad",
    }
}

#[fixture]
pub fn unsafe_place() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq u32 = &'y uniq x; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = &mut x;
            ()
        }",
        program_name: "unsafe_place",
    }
}

#[fixture]
pub fn gc_test() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x> {
            let x: u32 = 5;
            letrgn<'z> {
                let y: &'x shrd u32 = &'x shrd x; ()
            };
            let z: &'x uniq u32 = &'x uniq x; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            {
                let mut y: &u32 = &x;
                ()
            };
            let mut z: &mut u32 = &mut x;
            ()
        }",
        program_name: "gc_test",
    }
}

#[fixture]
pub fn gc_test2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y;
            let z: &'x uniq u32 = &'x uniq x; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y;
            let mut z: &mut u32 = &mut x;
            ()
        }",
        program_name: "gc_test2",
    }
}

#[fixture]
pub fn move_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq u32 = y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = y;
            ()
        }",
        program_name: "move_ref",
    }
}

#[fixture]
pub fn move_ref_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq u32 = y;
            let w: &'z uniq &'x uniq u32 = &'z uniq y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = y;
            let mut w: &mut &mut u32 = &mut y;
            ()
        }",
        program_name: "move_ref_err",
    }
}

#[fixture]
pub fn copy_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            let z: &'y shrd u32 = y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            let mut z: &u32 = y;
            ()
        }",
        program_name: "copy_ref",
    }
}

#[fixture]
pub fn copy_ref_usage() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y, 'z> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            let z: &'y shrd u32 = y;
            let w: &'z shrd &'x shrd u32 = &'z shrd y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            let mut z: &u32 = y;
            let mut w: &&u32 = &y;
            ()
        }",
        program_name: "copy_ref_usage",
    }
}

#[fixture]
pub fn move_to_place() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y;
            ()
        }",
        program_name: "move_to_place",
    }
}

#[fixture]
pub fn move_to_place2() -> TestProgram {
    TestProgram {
        oxide: "
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
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                y;
                ()
            };
            {
                let mut z: u32 = 3;
                ()
            }
        }",
        program_name: "move_to_place2",
    }
}

#[fixture]
pub fn move_to_place3() -> TestProgram {
    TestProgram {
        oxide: "
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
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                y
            };
            {
                let mut z: u32 = 3;
                let mut w: &mut u32 = &mut z;
                {
                    let mut a: &mut u32 = &mut x;
                    w
                };
                let mut b: &mut u32 = &mut z;
            }
        }",
        program_name: "move_to_place3",
    }
}

#[fixture]
pub fn move_to_place3_err() -> TestProgram {
    TestProgram {
        oxide: "
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
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                y
            };
            {
                let mut z: u32 = 3;
                let mut w: &mut u32 = &mut z;
                {
                    let mut a: &mut u32 = &mut x;
                    w
                };
                let mut b: &mut u32 = w;
            }
        }",
        program_name: "move_to_place3_err",
    }
}

#[fixture]
pub fn complex_rewrite() -> TestProgram {
    TestProgram {
        oxide: "
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
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                y
            };
            {
                let mut z: u32 = 3;
                let mut w: &mut u32 = &mut z;
                {
                    let mut a: &mut u32 = &mut x;
                    let mut b: &mut u32 = w;
                }
            }
        }",
        program_name: "complex_rewrite",
    }
}

#[fixture]
pub fn move_to_place_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y;
            let w: &'z uniq &'x uniq u32 = &'z uniq y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y;
            let mut w: &mut &mut u32 = &mut y;
            ()
        }",
        program_name: "move_to_place_err",
    }
}

#[fixture]
pub fn move_to_place_err2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y; y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y;
            y;
            ()
        }",
        program_name: "move_to_place_err2",
    }
}

#[fixture]
pub fn copy_to_place() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            y;
            ()
        }",
        program_name: "copy_to_place",
    }
}

#[fixture]
pub fn copy_to_place2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            y; y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            y;
            y;
            ()
        }",
        program_name: "copy_to_place2",
    }
}

#[fixture]
pub fn copy_to_place3() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            y; y; ()
        }; ()
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            y;
            y;
            ();
        }",
        program_name: "copy_to_place3",
    }
}

#[fixture]
pub fn copy_to_place_usage() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x,  'y, 'z> {
            let x: u32 = 5;
            let y: &'x shrd u32 = &'x shrd x;
            y;
            let w: &'z shrd &'x shrd u32 = &'z shrd y; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &u32 = &x;
            y;
            let mut w: &&u32 = &y;
            ()
        }",
        program_name: "copy_to_place_usage",
    }
}

#[fixture]
pub fn uniq_lifetime() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x> {
            let x: u32 = 5;
            letrgn<'y> {
                let z: &'y uniq u32 = &'y uniq x
            };
            let y: &'x uniq u32 = &'x uniq x
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            {
                let mut z: &mut u32 = &mut x;
            };
            let mut y: &mut u32 = &mut x;
        }",
        program_name: "uniq_lifetime",
    }
}

#[fixture]
pub fn move_inner_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let a: &'x uniq u32 = &'x uniq x;
            let b: &'y uniq &'x uniq u32 = &'y uniq a;
            a
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut a: &mut u32 = &mut x;
            let mut b: &mut &mut u32 = &mut a;
            a;
        }",
        program_name: "move_inner_ref",
    }
}

#[fixture]
pub fn complex_nesting() -> TestProgram {
    TestProgram {
        oxide: "
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
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            {
                {
                    let mut x_copy: u32 = 5;
                    let mut a: &mut u32 = &mut x_copy;
                    let mut b: &mut u32 = a;
                };
                let mut c: &mut u32 = &mut x;
            };
            let mut a: &mut u32 = &mut x;
            {
                let mut b: &&mut u32 = &a;
                {
                    let mut c: &&mut u32 = &a;
                    {
                        let mut d: &&mut u32 = c;
                        let mut e: &mut & &mut u32 = &mut d;
                    }
                }
            };
            let mut b: &mut u32 = a;
        }",
        program_name: "complex_nesting",
    }
}

#[fixture]
pub fn primitive_assignment() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            x = 6
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            x = 6;
        }",
        program_name: "primitive_assignment",
    }
}

#[fixture]
pub fn assign_under_ref_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            x = 6
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            x = 6;
        }",
        program_name: "assign_under_ref_err",
    }
}

#[fixture]
pub fn reassign_ref_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y = &'y uniq x
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y = &mut x;
        }",
        program_name: "reassign_ref_err",
    }
}

#[fixture]
pub fn reassign_ref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let z: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            y = &'y uniq z;
            let w: &'x uniq u32 = &'x uniq x
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut z: u32 = 5;
            let mut y: &mut u32 = &mut x;
            y = &mut z;
            let mut w: &mut u32 = &mut x;
        }",
        program_name: "reassign_ref",
    }
}

#[fixture]
pub fn dangling_ref_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            letrgn<'z> {
                let z: u32 = 5;
                y = &'y uniq z; ()
            }; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                let mut z: u32 = 5;
                y = &mut z;
                ()
            };
            ()
        }",
        program_name: "dangling_ref_err",
    }
}

#[fixture]
pub fn dangling_ref_err2() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y, 'w> {
            let x: u32 = 5;
            let w: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            letrgn<'z> {
                let z: u32 = 5;
                y = &'y uniq z; 
                y = &'w uniq w
            }; ()
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut w: u32 = 5;
            let mut y: &mut u32 = &mut x;
            {
                let mut z: u32 = 5;
                y = &mut z;
                y = &mut w;
            };
            ()
        }",
        program_name: "dangling_ref_err2",
    }
}

#[fixture]
pub fn basic_deref() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: u32 = *y
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: u32 = *y;
        }",
        program_name: "basic_deref",
    }
}

#[fixture]
pub fn reborrow() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq u32 = &'y uniq *y;
            z
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = &mut *y;
            z;
        }",
        program_name: "reborrow",
    }
}

#[fixture]
pub fn reborrow_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq u32 = &'y uniq *y;
            y
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut u32 = &mut *y;
            y;
        }",
        program_name: "reborrow_err",
    }
}

#[fixture]
pub fn reborrow_move_ref_err() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq &'x uniq u32 = &'y uniq y;
            let w: &'z uniq &'x uniq u32 = &'z uniq *z;
            *w
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            let mut w: &mut &mut u32 = &mut *z;
            *w;
        }",
        program_name: "reborrow_move_ref_err",
    }
}

#[fixture]
pub fn deref2x() -> TestProgram {
    TestProgram {
        oxide: "
        letrgn<'x, 'y, 'z> {
            let x: u32 = 5;
            let y: &'x uniq u32 = &'x uniq x;
            let z: &'y uniq &'x uniq u32 = &'y uniq y;
            let w: u32 = **z
        }
        ",
        rust: "
        fn main() {
            let mut x: u32 = 5;
            let mut y: &mut u32 = &mut x;
            let mut z: &mut &mut u32 = &mut y;
            let mut w: u32 = **z;
        }",
        program_name: "deref2x",
    }
}
