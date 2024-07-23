use std::fmt::Debug;

use smallvec::SmallVec;

pub struct DiffData<'a, 'b> {
    pub field: FieldIdentifier,
    pub self_value: &'a dyn Debug,
    pub other_value: &'b dyn Debug,
}

impl<'a, 'b> DiffData<'a, 'b> {
    pub fn push_field(mut self, field: &'static str) -> Self {
        self.field.push(field);
        self
    }
}

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

impl Debug for FieldIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // print as field.field.field
        let mut delimiter = "";
        for field in self.storage.iter().rev() {
            f.write_str(&delimiter)?;
            delimiter = ".";
            f.write_str(*field)?;
        }
        Ok(())
    }
}

pub trait LazyDiff {
    fn lazy_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData>;
}

// Impl for primatives
impl LazyDiff for u64 {
    fn lazy_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData> {
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
