# nyoom-json : json what goes nyoom

nyoom-json is a bare-bones streaming json generation library, built for specialized use cases and Just Going Fast :tm:
it's also no-std!

## credit where credit is due

nyoom-json borrows heavily in style from [write-json](https://github.com/matklad/write-json), and takes its string escaping code from [miniserde](https://github.com/dtolnay/miniserde)

## examples
```rust
use nyoom_json::{Serializer, Null};
let mut out = String::new();
let mut ser = Serializer::create(&mut out);

let mut obj = ser.object();
obj.field("kind", "cat");
obj.field("has_been_fed", false);
obj.field("meow_decibels", 45);
obj.field("illness", Null); // good! mew :3
obj.end();
```

```rust
use nyoom_json::Serializer;
let mut out = String::new();
let mut ser = Serializer::create(&mut out);

let mut arr = ser.array();
arr.add(1);
arr.add(2);
arr.add("three");
arr.end();

ser.end();
```

```rust
use nyoom_json::{Serializer, UnescapedStr};
let mut out = String::with_capacity(64);
let mut ser = Serializer::create(&mut out);
let mut arr = ser.array();
    
arr.add_complex(|mut ser| {
    let mut state = ser.object();
    state.field(UnescapedStr::create("mew"), 3);
    state.complex_field(UnescapedStr::create("meow"), |mut ser| {
        let mut seq = ser.array();
        seq.add(3);
        seq.add(2);
    });
});
arr.add("ny");
arr.end();
```

