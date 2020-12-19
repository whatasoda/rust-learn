#![allow(non_snake_case)]

mod database;
mod entity;
mod impls;
mod query;
mod wasm_utils;
use crate::database::Database;
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
use std::iter::FromIterator;

#[no_mangle]
static mut DB: Database<Root> = Database::new();

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
        consoleLog(&format!("{:?}", $v));
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Root {
    users: HashMap<u32, User>,
    games: HashMap<u32, Game>,
    allTags: TagRegistry,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: u32,
    name: String,
}

#[no_mangle]
pub extern "C" fn alloc(capacity: usize) -> js_value::Pointer {
    js_value::memory::alloc(capacity)
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: js_value::Pointer, capacity: usize) {
    js_value::memory::dealloc(ptr, capacity);
}

#[no_mangle]
pub extern "C" fn persist() {
    unsafe {
        DB.persist().write_js();
    }
}

#[no_mangle]
pub extern "C" fn init(ptr: js_value::Pointer) {
    unsafe {
        let snapshot = Vec::<u8>::read_from_js(ptr);
        DB.initialize(&snapshot, || Root {
            users: HashMap::new(),
            games: HashMap::new(),
            allTags: TagRegistry::new(),
        });
    }
}

#[no_mangle]
pub extern "C" fn updateGames(ptr: js_value::Pointer) {
    let json = String::read_from_js(ptr);
    if let Ok(games) = serde_json::from_str::<Vec<GameInput>>(&json) {
        unsafe {
            DB.exec_with_db(|mut root| {
                for game in &games {
                    let game = Game::from_game_input(game, &mut root.allTags);
                    root.games.insert(game.id, game);
                }
            })
            .ok();
        }
    }
}

#[derive(Serialize, Debug)]
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
    let idQueryInputs = IdQueryInput::read_from_js(idQueryInputs);
    let rcmQueryInputs = RecommendationQueryInput::read_many_from_js(rcmQueryInputs);
    // query.idQuery.build(queryInputs.into_iter());
    query.idQuery.build(vec![idQueryInputs].into_iter());
    query.recommendationQuery.build(rcmQueryInputs.into_iter());
    unsafe {
        DB.exec_with_db(|db| {
            let mut results = Vec::<GameQueryResult>::with_capacity(db.games.len());
            for (_, game) in db.games.iter() {
                if !query.idQuery.run(game) {
                    continue;
                }
                let mut score: Option<RecommendationScore> = None;
                if let Some(recommendations) = &game.recommendations {
                    score = match query.recommendationQuery.run(recommendations) {
                        Some(score) => Some(score),
                        None => continue,
                    };
                }
                results.push(GameQueryResult {
                    id: game.id.clone(),
                    name: game.name.clone(),
                    tags: game.tags.as_ref().cloned(),
                    releaseDate: game.releaseDate.as_ref().cloned(),
                    recommendations: score,
                });
            }
            serde_json::to_string(&results).unwrap().write_js();
        })
        .ok();
    }
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

#[no_mangle]
pub extern "C" fn getFullJson() {
    unsafe {
        DB.fullJSON().write_js();
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
