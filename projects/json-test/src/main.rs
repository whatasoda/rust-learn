use bincode;
use rustbreak::{deser::Bincode, MemoryDatabase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::mem;
use std::os::raw::c_void;

#[derive(Deserialize)]
struct Response<T> {
    success: u8,
    results: T,
}

#[repr(C)]
#[derive(Deserialize, Debug, Serialize, Clone)]
struct RawData {
    end_date: u32,
    start_date: u32,
    rollup_type: String,
    rollups: Vec<RawRecommendation>,
}

#[derive(Deserialize, Debug)]
struct Rollups(Vec<RawRecommendation>);

#[repr(C)]
#[derive(Debug)]
struct Partial {
    end_date: u32,
    start_date: u32,
    rollup_type: (usize, usize, usize),
    rollups: (usize, usize, usize),
}

#[repr(C)]
#[derive(Debug)]
struct Liner(u32, u32, usize, usize, usize, usize, usize, usize);

#[derive(Deserialize, Debug, Serialize, Clone)]
struct RawRecommendation {
    date: u32,
    recommendations_up: u32,
    recommendations_down: u32,
}

// #[derive(Debug)]
// enum RollupType {
//     Month = 0,
//     Week = 1,
// }

fn main() -> rustbreak::Result<()> {
    let raw = fs::read_to_string("./data.json").expect("Something went wrong reading the file");
    let ptr = alloc();
    // print!("{:?}\n", mem::size_of::<c_void>());
    // print!("{:?}\n", RollupType::Month as u8);
    // print!("{:?}\n", RollupType::Week as u8);
    assign(ptr, &mut parse(&raw));
    let from_ptr = unsafe { std::ptr::read_volatile(ptr as *mut RawData) };
    let db = MemoryDatabase::<HashMap<u32, RawData>, Bincode>::memory(HashMap::new())?;
    db.write(|db| {
        db.insert(0, from_ptr);
    })?;
    db.save()?;
    match db.get_data(false) {
        Ok(data) => {
            print!("{:?}", serde_json::ser::to_string(&data));
            print!("{:?}", bincode::serialize(&data));
        }
        _ => (),
    };
    print!("{:?}", db.get_data(false));

    // db.convert_data(|a| {
    //     print!("{:?}", a);
    // });
    // let mut up = 0;
    // let mut down = 0;
    // for a in from_ptr.rollups {
    //     if a.date < 10 || a.date > 1598918400 {
    //         continue;
    //     } else {
    //         up += a.recommendations_up;
    //         down += a.recommendations_down;
    //     }
    // }
    // print!("{:?}", (up, down));
    // // print!("{:?}\n", from_ptr);
    // // let from_ptr = unsafe { std::ptr::read_volatile(ptr as *mut Liner) };
    // // print!("{:?}\n", from_ptr);
    dealloc(ptr);
    Ok(())
}

fn alloc() -> *mut c_void {
    let mut buf = Vec::with_capacity(1024);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

fn dealloc(ptr: *mut c_void) {
    let _ = unsafe { Vec::from_raw_parts(ptr, 0, 1024) };
}

fn parse(json_str: &String) -> Option<RawData> {
    let res: Response<RawData> = serde_json::from_str(json_str).unwrap();
    match res.success {
        1 => Option::Some(res.results),
        _ => Option::None,
    }
}

fn assign(dest: *mut c_void, results: &mut Option<RawData>) {
    match results {
        Some(results) => {
            let dest = unsafe { (dest as *mut RawData).as_mut().unwrap() };
            // dest.rollups = Vec::with_capacity(results.rollups.capacity());
            // dest.rollup_type = String::with_capacity(results.rollup_type.capacity());
            mem::swap(dest, results);
            // mem::forget(dest);
        }
        None => print!("failed"),
    };
}
