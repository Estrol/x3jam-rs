pub use encoder_derive::{StructDeserializer, StructSerializer};

use byteorder_lite::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn get_buffer<T: StructEncodeImpl>(value: &T) -> Vec<u8> {
    let mut buffer = Vec::new();
    value.impl_encode(&mut buffer).unwrap();
    buffer
}

/// A trait for encoding structs into binary writers.
///
/// Auto implemeted to any type that implements `std::io::Write`
pub trait StructWriteExt {
    fn write_struct<T: StructEncodeImpl>(&mut self, value: &T) -> std::io::Result<()>;
}

/// A trait for decoding structs from binary readers.
///
/// Auto implemeted to any type that implements `std::io::Read`
pub trait StructReadExt {
    fn read_struct<T: StructDecodeImpl>(&mut self, out: &mut T) -> std::io::Result<()>;
    fn read_uninit<T: StructDecodeImpl>(
        &mut self,
        out: &mut std::mem::MaybeUninit<T>,
    ) -> std::io::Result<()>;
}

impl<W: std::io::Write> StructWriteExt for W {
    #[inline(always)]
    fn write_struct<T: StructEncodeImpl>(&mut self, value: &T) -> std::io::Result<()> {
        value.impl_encode(self)
    }
}

impl<R: std::io::Read> StructReadExt for R {
    #[inline(always)]
    fn read_struct<T: StructDecodeImpl>(&mut self, out: &mut T) -> std::io::Result<()> {
        *out = T::impl_decode(self)?;
        Ok(())
    }
    #[inline(always)]
    fn read_uninit<T: StructDecodeImpl>(
        &mut self,
        out: &mut std::mem::MaybeUninit<T>,
    ) -> std::io::Result<()> {
        unsafe {
            out.as_mut_ptr().write(T::impl_decode(self)?);
        }
        Ok(())
    }
}

pub trait StructEncodeImpl {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl StructEncodeImpl for char {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut buf = [0; 4];
        let encoded = self.encode_utf8(&mut buf);
        writer.write_all(encoded.as_bytes())
    }
}

impl StructEncodeImpl for u8 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(&[*self])
    }
}

impl StructEncodeImpl for i8 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(&[*self as u8])
    }
}

impl StructEncodeImpl for u16 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for i16 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_i16::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for u32 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for i32 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_i32::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for u64 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_u64::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for i64 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_i64::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for bool {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        (*self as u8).impl_encode(writer)
    }
}

impl StructEncodeImpl for f32 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_f32::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for f64 {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_f64::<LittleEndian>(*self)
    }
}

impl StructEncodeImpl for String {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.as_bytes();
        writer.write_u32::<LittleEndian>(bytes.len() as u32)?;
        writer.write_all(bytes)
    }
}

impl StructEncodeImpl for &str {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.as_bytes();
        writer.write_u32::<LittleEndian>(bytes.len() as u32)?;
        writer.write_all(bytes)
    }
}

impl StructEncodeImpl for std::ffi::CString {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.as_bytes_with_nul();
        writer.write_all(bytes)
    }
}

impl StructEncodeImpl for &std::ffi::CStr {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.to_bytes_with_nul();
        writer.write_all(bytes)
    }
}

impl<T: StructEncodeImpl> StructEncodeImpl for Vec<T> {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.len() as u32)?;
        for item in self {
            item.impl_encode(writer)?;
        }
        Ok(())
    }
}

impl<T: StructEncodeImpl> StructEncodeImpl for &[T] {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.len() as u32)?;
        for item in *self {
            item.impl_encode(writer)?;
        }
        Ok(())
    }
}

impl<T: StructEncodeImpl, const N: usize> StructEncodeImpl for [T; N] {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        for item in self {
            item.impl_encode(writer)?;
        }
        Ok(())
    }
}

impl<T: StructEncodeImpl> StructEncodeImpl for Option<T> {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Some(value) => {
                1u8.impl_encode(writer)?; // Indicate presence
                value.impl_encode(writer)
            }
            None => 0u8.impl_encode(writer), // Indicate absence
        }
    }
}

impl<T: StructEncodeImpl> StructEncodeImpl for Box<T> {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_ref().impl_encode(writer)
    }
}

impl StructEncodeImpl for () {
    #[inline(always)]
    fn impl_encode(&self, _writer: &mut impl std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

pub trait StructDecodeImpl {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self>
    where
        Self: Sized;
}

impl StructDecodeImpl for char {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        let s = std::str::from_utf8(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        s.chars().next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "No character found")
        })
    }
}

impl StructDecodeImpl for u8 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl StructDecodeImpl for i8 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }
}

