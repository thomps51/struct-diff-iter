pub use itertools;
use itertools::Itertools;
pub use struct_diff_iter_derive::LazyDiff;
pub use struct_diff_iter_internal::DiffData;
pub use struct_diff_iter_internal::FieldIdentifier;
pub use struct_diff_iter_internal::LazyDiff;

use std::fmt::Debug;

#[derive(PartialEq, Debug)]
enum MyEnum {
    One,
    Two,
    Three { field: u64 },
    Four(u64, u64),
}

impl LazyDiff for MyEnum {
    fn struct_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData> {
        // Since each leg of this match will be a different type, we need to Box up the iterator here.
        // An alternative would be to if let every combination, but that seems silly to give up that much
        // performance for a single allocation.
        let iter: Box<dyn Iterator<Item = DiffData>> = match (self, other) {
            (MyEnum::One, MyEnum::One) => Box::new([].into_iter()),
            (MyEnum::Two, MyEnum::Two) => Box::new([].into_iter()),
            (MyEnum::Three { field: self_field }, MyEnum::Three { field: other_field }) => {
                use itertools::Itertools;
                Box::new(
                    self_field
                        .struct_diff_iter(other_field)
                        .update(|x| x.field.push("Three.field")),
                )
            }
            (MyEnum::Four(self_field0, self_field1), MyEnum::Four(other_field0, other_field1)) => {
                Box::new(
                    self_field0
                        .struct_diff_iter(other_field0)
                        .update(|x| x.field.push("Three.0"))
                        .chain(
                            self_field1
                                .struct_diff_iter(other_field1)
                                .update(|x| x.field.push("Three.1")),
                        ),
                )
            }
            (_, _) => Box::new(
                [DiffData {
                    field: FieldIdentifier::new(),
                    self_value: self,
                    other_value: other,
                }]
                .into_iter(),
            ),
        };
        iter
    }
}

struct MyInnerType {
    field1: u64,
}

struct MyOuterType {
    field1: u64,
    field2: MyEnum,
    field3: String,
    field4: MyInnerType,
}

// Ugly impl that will be generated

impl LazyDiff for MyOuterType {
    fn struct_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = DiffData> {
        self.field1.struct_diff_iter(&other.field1).map(|mut x| {
            x.field.push("field1");
            x
        })
        // .chain(self.field2.lazy_diff_iter(&other.field2).inspect(|mut x| {
        //     x.field.push("field2");
        //     x
        // }))
        // .chain(self.field3.lazy_diff_iter(&other.field3))
        // .chain(self.field4.lazy_diff_iter(&other.field4))
    }
}
