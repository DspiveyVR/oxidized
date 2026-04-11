#[cfg(test)]
mod unit_tests;
#[cfg(test)]
mod oxide_programs;
#[cfg(test)]
mod display_impl_tests;

#[cfg(test)]
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}
#[cfg(test)]
pub(crate) use set_snapshot_suffix;