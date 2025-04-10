use crate::{row::Row, Result};


pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self>;
}

