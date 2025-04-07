
pub struct General(pub String);

macro_rules! general {
    ($($tt:tt)*) => {
        crate::common::General(format!($($tt)*))
    };
}

pub(crate) use general;

impl std::error::Error for General { }

impl std::fmt::Display for General {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for General {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

