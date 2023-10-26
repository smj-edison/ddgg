use alloc::vec::Vec;
use core::mem;

#[cfg(feature = "serde_string_indexes")]
use alloc::{fmt, format};
#[cfg(feature = "serde_string_indexes")]
use serde::de;
#[cfg(any(feature = "serde", feature = "serde_string_indexes"))]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_string_indexes")),
    derive(Serialize, Deserialize)
)]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_string_indexes")),
    serde(rename_all = "camelCase")
)]
pub struct Index {
    pub(crate) index: usize,
    pub(crate) generation: u32,
}

impl core::fmt::Debug for Index {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", (self.index, self.generation))
    }
}

#[cfg(feature = "serde_string_indexes")]
impl Serialize for Index {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}.{}", self.index, self.generation))
    }
}

#[cfg(feature = "serde_string_indexes")]
impl<'de> Deserialize<'de> for Index {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> de::Visitor<'de> for StringVisitor {
            type Value = Index;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct StringVisitor")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let (index_str, generation_str) = v.split_once('.').ok_or_else(|| {
                    de::Error::invalid_value(de::Unexpected::Str(v), &"{index}.{generation}")
                })?;

                let index: usize = str::parse(index_str).or_else(|_| {
                    Err(de::Error::invalid_value(
                        de::Unexpected::Str(index_str),
                        &"usize index",
                    ))
                })?;

                let generation: u32 = str::parse(generation_str).or_else(|_| {
                    Err(de::Error::invalid_value(
                        de::Unexpected::Str(generation_str),
                        &"u32 generation",
                    ))
                })?;

                Ok(Index { index, generation })
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

impl Index {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "js_names", serde(tag = "variant", content = "data"))]
pub enum Element<T> {
    Occupied(T, u32),
    Open(u32),
}

impl<T> Element<T> {
    pub fn as_occupied(self) -> Option<T> {
        match self {
            Self::Occupied(value, _) => Some(value),
            Self::Open(_) => None,
        }
    }

    pub fn generation(&self) -> u32 {
        match self {
            Self::Occupied(_, generation) => *generation,
            Self::Open(generation) => *generation,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(transparent)]
pub struct GenVec<T> {
    vec: Vec<Element<T>>,
}

impl<T> GenVec<T> {
    pub fn new() -> GenVec<T> {
        GenVec { vec: Vec::new() }
    }

    pub fn add(&mut self, value: T) -> Index {
        // check for an existing spot
        let open_slot_index = self.vec.iter().position(|elem| match elem {
            Element::Occupied(..) => false,
            Element::Open(generation) => generation > &0,
        });

        // there was an open slot, put it there
        if let Some(open_slot_index) = open_slot_index {
            let generation = self.vec[open_slot_index].generation();

            self.vec[open_slot_index] = Element::Occupied(value, generation);

            Index {
                index: open_slot_index,
                generation: generation,
            }
        } else {
            // else, add it to the end
            self.vec.push(Element::Occupied(value, 0));

            Index {
                index: self.vec.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        match self.vec[index.index] {
            Element::Occupied(ref value, generation) => {
                if generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            Element::Open(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.vec[index.index] {
            Element::Occupied(ref mut value, generation) => {
                if generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            Element::Open(_) => None,
        }
    }

    pub fn remove(&mut self, index: Index) -> Option<T> {
        let can_take = match self.vec[index.index] {
            Element::Occupied(_, generation) => generation == index.generation,
            Element::Open(_) => false,
        };

        if can_take {
            let removed = mem::replace(
                &mut self.vec[index.index],
                Element::Open(index.generation + 1),
            );

            Some(removed.as_occupied().unwrap())
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.vec.iter().filter_map(|element| match element {
            Element::Occupied(value, _) => Some(value),
            Element::Open(_) => None,
        })
    }

    pub fn indexes(&self) -> impl Iterator<Item = Index> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied(_, generation) => Some(Index {
                    index: i,
                    generation: *generation,
                }),
                Element::Open(_) => None,
            })
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &T)> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied(value, generation) => Some((
                    Index {
                        index: i,
                        generation: *generation,
                    },
                    value,
                )),
                Element::Open(_) => None,
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Index, &mut T)> + '_ {
        self.vec
            .iter_mut()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied(value, generation) => Some((
                    Index {
                        index: i,
                        generation: *generation,
                    },
                    value,
                )),
                Element::Open(_) => None,
            })
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Index, T)> {
        self.vec
            .into_iter()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied(value, generation) => Some((
                    Index {
                        index: i,
                        generation,
                    },
                    value,
                )),
                Element::Open(_) => None,
            })
    }

    pub(crate) fn remove_keep_generation(&mut self, index: Index) -> Option<T> {
        let can_take = match self.vec[index.index] {
            Element::Occupied(_, generation) => generation == index.generation,
            Element::Open(_) => false,
        };

        if can_take {
            let removed = mem::replace(&mut self.vec[index.index], Element::Open(index.generation));

            Some(removed.as_occupied().unwrap())
        } else {
            None
        }
    }

    pub(crate) fn vec_ref(&self) -> &Vec<Element<T>> {
        &self.vec
    }

    pub(crate) fn vec_ref_mut(&mut self) -> &mut Vec<Element<T>> {
        &mut self.vec
    }

    pub(crate) fn is_replaceable_by_index_rollback(&self, index: Index) -> bool {
        if let Element::Open(generation) = self.vec[index.index] {
            generation == index.generation + 1
        } else {
            false
        }
    }

    pub(crate) fn is_replaceable_by_index_apply(&self, index: Index) -> bool {
        if let Element::Open(generation) = self.vec[index.index] {
            generation == index.generation
        } else {
            false
        }
    }
}
