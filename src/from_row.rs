use crate::{row::{Decode, Row}, Result};


pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self>;
}

macro_rules! from_tuple {
    ($($id:ident),*) => {
        impl<$($id),*> FromRow for ($($id),*,)
        where
            $($id: for<'a> Decode<'a>),*
        {
            fn from_row(row: Row) -> Result<Self> {
                Ok((
                    $(row.try_column(0)?.try_decode::<$id>()?),*,
                ))
            }
        }
    };
}

from_tuple!(R1);
from_tuple!(R1,R2);
from_tuple!(R1,R2,R3);
from_tuple!(R1,R2,R3,R4);
from_tuple!(R1,R2,R3,R4,R5);
from_tuple!(R1,R2,R3,R4,R5,R6);
from_tuple!(R1,R2,R3,R4,R5,R6,R7);

