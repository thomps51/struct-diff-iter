use std::fmt::{Debug, Write};

use smallvec::SmallVec;

// Represents a field that was not equal within a struct/enum.
pub struct DiffData<'a, 'b> {
    pub field: FieldIdentifier,
    pub self_value: &'a dyn Debug,
    pub other_value: &'b dyn Debug,
}

// Represents the name of a field within a struct/enum.
// Such as field1, or field1.inner_field.inner_inner_field
pub struct FieldIdentifier {
    storage: SmallVec<[&'static str; 4]>,
}

impl FieldIdentifier {
    pub fn new() -> Self {
        FieldIdentifier {
            storage: Default::default(),
        }
    }

    pub fn push(&mut self, string: &'static str) {
        self.storage.push(string)
    }
}

impl ToString for FieldIdentifier {
    fn to_string(&self) -> String {
        let len = self.storage.iter().fold(0, |s, x| s + x.len());
        let mut result = String::with_capacity(len + self.storage.len() - 1);
        let mut delimiter = "";
        // stored in reverse order
        for field in self.storage.iter().rev() {
            result.write_str(&delimiter).unwrap();
            delimiter = ".";
            result.write_str(*field).unwrap();
        }
        result
    }
}

impl Debug for FieldIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // print as field.field.field
        let mut delimiter = "";
        // stored in reverse order
        for field in self.storage.iter().rev() {
            f.write_str(&delimiter)?;
            delimiter = ".";
            f.write_str(*field)?;
        }
        Ok(())
    }
}

pub trait LazyDiff {
    fn struct_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData>;
}

// Impl for primatives
impl LazyDiff for u64 {
    fn struct_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData> {
        let is_eq = self == other;
        [DiffData {
            field: FieldIdentifier::new(),
            self_value: self,
            other_value: other,
        }]
        .into_iter()
        .filter(move |_| !is_eq)
    }
}