impl StructDecodeImpl for u16 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_u16::<LittleEndian>()
    }
}

impl StructDecodeImpl for i16 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_i16::<LittleEndian>()
    }
}

impl StructDecodeImpl for u32 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_u32::<LittleEndian>()
    }
}

impl StructDecodeImpl for i32 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_i32::<LittleEndian>()
    }
}

impl StructDecodeImpl for u64 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_u64::<LittleEndian>()
    }
}

impl StructDecodeImpl for i64 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_i64::<LittleEndian>()
    }
}

impl StructDecodeImpl for bool {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let byte = u8::impl_decode(reader)?;
        Ok(byte != 0)
    }
}

impl StructDecodeImpl for f32 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_f32::<LittleEndian>()
    }
}

impl StructDecodeImpl for f64 {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        reader.read_f64::<LittleEndian>()
    }
}

impl StructDecodeImpl for String {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut buf = vec![0; len];
        reader.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl StructDecodeImpl for std::ffi::CString {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut bytes = Vec::new();
        loop {
            let byte = u8::impl_decode(reader)?;
            bytes.push(byte);
            if byte == 0 {
                break;
            }
        }
        Ok(std::ffi::CString::from_vec_with_nul(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
    }
}

impl<T: StructDecodeImpl> StructDecodeImpl for Vec<T> {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::impl_decode(reader)?);
        }
        Ok(vec)
    }
}

impl<T: StructDecodeImpl> StructDecodeImpl for Option<T> {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let flag = u8::impl_decode(reader)?;
        if flag == 0 {
            Ok(None)
        } else {
            Ok(Some(T::impl_decode(reader)?))
        }
    }
}

impl<T: StructDecodeImpl> StructDecodeImpl for Box<T> {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        Ok(Box::new(T::impl_decode(reader)?))
    }
}

impl<T: StructDecodeImpl, const N: usize> StructDecodeImpl for [T; N] {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut arr: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        for i in 0..N {
            arr[i] = std::mem::MaybeUninit::new(T::impl_decode(reader)?);
        }
        Ok(unsafe { std::ptr::read(&arr as *const _ as *const [T; N]) })
    }
}

impl StructDecodeImpl for () {
    fn impl_decode(_reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        Ok(())
    }
}

#[doc(hidden)]
pub use byteorder_lite;

#[cfg(test)]
pub mod test {
    #[allow(unused_imports)]
    use super::*;
    use std::io::Cursor;

    #[repr(u16)]
    #[derive(Debug, Copy, Clone, StructSerializer, StructDeserializer)]
    enum TestEnum {
        VariantA = 1,
        VariantB = 2,
        VariantC = 3,
    }

    #[test]
    fn test_enum_encoding() {
        let value = TestEnum::VariantB;

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&value).unwrap();

        let encoded = buffer.into_inner();
        println!("{:x?}", encoded);

        let mut read_buffer = Cursor::new(encoded);
        let mut decoded = TestEnum::VariantA; // Initialize with any variant
        read_buffer.read_struct(&mut decoded).unwrap();

