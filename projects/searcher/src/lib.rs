use serde::Deserialize;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;

#[link(wasm_import_module = "console")]
extern "C" {
    fn log(ptr: usize, len: usize);
}

fn console_log(msg: &str) {
    unsafe { log(msg.as_ptr() as usize, msg.len()) }
}

trait Empty<T> {
    fn empty() -> T;
}

#[derive(Deserialize)]
struct Response<T> {
    success: u8,
    results: T,
}

#[derive(Deserialize, Debug)]
struct RawData {
    start_date: u32,
    end_date: u32,
    rollup_type: String,
    rollups: Vec<Recommendation>,
}

#[repr(C)]
#[derive(Deserialize, Debug)]
pub struct Recommendation {
    date: u32,
    recommendations_up: u32,
    recommendations_down: u32,
}

#[repr(C, align(4))]
#[derive(Debug)]
struct Histogram {
    start_date: u32,
    end_date: u32,
    rollup_type: RollupType,
    rollups: Vec<Recommendation>,
}

impl Empty<Histogram> for Histogram {
    fn empty() -> Histogram {
        Histogram {
            start_date: 0,
            end_date: 0,
            rollup_type: RollupType::Month,
            rollups: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum RollupType {
    Month = 0,
    Week = 1,
}

#[no_mangle]
pub static BYTE_LENGTH_HISTOGRAM: usize = mem::size_of::<Histogram>();
#[no_mangle]
pub static BYTE_LENGTH_RECOMMENDATION: usize = mem::size_of::<Recommendation>();

#[no_mangle]
pub extern "C" fn alloc(capacity: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(capacity);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut c_void, capacity: usize) {
    let _ = Vec::from_raw_parts(ptr, 0, capacity);
}

#[no_mangle]
pub extern "C" fn convert(ptr: *mut c_void) -> *const c_void {
    let raw = load_str(ptr);
    resolve(parse(&raw).unwrap())
}

#[no_mangle]
pub extern "C" fn accumulate(ptr: *mut c_void, min: u32, max: u32) -> *const c_void {
    let histogram: &Histogram = read(ptr);
    let mut up = 0;
    let mut down = 0;
    for r in &histogram.rollups {
        if r.date >= min && r.date <= max {
            up += r.recommendations_up;
            down += r.recommendations_down;
        }
    }
    resolve((up, down))
}

fn load_str(ptr: *mut c_void) -> String {
    let cstr = unsafe { CStr::from_ptr(ptr as *const i8).to_str().unwrap() };
    String::from(cstr)
}

fn parse(json_str: &String) -> Option<Histogram> {
    console_log("Hello!");
    let res: Response<RawData> = serde_json::from_str(json_str).unwrap();
    match res.success {
        1 => Option::Some(Histogram {
            start_date: res.results.start_date,
            end_date: res.results.end_date,
            rollup_type: match &*res.results.rollup_type {
                "month" => RollupType::Month,
                "week" => RollupType::Week,
                _ => RollupType::Week,
            },
            rollups: res.results.rollups,
        }),
        _ => Option::None,
    }
}

fn read<'a, T>(ptr: *mut c_void) -> &'a T {
    unsafe { (ptr as *const T).as_ref() }.unwrap()
}

fn resolve<T>(src: T) -> *const c_void {
    let mut vec = vec![src];
    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    ptr as *const c_void
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
