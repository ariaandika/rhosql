use crate::{row::{Decode, Row}, Result};


pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self>;
}

macro_rules! from_tuple {
    ($($id:ident $i:literal),*) => {
        impl<$($id),*> FromRow for ($($id),*,)
        where
            $($id: for<'a> Decode<'a>),*
        {
            fn from_row(row: Row) -> Result<Self> {
                Ok((
                    $(row.try_column($i)?.try_decode::<$id>()?),*,
                ))
            }
        }
    };
}

from_tuple!(R1 0);
from_tuple!(R1 0,R2 1);
from_tuple!(R1 0,R2 1,R3 2);
from_tuple!(R1 0,R2 1,R3 2,R4 3);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4,R6 5);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4,R6 5,R7 6);