        assert_eq!(value as u16, decoded as u16);
    }

    #[repr(u8)]
    #[derive(Debug, Copy, Clone, StructSerializer, StructDeserializer)]
    enum SmallEnum {
        A = 0,
    }

    #[repr(u16)]
    #[derive(Debug, Copy, Clone, StructSerializer, StructDeserializer)]
    enum MediumEnum {
        A = 0,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, StructSerializer, StructDeserializer)]
    enum LargeEnum {
        A = 0,
    }

    #[repr(u64)]
    #[derive(Debug, Copy, Clone, StructSerializer, StructDeserializer)]
    enum HugeEnum {
        A = 0,
    }

    #[test]
    fn test_enum_size() {
        assert_eq!(std::mem::size_of::<SmallEnum>(), 1);
        assert_eq!(std::mem::size_of::<MediumEnum>(), 2);
        assert_eq!(std::mem::size_of::<LargeEnum>(), 4);
        assert_eq!(std::mem::size_of::<HugeEnum>(), 8);

        let small = SmallEnum::A;
        let medium = MediumEnum::A;
        let large = LargeEnum::A;
        let huge = HugeEnum::A;

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&small).unwrap();

        assert_eq!(buffer.into_inner(), vec![0]);

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&medium).unwrap();

        assert_eq!(buffer.into_inner(), vec![0, 0]);

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&large).unwrap();

        assert_eq!(buffer.into_inner(), vec![0, 0, 0, 0]);

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&huge).unwrap();

        assert_eq!(buffer.into_inner(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[derive(StructSerializer, StructDeserializer)]
    struct TestStruct {
        #[field(0)]
        a: u32,
        #[field(1)]
        b: String,
        #[field(2)]
        c: Vec<u8>,
    }

    #[test]
    fn test_encoding() {
        let test = TestStruct {
            a: 42,
            b: "Hello".to_string(),
            c: vec![1, 2, 3],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_struct(&test).unwrap();

        let encoded = buffer.into_inner();
        println!("{:x?}", encoded);

        let mut read_buffer = Cursor::new(encoded);
        let mut decoded = TestStruct {
            a: 0,
            b: String::new(),
            c: Vec::new(),
        };
        read_buffer.read_struct(&mut decoded).unwrap();

        assert_eq!(test.a, decoded.a);
        assert_eq!(test.b, decoded.b);
        assert_eq!(test.c, decoded.c);
    }

    #[derive(StructSerializer)]
    struct NestedStruct {
        #[field(0)]
        x: u32,
        #[field(1)]
        y: TestStruct,
    }

    #[test]
    fn test_nested_encoding() {
        let nested = NestedStruct {
            x: 99,
            y: TestStruct {
                a: 42,
                b: "Hello".to_string(),
                c: vec![1, 2, 3],
            },
        };

        let mut buffer = Cursor::new(Vec::new());
        nested.impl_encode(&mut buffer).unwrap();

        let encoded = buffer.into_inner();
        println!("{:x?}", encoded);
    }

    #[allow(non_camel_case_types)]
    type bool32 = u32;

    #[derive(StructSerializer)]
    struct AllTypesStruct {
        #[field(0)]
        a: char,
        #[field(1)]
        b: u8,
        #[field(2)]
        c: i8,
        #[field(3)]
        d: u16,
        #[field(4)]
        e: i16,
        #[field(5)]
        f: u32,
        #[field(6)]
        g: i32,
        #[field(7)]
        h: u64,
        #[field(8)]
        i: i64,
        #[field(9)]
        j: bool,
        #[field(10)]
        k: bool32,
        #[field(11)]
        l: f32,
        #[field(12)]
        m: f64,
        #[field(13)]
        n: String,
        #[field(14)]
        o: Vec<u8>,
        #[field(15)]
        p: Vec<TestStruct>,
        #[field(16)]
        q: NestedStruct,
        r: Option<u32>,
        s: Option<u32>,
        r#enum: TestEnum,
    }

    #[test]
    fn test_all_types_encoding() {
        let all = AllTypesStruct {
            a: 'A',
            b: 255,
            c: -128,
            d: 65535,
            e: -32768,
            f: 4294967295,
            g: -2147483648,
            h: 18446744073709551615,
            i: -9223372036854775808,
            j: true,
            k: 1, // true as bool32
            l: 3.14,
            m: 2.71828,
            n: "Test".to_string(),
            o: vec![1, 2, 3, 4],
            p: vec![
                TestStruct {
                    a: 42,
                    b: "Hello".to_string(),
                    c: vec![1, 2, 3],
                },
                TestStruct {
                    a: 100,
                    b: "World".to_string(),
                    c: vec![4, 5, 6],
                },
            ],
            q: NestedStruct {
                x: 99,
                y: TestStruct {
                    a: 42,
                    b: "Hello".to_string(),
                    c: vec![1, 2, 3],
                },
            },
            r: Some(12345),
            s: None,
            r#enum: TestEnum::VariantC,
        };

        let mut buffer = Cursor::new(Vec::new());
        all.impl_encode(&mut buffer).unwrap();

        let encoded = buffer.into_inner();
        println!("{:x?}", encoded);
    }

    #[derive(StructSerializer)]
    struct ZeroSizedStruct {}

    #[test]
    fn test_zero_sized_struct_encoding() {
        let zero = ZeroSizedStruct {};
        let mut buffer = Cursor::new(Vec::new());
        zero.impl_encode(&mut buffer).unwrap();

        let encoded = buffer.into_inner();
        println!("{:x?}", encoded); // Should be empty
    }

    #[test]
    fn test_enum() {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, StructSerializer, StructDeserializer)]
        pub enum TestEnum2 {
            VariantA = 0x09,
            VariantB = 0x00,
            VariantC = 0x01,
        }

        let byte = [0x09];

        let mut cursor = Cursor::new(&byte);
        let mut decoded = TestEnum2::VariantB; // Initialize with any variant

        cursor.read_struct(&mut decoded).unwrap();

        assert_eq!(decoded as u8, TestEnum2::VariantA as u8);
    }
}
