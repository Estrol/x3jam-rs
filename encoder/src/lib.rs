pub mod stringutil;

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

// impl StructEncodeImpl for std::ffi::CString {
//     #[inline(always)]
//     fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
//         let bytes = self.as_bytes_with_nul();
//         writer.write_all(bytes)
//     }
// }

impl StructEncodeImpl for &std::ffi::CStr {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.to_bytes_with_nul();
        writer.write_all(bytes)
    }
}

impl StructEncodeImpl for stringutil::CStrEx {
    #[inline(always)]
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut new_string_bytes = self.string.clone();
        new_string_bytes.push(0);

        writer.write_all(&new_string_bytes)
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

// impl StructDecodeImpl for std::ffi::CString {
//     fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
//         let mut bytes = Vec::new();
//         loop {
//             let byte = u8::impl_decode(reader)?;
//             bytes.push(byte);
//             if byte == 0 {
//                 break;
//             }
//         }
//         Ok(std::ffi::CString::from_vec_with_nul(bytes)
//             .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
//     }
// }

impl StructDecodeImpl for stringutil::CStrEx {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut bytes = Vec::new();
        loop {
            let byte = u8::impl_decode(reader)?;
            bytes.push(byte);
            if byte == 0 {
                break;
            }
        }

        let cstr = std::ffi::CStr::from_bytes_with_nul(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(stringutil::CStrEx::from_cstr(cstr))
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
