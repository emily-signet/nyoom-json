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

/// Any buffer which JSON may be written into.
pub trait JsonBuffer {
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);
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
}

#[cfg(feature = "alloc")]
pub struct StringBuffer<'a>(pub &'a mut String);

#[cfg(feature = "alloc")]
impl<'a> JsonBuffer for StringBuffer<'a> {
    #[inline(always)]
    fn push(&mut self, c: char) {
        self.0.push(c)
    }

    #[inline(always)]
    fn push_str(&mut self, s: &str) {
        self.0.push_str(s)
    }
}
/// A general JSON serializer, over a mutable buffer of some sort.
/// # Examples
/// ```
/// use nyoom_json::Serializer;
/// 
/// let mut out = String::new();
/// let mut ser = Serializer::create(&mut out);
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
pub struct Serializer<S: JsonBuffer> {
    buf: S,
}

#[cfg(feature = "alloc")]
impl<'a> Serializer<StringBuffer<'a>> {
    /// Creates a serializer over a mutable string reference.
    #[inline(always)]
    pub fn create(buf: &'a mut String) -> Serializer<StringBuffer<'a>> {
        Serializer { buf: StringBuffer(buf) }
    }
}


impl<S: JsonBuffer> Serializer<S> {
    pub fn new(buf: S) -> Serializer<S> {
        Serializer { buf }
    }

    /// Writes out a single primitive JSON value.
    /// # Examples
    /// 
    /// ```
    /// use nyoom_json::Serializer;
    /// 
    /// let mut out = String::new();
    /// let mut ser = Serializer::create(&mut out);
    /// ser.write(3);
    /// ```
    #[inline(always)]
    pub fn write(&mut self, val: impl WriteToJson<S>) {
        val.write_to_json(&mut self.buf)
    }

    /// Starts serialization of an array.
    /// # Examples
    /// 
    /// ```
    /// use nyoom_json::Serializer;
    /// 
    /// let mut out = String::new();
    /// let mut ser = Serializer::create(&mut out);
    /// 
    /// let mut arr = ser.array();
    /// arr.add("friends");
    /// arr.add("romans");
    /// arr.add("countrymen");
    /// arr.end();
    /// ```
    #[inline(always)]
    pub fn array<'a>(&'a mut self) -> ArrayWriter<&'a mut S> {
        ArrayWriter::start(&mut self.buf)
    }

    /// Starts serialization of an object.
    /// # Examples
    /// 
    /// ```
    /// use nyoom_json::Serializer;
    /// 
    /// let mut out = String::new();
    /// let mut ser = Serializer::create(&mut out);
    /// 
    /// let mut obj = ser.object();
    /// obj.field("kind", "cat");
    /// obj.field("has_been_fed", false);
    /// obj.field("meow_decibels", 45);
    /// obj.end();
    /// ```
    #[inline(always)]
    pub fn object<'a>(&'a mut self) -> ObjectWriter<&'a mut S> {
        ObjectWriter::start(&mut self.buf)
    }

    /// Ends the serializer.
    pub fn end(self) -> S {
        self.buf
    }
}

/// A serializer that is only able to serialize a single value. See documentation of [Serializer](Serializer) 
pub struct SingleValueSerializer<S: JsonBuffer> {
    guard: ManuallyDrop<S>,
}

impl<S: JsonBuffer> SingleValueSerializer<S> {
    #[inline(always)]
    pub fn new(val: S) -> SingleValueSerializer<S> {
        SingleValueSerializer {
            guard: ManuallyDrop::new(val),
        }
    }

    #[inline(always)]
    pub fn write(mut self, val: impl WriteToJson<S>) {
        let mut buf = unsafe { ManuallyDrop::<S>::take(&mut self.guard) };
        val.write_to_json(&mut buf);
        core::mem::forget(self);
    }

    #[inline(always)]
    pub fn array(mut self) -> ArrayWriter<S> {
        let buf = unsafe { ManuallyDrop::<S>::take(&mut self.guard) };
        let w = ArrayWriter::start(buf);
        core::mem::forget(self);
        w
    }

    #[inline(always)]
    pub fn object(mut self) -> ObjectWriter<S> {
        let buf = unsafe { ManuallyDrop::<S>::take(&mut self.guard) };
        let w = ObjectWriter::start(buf);
        core::mem::forget(self);
        w
    }
}

