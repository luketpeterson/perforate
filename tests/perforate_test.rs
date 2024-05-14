
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
    four: [u64; 2048],
}


// #[derive(Debug, Clone, PartialEq, Eq)]
// #[perforate(one, two)]
// #[repr(C)]
// pub struct TestStruct {
//     one: String,
//     two: Vec<u8>,
//     three: u64,
// }



//GOAT WTF Test
impl TestStruct {
    pub fn perforate_onezz(self) -> (TestStruct, String) {
        // (unsafe{ core::mem::transmute(self) }, "one".to_string())
        (self, "one".to_string())
    }

    pub fn perforate_oneaa(self) -> Self {
        self
    }

}


#[test]
fn perforate_test() {

    // println!("GOAT {} {}", core::mem::size_of::<TestStruct>(), core::mem::size_of::<TestStructPerfOne>());
    // println!("GOAT {} {}", core::mem::offset_of!(TestStruct, one), core::mem::offset_of!(TestStructPerfOne, __perforation));

    let new_test = TestStruct{one: "one".to_string(), two: vec![42], three: 42, four: [0; 2048]};

//WTF!!  GOAT
// println!("GOAT test {:p}", core::ptr::addr_of!(new_test));
//     let perforated = new_test.perforate_oneaa();
// println!("GOAT perf {:p}", core::ptr::addr_of!(perforated));


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
