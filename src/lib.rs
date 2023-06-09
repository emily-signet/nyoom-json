#![doc = include_str!("../README.md")]
#![no_std]

use core::mem::ManuallyDrop;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

use sealed::sealed;

mod escape;
mod write_to_json;
pub use write_to_json::*;

#[inline]
#[cold]
fn cold() {}

/// Any buffer which JSON may be written into.
pub trait JsonBuffer {
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);
    fn reserve(&mut self, l: usize);
}

impl<S> JsonBuffer for &mut S
where
    S: JsonBuffer,
{
    #[inline(always)]
    fn push(&mut self, c: char) {
        (*self).push(c)
    }

    #[inline(always)]
    fn push_str(&mut self, s: &str) {
        (*self).push_str(s)
    }

    #[inline(always)]
    fn reserve(&mut self, l: usize) {
        (*self).reserve(l)
    }
}

#[cfg(feature = "alloc")]
impl JsonBuffer for String {
    #[inline(always)]
    fn push(&mut self, c: char) {
        self.push(c)
    }

    #[inline(always)]
    fn push_str(&mut self, s: &str) {
        self.push_str(s)
    }

    #[inline(always)]
    fn reserve(&mut self, l: usize) {
        self.reserve(l)
    }
}

/// A general JSON serializer, over a mutable buffer of some sort.
/// # Examples
/// ```
/// use nyoom_json::Serializer;
///
/// let mut out = String::new();
/// let mut ser = Serializer::new(&mut out);
///
/// let mut obj = ser.object();
/// obj.field("kind", "cat");
/// obj.field("has_been_fed", false);
/// obj.field("meow_decibels", 45);
/// obj.end();
///
/// let mut arr = ser.array();
/// arr.add("friends");
/// arr.add("romans");
/// arr.add("countrymen");
/// arr.end();
///
/// ser.end();
/// ```
#[repr(transparent)]
pub struct Serializer<'a, S: JsonBuffer> {
    buf: &'a mut S,
}

impl<'a, S: JsonBuffer> Serializer<'a, S> {
    /// Creates a new serializer over a JSON output buffer.
    pub fn new(buf: &mut S) -> Serializer<S> {
        Serializer { buf }
    }

    /// Writes out a single primitive JSON value.
    /// # Examples
    ///
    /// ```
    /// use nyoom_json::Serializer;
    ///
    /// let mut out = String::new();
    /// let mut ser = Serializer::new(&mut out);
    /// ser.write(3);
    /// ```
    pub fn write(&mut self, val: impl WriteToJson<S>) {
        val.write_to_json(self.buf)
    }

    /// Starts serialization of an array.
    /// # Examples
    ///
    /// ```
    /// use nyoom_json::Serializer;
    ///
    /// let mut out = String::new();
    /// let mut ser = Serializer::new(&mut out);
    ///
    /// let mut arr = ser.array();
    /// arr.add("friends");
    /// arr.add("romans");
    /// arr.add("countrymen");
    /// arr.end();
    /// ```
    pub fn array(&mut self) -> ArrayWriter<S> {
        ArrayWriter::start(self.buf)
    }

    /// Starts serialization of an object.
    /// # Examples
    ///
    /// ```
    /// use nyoom_json::Serializer;
    ///
    /// let mut out = String::new();
    /// let mut ser = Serializer::new(&mut out);
    ///
    /// let mut obj = ser.object();
    /// obj.field("kind", "cat");
    /// obj.field("has_been_fed", false);
    /// obj.field("meow_decibels", 45);
    /// obj.end();
    /// ```
    pub fn object(&mut self) -> ObjectWriter<S> {
        ObjectWriter::start(self.buf)
    }

    /// Ends the serializer.
    pub fn end(self) {}
}

/// A serializer that is only able to serialize a single value. See documentation of [Serializer](Serializer)
pub struct SingleValueSerializer<'a, S: JsonBuffer> {
    guard: ManuallyDrop<&'a mut S>,
}

