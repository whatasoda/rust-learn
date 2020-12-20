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
        let pointer = pointer.cast::<Self::Liner>();
        let mut liner: Option<Self::Liner> = None;
        unsafe {
            *(&mut liner as *mut Option<Self::Liner> as *mut Option<()>) = Some(());
            ptr::copy_nonoverlapping(pointer, liner.as_mut().unwrap(), 1);
            ptr::drop_in_place(pointer);
        }
        Self::from_liner(liner.unwrap())
    }
    fn read_many_from_js(ptr: js_value::Pointer) -> Vec<Self> {
        Vec::from_iter(
            Vec::<js_value::Pointer>::read_from_js(ptr)
                .into_iter()
                .map(Self::read_from_js),
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
                mem::size_of::<(u32, Self::Liner)>(),
            );
        }
    }
}

pub mod js_value {
    use std::os::raw::c_void;
    pub type Pointer = *mut c_void;

    pub mod memory {
        use crate::wasm_utils::js_value::Pointer;
        use std::mem;

        pub fn alloc(capacity: usize) -> Pointer {
            let mut buf = Vec::with_capacity(capacity);
            let ptr = buf.as_mut_ptr();
            mem::forget(buf);
            ptr
        }
        // pub fn dealloc(ptr: Pointer, capacity: usize) {
        //     let _ = unsafe { Vec::from_raw_parts(ptr, 0, capacity) };
        // }
    }
}
