pub mod entity {
    pub mod game {
        use super::recommendation::Recommendation;
        use super::tag::TagRegistry;
        use serde::{Deserialize, Serialize};
        use std::iter::FromIterator;

        #[derive(Serialize, Deserialize)]
        pub struct GameInput {
            pub id: u32,
            pub name: String,
            pub tags: Option<Vec<String>>,
            pub releaseDate: Option<u32>,
            pub recommendations: Option<Vec<Recommendation>>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct Game {
            pub id: u32,
            pub name: String,
            pub tags: Option<Vec<u32>>,
            pub releaseDate: Option<u32>,
            pub recommendations: Option<Vec<Recommendation>>,
        }
        impl Game {
            pub fn from_game_input(json: GameInput, allTags: &mut TagRegistry) -> Self {
                let tagsIds = match json.tags {
                    None => None,
                    Some(tags) => {
                        let iter = tags.into_iter().map(|tag| allTags.get_id_by_tag(tag));
                        Some(Vec::from_iter(iter))
                    }
                };
                Game {
                    id: json.id,
                    name: json.name,
                    tags: tagsIds,
                    releaseDate: json.releaseDate,
                    recommendations: json.recommendations,
                }
            }
        }
    }

    pub mod tag {
        use serde::de::{MapAccess, Visitor};
        use serde::ser::SerializeMap;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
        use std::collections::HashMap;
        use std::fmt;

        pub struct TagRegistry {
            map: HashMap<String, u32>,
        }
        impl TagRegistry {
            pub fn new() -> Self {
                TagRegistry {
                    map: HashMap::<String, u32>::new(),
                }
            }

            pub fn get_id_by_tag(&mut self, tag: String) -> u32 {
                match self.map.get(&tag) {
                    Some(id) => *id,
                    None => {
                        let newId = self.map.len() as u32;
                        self.map.insert(tag, newId);
                        newId
                    }
                }
            }
        }
        impl Serialize for TagRegistry {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut s = serializer.serialize_map(Some(self.map.len()))?;
                for (key, val) in self.map.iter() {
                    s.serialize_entry(val, key)?;
                }
                s.end()
            }
        }
        impl<'de> Deserialize<'de> for TagRegistry {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct MapVisitor;
                impl<'de> Visitor<'de> for MapVisitor {
                    type Value = HashMap<String, u32>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a map")
                    }

                    #[inline]
                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut values = HashMap::with_capacity(map.size_hint().unwrap_or(4096));
                        while let Some((key, value)) = map.next_entry()? {
                            values.insert(value, key); // swap
                        }
                        Ok(values)
                    }
                }

                let visitor = MapVisitor {};
                let map = deserializer.deserialize_map(visitor)?;
                Ok(TagRegistry { map })
            }
        }
    }

    pub mod recommendation {
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize, Clone)]
        pub struct Recommendation {
            pub date: u32,
            pub up: u32,
            pub down: u32,
        }

        #[derive(Serialize, Clone)]
        pub struct RecommendationScore {
            pub up: u32,
            pub down: u32,
            pub sum: i32,
        }
    }
}