impl<'a, S: JsonBuffer> SingleValueSerializer<'a, S> {
    pub fn new(val: &'a mut S) -> SingleValueSerializer<'a, S> {
        SingleValueSerializer {
            guard: ManuallyDrop::new(val),
        }
    }

    pub fn write(mut self, val: impl WriteToJson<S>) {
        let buf = unsafe { ManuallyDrop::<&'a mut S>::take(&mut self.guard) };
        val.write_to_json(buf);
        core::mem::forget(self);
    }

    pub fn array(mut self) -> ArrayWriter<'a, S> {
        let buf = unsafe { ManuallyDrop::<&'a mut S>::take(&mut self.guard) };
        let w = ArrayWriter::start(buf);
        core::mem::forget(self);
        w
    }

    pub fn object(mut self) -> ObjectWriter<'a, S> {
        let buf = unsafe { ManuallyDrop::<&'a mut S>::take(&mut self.guard) };
        let w = ObjectWriter::start(buf);
        core::mem::forget(self);
        w
    }
}

impl<'a, S: JsonBuffer> Drop for SingleValueSerializer<'a, S> {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::<&'a mut S>::take(&mut self.guard).push_str("null") };
    }
}

/// Serializer for a JSON array.
pub struct ArrayWriter<'a, S: JsonBuffer> {
    buf: &'a mut S,
    first_element: bool,
}

impl<'a, S: JsonBuffer> ArrayWriter<'a, S> {
    fn start(buf: &'a mut S) -> ArrayWriter<'a, S> {
        buf.push('[');
        ArrayWriter {
            buf,
            first_element: true,
        }
    }

    fn comma(&mut self) {
        match self.first_element {
            true => {
                cold();
                self.first_element = false
            }
            false => self.buf.push(','),
        }
    }

    /// Adds a single primitive JSON value to this array.
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.add("friends");
    /// arr.add("romans");
    /// arr.add("countrymen");
    /// arr.end();
    /// ```
    pub fn add(&mut self, val: impl WriteToJson<S>) {
        self.comma();
        val.write_to_json(self.buf)
    }

    /// Adds a slice of a JSON primitive to this array.
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.extend(&["friends", "romans", "countrymen"]);
    /// arr.end();
    /// ```
    pub fn extend<V: WriteToJson<S>>(&mut self, vals: impl IntoIterator<Item = V>) {
        for val in vals {
            self.add(val);
        }
    }

    /// Adds an arbitrary JSON object to this array.
    ///
    /// # Arguments
    ///
    /// * `encoder` - A closure that encodes a single value into the array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.add_complex(|mut ser| {
    ///     let mut obj = ser.object();
    ///     obj.field("kitten", true);
    ///     obj.field("cuteness", 10.0);
    /// });
    /// arr.end();
    /// ```
    pub fn add_complex<F, O>(&mut self, encoder: F) -> O
    where
        F: FnOnce(SingleValueSerializer<&mut S>) -> O,
    {
        self.comma();
        encoder(SingleValueSerializer::new(&mut self.buf))
    }

    /// Adds a JSON object to this array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    ///
    /// let mut obj = arr.add_object();
    /// obj.field("kitten", true);
    /// obj.field("cuteness", 10.0);
    /// obj.end();
    ///
    /// arr.end();
    /// ```
    pub fn add_object(&mut self) -> ObjectWriter<S> {
        self.comma();
        ObjectWriter::start(self.buf)
    }

    /// Adds a JSON array.. to this array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    ///
    /// let mut inner_arr = arr.add_array();
    /// inner_arr.extend(&[1,2,3]);
    /// inner_arr.end();
    ///
    /// arr.end();
    /// ```
    pub fn add_array(&mut self) -> ArrayWriter<S> {
        self.comma();
        ArrayWriter::start(self.buf)
    }

    /// Finishes out the array. Equivalent to drop(arr);
    pub fn end(self) {}
}

impl<S: JsonBuffer> Drop for ArrayWriter<'_, S> {
    fn drop(&mut self) {
        self.buf.push(']');
    }
}

