use std::fmt::Debug;
use std::iter::FromIterator;
use std::mem;
use std::ptr;

#[link(wasm_import_module = "ctx")]
extern "C" {
    fn resolve(ptr: usize, len: usize);
}

pub trait LinerJavaScriptInput
where
    Self: Sized,
    Self::Liner: Sized + Debug,
{
    type Liner;
    fn from_liner(input: Self::Liner) -> Self;

    fn read_from_js(pointer: js_value::Pointer) -> Self {
        unsafe {
            let pointer = pointer.cast::<Self::Liner>();
            let mut opt: Option<Self::Liner> = None;
            *(&mut opt as *mut Option<Self::Liner> as *mut Option<()>) = Some(());

            let mut dest = opt.unwrap();
            ptr::copy_nonoverlapping(pointer, &mut dest, 1);
            ptr::drop_in_place(pointer);
            Self::from_liner(dest)
        }
    }
    fn read_many_from_js(ptr: js_value::Pointer) -> Vec<Self> {
        Vec::from_iter(
            Vec::<js_value::Pointer>::read_from_js(ptr)
                .into_iter()
                .map(|ptr| Self::read_from_js(ptr)),
        )
    }
}

pub trait LinerJavaScriptOutput
where
    Self: Sized,
    Self::Liner: Sized,
{
    type Liner;
    const KIND: u32;
    fn to_liner(&self) -> Self::Liner;

    fn write_js(&self) {
        let src = (Self::KIND, self.to_liner());
        unsafe {
            resolve(
                &(src) as *const (u32, Self::Liner) as usize,
                mem::size_of::<Self::Liner>() + 4,
            );
        }
    }
}

pub mod js_value {
    use std::mem;
    use std::os::raw::c_void;
    pub type Pointer = *mut c_void;

    pub fn write<T>(src: T) -> Pointer
    where
        T: Sized,
    {
        let vec0 = vec![src];
        let ptr0 = vec0.as_ptr();
        mem::forget(vec0);
        let vec1: Vec<(u32, u32)> = vec![(ptr0 as u32, mem::size_of::<T>() as u32)];
        let ptr1 = vec1.as_ptr() as Pointer;
        mem::forget(vec1);
        ptr1
    }

    pub mod memory {
        use crate::wasm_utils::js_value::Pointer;
        use std::mem;

        pub fn alloc(capacity: usize) -> Pointer {
            let mut buf = Vec::with_capacity(capacity);
            let ptr = buf.as_mut_ptr();
            mem::forget(buf);
            ptr
        }
        pub fn dealloc(ptr: Pointer, capacity: usize) {
            let _ = unsafe { Vec::from_raw_parts(ptr, 0, capacity) };
        }
    }
}