impl<S: JsonBuffer> Drop for SingleValueSerializer<S> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { ManuallyDrop::<S>::take(&mut self.guard).push_str("null") };
    }
}

/// Serializer for a JSON array.
pub struct ArrayWriter<S: JsonBuffer> {
    buf: S,
    first_element: bool,
}

impl<S: JsonBuffer> ArrayWriter<S> {
    #[inline(always)]
    fn start(mut buf: S) -> ArrayWriter<S> {
        buf.push('[');
        ArrayWriter {
            buf,
            first_element: true,
        }
    }

    #[inline(always)]
    fn comma(&mut self) {
        match self.first_element {
            true => self.first_element = false,
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
    /// # let mut ser = Serializer::create(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.add("friends");
    /// arr.add("romans");
    /// arr.add("countrymen");
    /// arr.end();
    /// ```
    #[inline(always)]
    pub fn add(&mut self, val: impl WriteToJson<S>) {
        self.comma();
        val.write_to_json(&mut self.buf)
    }

    /// Adds a slice of a JSON primitive to this array.
    /// # Examples
    /// 
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::create(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.extend(&["friends", "romans", "countrymen"]);
    /// arr.end();
    /// ```
    #[inline(always)]
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
    /// # let mut ser = Serializer::create(&mut out);
    /// #
    /// let mut arr = ser.array();
    /// arr.add_complex(|mut ser| {
    ///     let mut obj = ser.object();
    ///     obj.field("kitten", true);
    ///     obj.field("cuteness", 10.0);
    /// });
    /// arr.end();
    /// ```
    #[inline(always)]
    pub fn add_complex<F>(&mut self, encoder: F)
    where
        F: FnOnce(SingleValueSerializer<&mut S>),
    {
        self.comma();
        encoder(SingleValueSerializer::new(&mut self.buf));
    }

    /// Finishes out the array. Equivalent to drop(arr);
    #[inline(always)]
    pub fn end(self) {}
}

impl<S: JsonBuffer> Drop for ArrayWriter<S> {
    #[inline(always)]
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
    #[inline(always)]
    fn write<S: JsonBuffer>(self, out: &mut S) {
        self.write_to_json(out)
    }
}

#[sealed]
impl<T: AsRef<str>> Key for T {
    #[inline(always)]
    fn write<S: JsonBuffer>(self, out: &mut S) {
        self.as_ref().write_to_json(out)
    }
}

/// A serializer for a JSON object.
pub struct ObjectWriter<S: JsonBuffer> {
    buf: S,
    first_element: bool,
}

impl<S: JsonBuffer> ObjectWriter<S> {
    #[inline(always)]
    fn start(mut buf: S) -> ObjectWriter<S> {
        buf.push('{');
        ObjectWriter {
            buf,
            first_element: true,
        }
    }

    #[inline(always)]
    fn comma(&mut self) {
        match self.first_element {
            true => self.first_element = false,
            false => self.buf.push(','),
        }
    }

    #[inline(always)]
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
    /// # let mut ser = Serializer::create(&mut out);
    /// #
    /// let mut obj = ser.object();
    /// obj.field("kind", "cat");
    /// obj.field("has_been_fed", false);
    /// obj.field("meow_decibels", 45);
    /// obj.end();
    /// ```
    #[inline(always)]
    pub fn field<K: Key>(&mut self, key: K, val: impl WriteToJson<S>) {
        self.key(key);
        val.write_to_json(&mut self.buf);
    }

    /// Adds an arbitrary JSON object to this object.
    /// 
    /// # Arguments
    /// 
    /// * `key` - the key for the field
    /// * `encoder` - A closure that encodes a single value into the field.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use nyoom_json::Serializer;
    /// #
    /// # let mut out = String::new();
    /// # let mut ser = Serializer::create(&mut out);
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
    #[inline(always)]
    pub fn complex_field<K, F>(&mut self, key: K, encode: F)
    where
        K: Key,
        F: FnOnce(SingleValueSerializer<&mut S>),
    {
        self.key(key);
        encode(SingleValueSerializer::new(&mut self.buf))
    }

    #[inline(always)]
    pub fn end(self) {}
}

impl<S: JsonBuffer> Drop for ObjectWriter<S> {
    #[inline(always)]
    fn drop(&mut self) {
        self.buf.push('}');
    }
}
