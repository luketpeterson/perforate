
use core::mem::{size_of, size_of_val};

use perforate::Perforate;

#[derive(Perforate, Debug, Clone, PartialEq, Eq)]
// #[derive(Perforate)]
#[repr(C)]
pub struct TestStruct {
    #[perforate]
    one: String,
    #[perforate]
    two: Vec<usize>,
    three: u64,
}


// #[derive(Debug, Clone, PartialEq, Eq)]
// #[perforate(one, two)]
// #[repr(C)]
// pub struct TestStruct {
//     one: String,
//     two: Vec<u8>,
//     three: u64,
// }


#[test]
fn perforate_test() {

    let new_test = TestStruct{one: "one".to_string(), two: vec![42], three: 42};

    let (perforated, one) = new_test.perforate_one();
    assert_eq!(size_of::<TestStruct>(), size_of_val(&perforated));
    assert_eq!(perforated.three, 42);
    assert_eq!(perforated.two, vec![42]);
    assert_eq!(one, "one");

    let original = perforated.replace_perf(one);
    assert_eq!(original.three, 42);
    assert_eq!(original.two, vec![42]);
    assert_eq!(original.one, "one");
}

#[derive(Perforate)]
#[repr(C)]
pub struct DropTest {
    #[perforate]
    bomb: DropBomb,
    payload: u64,
}

#[test]
fn drop_test() {
    let new_test = DropTest{ bomb: DropBomb, payload: 42 };
    let (_perforated, bomb) = new_test.perforate_bomb();
    core::mem::forget(bomb);
}

pub struct DropBomb;

impl Drop for DropBomb {
    fn drop(&mut self) {
        panic!("Don't drop bombs!")
    }
}


#[test]
fn boxed_test() {

    let new_box = Box::new(TestStruct{one: "one".to_string(), two: vec![42], three: 42});

    let (perforated_box, one) = TestStruct::boxed_perforate_one(new_box);
    assert_eq!(perforated_box.three, 42);
    assert_eq!(perforated_box.two, vec![42]);
    assert_eq!(one, "one");

    let original_box = TestStruct::boxed_replace_one(perforated_box, one);
    assert_eq!(original_box.three, 42);
    assert_eq!(original_box.two, vec![42]);
    assert_eq!(original_box.one, "one");
}
