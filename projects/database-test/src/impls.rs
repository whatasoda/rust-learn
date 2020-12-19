use crate::database::DatabaseResponse;
use crate::wasm_utils::LinerJavaScriptOutput;
use crate::LinerJavaScriptInput;
use std::mem;

pub enum ValueKind {
    Vec = 0,
    String = 1,
    DatabaseResponse = 2,
}

impl<T> LinerJavaScriptInput for Vec<T> {
    type Liner = (usize, usize);
    fn from_liner((ptr, len): Self::Liner) -> Vec<T> {
        unsafe { Vec::<T>::from_raw_parts(ptr as *mut T, len, len) }
    }
}
impl<T> LinerJavaScriptOutput for Vec<T>
where
    T: Sized,
{
    const KIND: u32 = ValueKind::Vec as u32;
    type Liner = (usize, usize, usize);
    fn to_liner(&self) -> Self::Liner {
        (self.as_ptr() as usize, self.len(), mem::size_of::<T>())
    }
}

impl LinerJavaScriptInput for String {
    type Liner = (usize, usize);
    fn from_liner((ptr, len): Self::Liner) -> String {
        unsafe { String::from_raw_parts(ptr as *mut u8, len, len) }
    }
}
impl LinerJavaScriptOutput for String {
    const KIND: u32 = ValueKind::String as u32;
    type Liner = (usize, usize);
    fn to_liner(&self) -> Self::Liner {
        (self.as_ptr() as usize, self.len())
    }
}

impl LinerJavaScriptOutput for DatabaseResponse {
    const KIND: u32 = ValueKind::DatabaseResponse as u32;
    type Liner = (u32, usize, usize);
    fn to_liner(&self) -> Self::Liner {
        match self {
            Self::JSON(s) => {
                let s = s.to_liner();
                (0, s.0, s.1)
            }
            Self::Snapshot(v) => {
                let v = v.to_liner();
                (1, v.0, v.1)
            }
            Self::Uninitialized => (2, 0, 0),
            Self::SerializationError => (3, 0, 0),
            Self::UnknownQueryError => (4, 0, 0),
            Self::UnknownSaveError => (5, 0, 0),
        }
    }
}

pub mod query {
    pub mod id {
        use crate::query::query::id::{FilterPolicy, IdFilterInput, QueryInput};
        use crate::LinerJavaScriptInput;
        impl LinerJavaScriptInput for QueryInput {
            type Liner = (u32, u32, usize, usize);
            fn from_liner((kind, policy, ptr, len): Self::Liner) -> Self {
                let list = Vec::<u32>::from_liner((ptr, len));
                let policy = match policy {
                    0 => Some(FilterPolicy::Include),
                    1 => Some(FilterPolicy::Exclude),
                    _ => None,
                };
                match (kind, policy) {
                    (0, Some(policy)) => Self::GameId(IdFilterInput { policy, list }),
                    (1, Some(policy)) => Self::TagId(IdFilterInput { policy, list }),
                    _ => panic!(),
                }
            }
        }
    }

    pub mod recommendation {
        use crate::query::query::recommendation::{
            ComplexRangeInput, QueryInput, RangeFormat, SimpleRangeInput,
        };
        use crate::LinerJavaScriptInput;
        impl LinerJavaScriptInput for QueryInput {
            type Liner = (u32, u32, u32, i32, i32);
            fn from_liner((kind, format, baseline, min, max): Self::Liner) -> Self {
                let format = match format {
                    0 => None,
                    1 => Some(RangeFormat::Pct { baseline }),
                    2 => Some(RangeFormat::Count),
                    _ => None,
                };
                let range = SimpleRangeInput { min, max };
                match (kind, format) {
                    (0, None) => Self::Date(range),
                    (1, None) => Self::Total(range),
                    (2, Some(format)) => Self::Up(ComplexRangeInput { format, range }),
                    (3, Some(format)) => Self::Down(ComplexRangeInput { format, range }),
                    (4, Some(format)) => Self::Sum(ComplexRangeInput { format, range }),
                    _ => panic!("Invalid Input"),
                }
            }
        }
    }
}
