#![allow(non_snake_case)]

mod entity;
mod impls;
mod query;
mod wasm_utils;
use crate::entity::entity::game::{Game, GameInput};
use crate::entity::entity::recommendation::RecommendationScore;
use crate::entity::entity::tag::TagRegistry;
use crate::query::query::game::GameQuery;
use crate::query::query::id::QueryInput as IdQueryInput;
use crate::query::query::recommendation::QueryInput as RecommendationQueryInput;
use crate::wasm_utils::js_value;
use crate::wasm_utils::{LinerJavaScriptInput, LinerJavaScriptOutput};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::mem;
use std::ptr;
static mut ROOT_PTR: Option<*mut Option<Root>> = None;
fn init_db(root: Root) {
    if unsafe { ROOT_PTR.is_none() } {
        let mut v = vec![Some(root)];
        unsafe {
            ROOT_PTR = Some(v.as_mut_ptr());
        }
        mem::forget(v);
    }
}
fn with_db<T>(task: T)
where
    T: FnOnce(&mut Root),
{
    if let Some(pointer) = unsafe { ROOT_PTR } {
        let mut stack: Option<Root> = None;
        unsafe {
            // *(&mut stack as *mut Option<Root> as *mut Option<()>) = Some(());
            ptr::swap_nonoverlapping(pointer, &mut stack, 1);
            ptr::drop_in_place(pointer);
        }
        let mut root = stack.unwrap();
        task(&mut root);
        let mut v = vec![Some(root)];
        unsafe {
            ROOT_PTR = Some(v.as_mut_ptr());
        }
        mem::forget(v);
    }
}

#[link(wasm_import_module = "console")]
extern "C" {
    fn log(ptr: usize, len: usize);
}
pub fn consoleLog(msg: &str) {
    unsafe { log(msg.as_ptr() as usize, msg.len()) }
}
#[macro_export]
macro_rules! consoleLog {
    ($v:expr) => {
        consoleLog(&serde_json::to_string($v).unwrap());
    };
}

#[derive(Serialize, Deserialize)]
struct Root {
    users: HashMap<u32, User>,
    games: HashMap<u32, Game>,
    allTags: TagRegistry,
    c: u32,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

#[no_mangle]
pub extern "C" fn alloc(capacity: usize) -> js_value::Pointer {
    js_value::memory::alloc(capacity)
}

#[no_mangle]
pub extern "C" fn persist() {
    with_db(|root| bincode::serialize(root).unwrap().write_js());
}

#[no_mangle]
pub extern "C" fn getFullJson() {
    with_db(|root| serde_json::to_string(root).unwrap().write_js());
}

#[no_mangle]
pub extern "C" fn init(ptr: js_value::Pointer) {
    init_db(Root {
        users: HashMap::new(),
        games: HashMap::new(),
        allTags: TagRegistry::new(),
        c: 0,
    });
}

#[no_mangle]
pub extern "C" fn updateGames(ptr: js_value::Pointer) {
    let json = String::read_from_js(ptr);
    if let Ok(games) = serde_json::from_str::<Vec<GameInput>>(&json) {
        with_db(|root| {
            for game in games {
                let game = Game::from_game_input(game, &mut root.allTags);
                root.games.insert(game.id, game);
            }
        });
    }
}

#[derive(Serialize)]
struct GameQueryResult {
    id: u32,
    name: String,
    tags: Option<Vec<u32>>,
    releaseDate: Option<u32>,
    recommendations: Option<RecommendationScore>,
}

#[no_mangle]
pub extern "C" fn filterGames(idQueryInputs: js_value::Pointer, rcmQueryInputs: js_value::Pointer) {
    let mut query = GameQuery::new();
    let idQueryInputs = IdQueryInput::read_many_from_js(idQueryInputs);
    let rcmQueryInputs = RecommendationQueryInput::read_many_from_js(rcmQueryInputs);
    query.idQuery.build(idQueryInputs.into_iter());
    query.recommendationQuery.build(rcmQueryInputs.into_iter());
    let mut results = Vec::<GameQueryResult>::new();
    with_db(|root| {
        root.c += 1;
        for (_, game) in root.games.iter() {
            if !query.idQuery.run(game) {
                continue;
            }
            let mut score: Option<RecommendationScore> = None;
            if let Some(recommendations) = &game.recommendations {
                score = query.recommendationQuery.run(recommendations);
                if score.is_none() {
                    continue;
                }
            }
            results.push(GameQueryResult {
                id: game.id.clone(),
                name: game.name.clone(),
                tags: game.tags.as_ref().cloned(),
                releaseDate: game.releaseDate.as_ref().cloned(),
                recommendations: score,
            });
        }
    });
    serde_json::to_string(&results).unwrap().write_js();
}

// #[no_mangle]
// pub extern "C" fn updateUsers(ptr: js_value::Pointer) {
//     let json = String::read_from_js(ptr);
//     if let Ok(users) = serde_json::from_str::<Vec<User>>(&json) {
//         unsafe {
//             DB.exec_with_db(|db| {
//                 for user in &users {
//                     db.users.insert(user.id, user);
//                 }
//             })
//             .ok();
//         }
//     };
// }

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
