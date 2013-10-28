pub mod foo {
  pub struct SomeStruct {
    unused: int,
    to_rename: ~str
  }
}

fn main() {
  let s = foo::SomeStruct {
    unused: 0,
    to_rename: ~"whee"
  };
}
