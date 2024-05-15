
# Perforate

Perforate generates variants of a struct to allow a struct field to be "split" from the rest of the struct as in a [split borrow](https://doc.rust-lang.org/nomicon/borrow-splitting.html).

This is similar in effect to using [mem::take](https://doc.rust-lang.org/std/mem/fn.take.html) or one of the other replacement functions, except it can be performed on fields that don't implement [Default](https://doc.rust-lang.org/std/default/trait.Default.html) or when a suitable placeholer can't be constructed.

## Usage

Perforate an owned struct on the stack.

```rust
use perforate::Perforate;

#[derive(Perforate)]
#[repr(C)]
pub struct TestStruct {
    #[perforate]
    one: String,
    two: u64,
}

let test_struct = TestStruct{one: "one".to_string(), two: 42};

let (perforated, one) = test_struct.perforate_one();
assert_eq!(core::mem::size_of::<TestStruct>(), core::mem::size_of_val(&perforated));
assert_eq!(perforated.two, 42);
assert_eq!(one, "one");

let original = perforated.replace_perf(one);
assert_eq!(original.two, 42);
assert_eq!(original.one, "one");
```

Or perforate a struct in an owned box.

```rust
use perforate::Perforate;

#[derive(Perforate)]
#[repr(C)]
pub struct TestStruct {
    #[perforate]
    one: String,
    two: u64,
}

let test_struct = Box::new(TestStruct{one: "one".to_string(), two: 42});

let (perforated_box, one) = TestStruct::boxed_perforate_one(test_struct);
assert_eq!(perforated_box.two, 42);
assert_eq!(one, "one");

let original_box = TestStruct::boxed_replace_one(perforated_box, one);
assert_eq!(original_box.two, 42);
assert_eq!(original_box.one, "one");
```

## Caveats

If a struct has generic parameters or lifetimes that are used only by a field marked with the `#[perforate]` attribute, you must add a [PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) to your struct to prevent compile errors.  In addition, you cannot perforate a field with a generic type parameter on `stable` until [the issue](https://github.com/rust-lang/rust/issues/76560) is merged.

You may access the other "unperforated" fields of the perforated struct, however the perforated version of the struct is a new type and does not have any of the trait implementations from its progenitor.  This includes a custom [Drop](https://doc.rust-lang.org/std/ops/trait.Drop.html) trait.  So if your struct needs special cleanup behavior you must reassemble it before dropping.
