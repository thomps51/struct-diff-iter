use lazy_struct_diff::LazyDiff;

#[derive(PartialEq, LazyDiff, Clone)]
struct SimpleStruct {
    field1: u64,
    field2: u64,
    field3: u64,
}

#[derive(PartialEq, LazyDiff, Clone)]
struct CombinedStruct {
    field1: SimpleStruct,
    field2: StructUnnamedFields,
    field3: MyEnum,
    field4: DummyType3,
}

#[derive(PartialEq, LazyDiff, Clone)]
struct StructUnnamedFields(u64, u64);

#[derive(PartialEq, LazyDiff, Clone)]
struct DummyType3;

#[derive(Debug, PartialEq, LazyDiff, Clone)]
enum MyEnum {
    One,
    Two,
    Three { field: u64 },
    Four(u64, u64),
}

#[test]
fn test_basic_struct() {
    let dt = SimpleStruct {
        field1: 1,
        field2: 2,
        field3: 3,
    };
    let mut other = dt.clone();
    other.field2 = 3;
    let result: Vec<_> = dt.lazy_diff_iter(&other).collect();
    assert_eq!(result.len(), 1);
    let diff = &result[0];
    assert_eq!(format!("{:?}", diff.field), "field2");
}

#[test]
fn test_inner_struct() {
    let dt = CombinedStruct {
        field1: SimpleStruct {
            field1: 1,
            field2: 2,
            field3: 3,
        },
        field2: StructUnnamedFields(1, 2),
        field3: MyEnum::Three { field: 1 },
        field4: DummyType3,
    };
    let mut other = dt.clone();
    other.field1.field2 = 3;
    other.field2.1 = 3;
    other.field3 = MyEnum::Three { field: 2 };
    let diff: Vec<_> = dt.lazy_diff_iter(&other).collect();
    assert_eq!(format!("{:?}", diff[0].field), "field1.field2");
    assert_eq!(format!("{:?}", diff[1].field), "field2.1");
    assert_eq!(format!("{:?}", diff[2].field), "field3.Three.field");
}
