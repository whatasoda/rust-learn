pub mod query {
    pub mod game {
        use super::id::IdQuery;
        use super::recommendation::RecommendationQuery;

        pub struct GameQuery {
            pub idQuery: IdQuery,
            pub recommendationQuery: RecommendationQuery,
        }
        impl GameQuery {
            pub fn new() -> Self {
                GameQuery {
                    idQuery: IdQuery::new(),
                    recommendationQuery: RecommendationQuery::new(),
                }
            }
        }
    }
    pub mod id {
        use crate::entity::entity::game::Game;
        use std::collections::HashSet;

        #[derive(Debug)]
        pub enum QueryInput {
            GameId(IdFilterInput),
            TagId(IdFilterInput),
        }

        #[derive(Debug)]
        pub struct IdFilterInput {
            pub policy: FilterPolicy,
            pub list: Vec<u32>,
        }

        #[derive(Debug)]
        pub enum FilterPolicy {
            Include,
            Exclude,
        }

        pub struct IdFilter {
            shouldIncludes: Option<HashSet<u32>>,
            shouldExcludes: Option<HashSet<u32>>,
        }

        pub struct IdQuery {
            gameId: IdFilter,
            tagId: IdFilter,
        }

        impl IdFilter {
            pub fn new() -> Self {
                IdFilter {
                    shouldIncludes: None,
                    shouldExcludes: None,
                }
            }

            pub fn is_none(&self) -> bool {
                self.shouldExcludes.is_none() && self.shouldIncludes.is_none()
            }

            pub fn add_input(&mut self, input: IdFilterInput) {
                match input.policy {
                    FilterPolicy::Include if self.shouldIncludes.is_none() => {
                        self.shouldIncludes = Some(HashSet::<u32>::new());
                    }
                    FilterPolicy::Exclude if self.shouldExcludes.is_none() => {
                        self.shouldExcludes = Some(HashSet::<u32>::new());
                    }
                    _ => (),
                }
                let set = match input.policy {
                    FilterPolicy::Include => self.shouldIncludes.as_mut().unwrap(),
                    FilterPolicy::Exclude => self.shouldExcludes.as_mut().unwrap(),
                };
                set.extend(input.list);
            }

            fn verify_single(&self, compareee: &u32) -> bool {
                for (shouldHave, option) in vec![
                    (true, self.shouldIncludes.as_ref()),
                    (false, self.shouldExcludes.as_ref()),
                ] {
                    if let Some(set) = option {
                        if shouldHave != set.get(compareee).is_some() {
                            return false;
                        }
                    }
                }
                true
            }

            fn verify_list(&self, compareees: &Vec<u32>) -> bool {
                if self.is_none() {
                    return true;
                }
                let compareees: HashSet<u32> = compareees.into_iter().cloned().collect();
                for (shouldHave, option) in vec![
                    (true, self.shouldIncludes.as_ref()),
                    (false, self.shouldExcludes.as_ref()),
                ] {
                    if let Some(set) = option {
                        let mut intersection = set.intersection(&compareees);
                        if shouldHave != intersection.next().is_some() {
                            return false;
                        }
                    }
                }
                true
            }
        }

        impl IdQuery {
            pub fn new() -> Self {
                IdQuery {
                    gameId: IdFilter::new(),
                    tagId: IdFilter::new(),
                }
            }

            pub fn build<T>(&mut self, inputs: T)
            where
                T: Iterator<Item = QueryInput>,
            {
                inputs.for_each(|input| {
                    macro_rules! commit_inputs {
                        ($(($Field:ident, $field:ident)),*,) => {
                            match input {
                                $(
                                    QueryInput::$Field(filter) => self.$field.add_input(filter),
                                )*
                            }
                        };
                    }
                    commit_inputs!((GameId, gameId), (TagId, tagId),);
                });
            }

            pub fn run(&self, game: &Game) -> bool {
                return self.gameId.verify_single(&game.id)
                    && game
                        .tags
                        .as_ref()
                        .and_then(|tags| Some(self.tagId.verify_list(tags)))
                        .unwrap_or(true);
            }
        }
    }

    pub mod recommendation {
        use crate::entity::entity::recommendation::{Recommendation, RecommendationScore};
        use num_integer::Integer;
        use num_traits::ToPrimitive;

        pub enum QueryInput {
            Date(SimpleRangeInput),
            Total(SimpleRangeInput),
            Up(ComplexRangeInput),
            Down(ComplexRangeInput),
            Sum(ComplexRangeInput),
        }
        pub struct SimpleRange(Option<SimpleRangeInput>);
        pub struct SimpleRangeInput {
            pub min: i32,
            pub max: i32,
        }
        pub struct ComplexRange(Option<ComplexRangeInput>);
        pub struct ComplexRangeInput {
            pub format: RangeFormat,
            pub range: SimpleRangeInput,
        }
        pub enum RangeFormat {
            Pct { baseline: u32 },
            Count,
        }

        pub struct RecommendationQuery {
            date: SimpleRange,
            total: SimpleRange,
            up: ComplexRange,
            down: ComplexRange,
            sum: ComplexRange,
        }

        impl RecommendationQuery {
            pub fn new() -> Self {
                RecommendationQuery {
                    date: SimpleRange::None(),
                    total: SimpleRange::None(),
                    up: ComplexRange::None(),
                    down: ComplexRange::None(),
                    sum: ComplexRange::None(),
                }
            }

            pub fn build<T>(&mut self, inputs: T)
            where
                T: Iterator<Item = QueryInput>,
            {
                inputs.for_each(|input| {
                    macro_rules! commit_query {
                        (
                            $(($Field:ident, $field:ident, $kind:ident)),*,
                        ) => (
                            match input {
                                $(QueryInput::$Field(filter) => {
                                    if self.$field.is_none() {
                                        self.$field = $kind(Some(filter));
                                    }
                                }),*
                            }
                        )
                    };
                    commit_query!(
                        (Date, date, SimpleRange),
                        (Total, total, SimpleRange),
                        (Up, up, ComplexRange),
                        (Down, down, ComplexRange),
                        (Sum, sum, ComplexRange),
                    );
                });
            }

            pub fn run(
                &self,
                recommendations: &Vec<Recommendation>,
            ) -> Option<RecommendationScore> {
                if self.date.is_none() {
                    self.evaluate(recommendations.iter())
                } else {
                    self.evaluate(
                        recommendations
                            .iter()
                            .filter(|r| self.date.verify_u32(r.date)),
                    )
                }
            }

            fn evaluate<'a, T>(&self, iter: T) -> Option<RecommendationScore>
            where
                T: Iterator<Item = &'a Recommendation>,
            {
                let mut up = 0;
                let mut down = 0;
                iter.for_each(|r| {
                    up += r.up;
                    down += r.down;
                });
                let total = up + down;
                let sum = if let (Some(up), Some(down)) = (up.to_i32(), down.to_i32()) {
                    up - down
                } else {
                    return None;
                };
                if self.total.verify_u32(total)
                    && self.up.verify_u32(up, total)
                    && self.down.verify_u32(down, total)
                    && self.sum.verify_i32(sum, total)
                {
                    Some(RecommendationScore { up, down, sum })
                } else {
                    None
                }
            }
        }

        impl SimpleRange {
            pub fn None() -> Self {
                SimpleRange(None)
            }
            pub fn is_none(&self) -> bool {
                self.0.is_none()
            }
            pub fn verify_u32(&self, v: u32) -> bool {
                self.0.is_none() || self.0.as_ref().unwrap().verify_u32(v)
            }
            // pub fn verify_i32(&self, v: i32) -> bool {
            //     self.0.is_none() || self.0.as_ref().unwrap().verify_i32(v)
            // }
        }

        impl SimpleRangeInput {
            fn verify_u32(&self, v: u32) -> bool {
                return (self.min.is_negative() || v >= self.min.to_u32().unwrap())
                    && (self.max.is_positive() || v <= self.max.to_u32().unwrap());
            }
            fn verify_i32(&self, v: i32) -> bool {
                v >= self.min && v <= self.max
            }
        }

        impl ComplexRange {
            pub fn None() -> Self {
                ComplexRange(None)
            }
            pub fn is_none(&self) -> bool {
                self.0.is_none()
            }
            pub fn verify_u32(&self, v: u32, total: u32) -> bool {
                self.0.is_none() || self.0.as_ref().unwrap().verify_u32(v, total)
            }
            pub fn verify_i32(&self, v: i32, total: u32) -> bool {
                self.0.is_none() || self.0.as_ref().unwrap().verify_i32(v, total)
            }
        }

        impl ComplexRangeInput {
            fn verify_u32(&self, v: u32, total: u32) -> bool {
                let compareee = match self.format {
                    RangeFormat::Count => v,
                    RangeFormat::Pct { baseline } if total != 0 => (v * baseline).div_ceil(&total),
                    _ => 0,
                };
                self.range.verify_u32(compareee)
            }
            fn verify_i32(&self, v: i32, total: u32) -> bool {
                let compareee = match self.format {
                    RangeFormat::Count => v,
                    RangeFormat::Pct { baseline } if total != 0 => {
                        if let (Some(total), Some(baseline)) = (total.to_i32(), baseline.to_i32()) {
                            (v * baseline).div_ceil(&total)
                        } else {
                            return false;
                        }
                    }
                    _ => 0,
                };
                self.range.verify_i32(compareee)
            }
        }
    }
}
