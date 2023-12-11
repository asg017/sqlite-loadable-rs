mod fd;
mod fixed;

// Not all IO Uring ops support fixed file indices, this is kept here for future use (>10-12-2023)
// e.g. Write does not support it.
// Fortunately, File creation and getting its raw fd is O(1), the perceived drawback
// is us not being able to use OpenAt/OpenAt2 to fetch the fd.
pub use fd::OpsFd;
pub use fixed::OpsFixed;
