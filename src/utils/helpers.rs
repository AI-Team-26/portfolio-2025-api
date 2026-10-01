/*
use std::any::type_name;

pub trait VecFindExt<T> {
    fn find<F>(&self, predicate: F, predicate_text: &str) -> Result<T, String>
    where
        F: Fn(&T) -> bool,
        T: Clone;
}

impl<T> VecFindExt<T> for Vec<T> {
    fn find<F>(&self, predicate: F, predicate_text: &str) -> Result<T, String>
    where
        F: Fn(&T) -> bool,
        T: Clone,
    {
        self.iter()
            .find(|item| predicate(item))
            .cloned()
            .ok_or_else(|| {
                let type_name = type_name::<T>().rsplit("::").next().unwrap_or("Unknown");
                format!("Item not found in Vec<{type_name}> (filter: `{predicate_text}`)")
            })
    }
}

#[macro_export]
macro_rules! find_by {
    ($vec:expr, |$item:ident| $pred:expr) => {
        $crate::utils::helpers::VecFindExt::find(&$vec, |$item| $pred, stringify!($pred))
    };
}


pub trait ResultVecExt<T, E> {
    fn to_vec(self) -> Result<Vec<T>, E>;
}

impl<I, T, E> ResultVecExt<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn to_vec(self) -> Result<Vec<T>, E> {
        self.collect()
    }
}
    */
