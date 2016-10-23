pub use unreachable::unreachable as unreachable;

#[macro_export]
macro_rules! fast_unreachable {
    () => (
        if cfg!( debug_assertions ) {
            unreachable!();
        } else {
            $crate::unreachable();
        }
    )
}
