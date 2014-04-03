pub mod foo {
  pub struct SomeStruct {
    unused: int,
    to_rename: ~str
  }
}

pub mod bar {
    use super::foo::SomeStruct;
    pub fn yay() {
        let s = SomeStruct {
            unused: 0,
            to_rename: ~"whee"
        };
    }
}

fn main() {
  let s = foo::SomeStruct {
    unused: 0,
    to_rename: ~"whee"
  };
}
