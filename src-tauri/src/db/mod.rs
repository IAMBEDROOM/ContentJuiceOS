mod connection;
pub mod error;
pub(crate) mod migration;

pub use connection::Database;
#[allow(unused_imports)]
pub use error::{DbError, DbResult};
