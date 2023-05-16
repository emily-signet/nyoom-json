use crate::{escape::escape_str, JsonBuffer};



/// A value that is able to be written directly into JSON.
pub trait WriteToJson<S: JsonBuffer> {
    fn write_to_json(self, out: &mut S);
}

macro_rules! impl_int {
    ($($ty:ty),*) => {
        $(
            impl<S: JsonBuffer> WriteToJson<S> for $ty {
                #[inline(always)]
                fn write_to_json(self, out: &mut S) {
                    let mut int_buf = itoa::Buffer::new();
                    out.push_str(int_buf.format(self));
                }
            }
        )*

    }
}

macro_rules! impl_float {
    ($($ty:ty),*) => {
        $(
            impl<S: JsonBuffer> WriteToJson<S> for $ty {
                #[inline(always)]
                fn write_to_json(self, out: &mut S) {
                    let mut float_buf = ryu::Buffer::new();
                    out.push_str(float_buf.format(self));
                }
            }
        )*

    }
}

impl_int!(u8, u16, u32, u64, i8, i16, i32, i64);
impl_float!(f32, f64);

impl<S: JsonBuffer> WriteToJson<S> for &str {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        out.push('"');
        escape_str(self, out);
        out.push('"');
    }
}

impl<S: JsonBuffer> WriteToJson<S> for bool {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        match self {
            true => out.push_str("true"),
            false => out.push_str("false"),
        }
    }
}



/// The JSON null value!
pub struct Null;

impl<S: JsonBuffer> WriteToJson<S> for () {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        out.push_str("null")
    }
}

impl<S: JsonBuffer> WriteToJson<S> for Null {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        out.push_str("null")
    }
}

impl<S: JsonBuffer, T> WriteToJson<S> for &T where T: Copy + WriteToJson<S> {
    fn write_to_json(self, out: &mut S) {
        (*self).write_to_json(out)
    }
}

/// A string that will *not* have escapes applied to it. You should only use this if you're *absolutely* sure you don't need them.
#[repr(transparent)]
pub struct UnescapedStr<'a>(&'a str);

impl<'a> UnescapedStr<'a> {
    #[inline(always)]
    pub fn create(val: &'a str) -> UnescapedStr<'a> {
        debug_assert!(
            val.as_bytes()
                .iter()
                .all(|val| { crate::escape::ESCAPE[*val as usize] == 0 }),
            "string contains characters that need to be escaped!"
        );

        UnescapedStr(val)
    }
}

impl<'a, S: JsonBuffer> WriteToJson<S> for UnescapedStr<'a> {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        out.push('"');
        out.push_str(self.0);
        out.push('"');
    }
}