/// A key for a JSON object's field.
#[sealed]
pub trait Key {
    fn write<S: JsonBuffer>(self, out: &mut S);
}

#[sealed]
impl Key for UnescapedStr<'_> {
    fn write<S: JsonBuffer>(self, out: &mut S) {
        self.write_to_json(out)
    }
}

#[sealed]
impl<T: AsRef<str>> Key for T {
    fn write<S: JsonBuffer>(self, out: &mut S) {
        self.as_ref().write_to_json(out)
    }
}

/// A serializer for a JSON object.
pub struct ObjectWriter<'a, S: JsonBuffer> {
    buf: &'a mut S,
    first_element: bool,
}

impl<'a, S: JsonBuffer> ObjectWriter<'a, S> {
    fn start(buf: &'a mut S) -> ObjectWriter<S> {
        buf.push('{');
        ObjectWriter {
            buf,
            first_element: true,
        }
    }

    fn comma(&mut self) {
        match self.first_element {
            true => {
                cold();
                self.first_element = false
            }
            false => self.buf.push(','),
        }
    }

    fn key<K: Key>(&mut self, key: K) {
        self.comma();
        key.write(&mut self.buf);
        self.buf.push(':');
    }

    /// Adds a field to this object.
    ///
    /// # Examples
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut obj = ser.object();
    /// obj.field("kind", "cat");
    /// obj.field("has_been_fed", false);
    /// obj.field("meow_decibels", 45);
    /// obj.end();
    /// ```
    pub fn field<K: Key>(&mut self, key: K, val: impl WriteToJson<S>) {
        self.key(key);
        val.write_to_json(self.buf);
    }

    /// Adds an arbitrary JSON object to this object.
    ///
    /// # Arguments
    ///
    /// * `key` - the key for the field
    /// * `encoder` - A closure that encodes a single value into the field. It may return an arbitrary value that will be passed back to the caller.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut obj = ser.object();
    /// obj.complex_field("numbers", |mut ser| {
    ///     let mut arr = ser.array();
    ///     arr.add(1);
    ///     arr.add(2);
    ///     arr.add("three");
    /// });
    /// obj.end()
    /// ```
    pub fn complex_field<K, F, O>(&mut self, key: K, encode: F) -> O
    where
        K: Key,
        F: FnOnce(SingleValueSerializer<&mut S>) -> O,
    {
        self.key(key);
        encode(SingleValueSerializer::new(&mut self.buf))
    }

    /// Adds a JSON object field to this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut obj = ser.object();
    /// obj.field("kitten", true);
    /// obj.field("cuteness", 10.0);
    ///
    /// let mut bed = obj.object_field("bed");
    /// bed.field("cozy", true);
    /// bed.field("wear_and_tear", 2.0);
    /// bed.end();
    ///
    /// obj.end();
    /// ```
    pub fn object_field<K: Key>(&mut self, key: K) -> ObjectWriter<S> {
        self.key(key);
        ObjectWriter::start(self.buf)
    }

    /// Adds a JSON array field to this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::new(&mut out);
    /// #
    /// let mut arr = ser.array();
    ///
    /// let mut obj = arr.add_object();
    /// obj.field("kitten", true);
    /// obj.field("cuteness", 10.0);
    ///
    /// let mut toys = obj.array_field("toys");
    /// toys.extend(&["mouse", "ball", "string", "box", "scratcher"]);
    /// toys.end();
    ///
    /// obj.end();
    ///
    /// arr.end();
    /// ```
    pub fn array_field<K: Key>(&mut self, key: K) -> ArrayWriter<S> {
        self.key(key);
        ArrayWriter::start(self.buf)
    }

    pub fn end(self) {}
}

impl<S: JsonBuffer> Drop for ObjectWriter<'_, S> {
    fn drop(&mut self) {
        self.buf.push('}');
    }
}
