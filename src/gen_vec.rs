use alloc::vec::Vec;
use core::{marker::PhantomData, mem, ops};

#[cfg(feature = "serde")]
use alloc::fmt;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Index {
    pub(crate) index: usize,
    pub(crate) generation: u32,
}

impl core::fmt::Debug for Index {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", (self.index, self.generation))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Index {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format_args!("{}.{}", self.index, self.generation))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Index {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> de::Visitor<'de> for StringVisitor {
            type Value = Index;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an index and generation in the form of {index}.{generation}")
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "js_names", serde(tag = "variant", content = "data"))]
pub enum Element<T> {
    Occupied {
        value: T,
        generation: u32,
    },
    Open {
        generation: u32,
        next: Option<usize>,
    },
}

impl<T> Element<T> {
    pub fn as_occupied(self) -> Option<T> {
        match self {
            Self::Occupied { value: element, .. } => Some(element),
            Self::Open { .. } => None,
        }
    }

    pub fn generation(&self) -> u32 {
        match self {
            Self::Occupied { generation, .. } => *generation,
            Self::Open { generation, .. } => *generation,
        }
    }

    pub fn next_open(&self) -> Option<usize> {
        match self {
            Self::Occupied { .. } => None,
            Self::Open { next, .. } => *next,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenVec<T> {
    pub(crate) vec: Vec<Element<T>>,
    pub(crate) next_open_slot: Option<usize>,
}

impl<T> GenVec<T> {
    pub fn new() -> GenVec<T> {
        GenVec {
            vec: Vec::new(),
            next_open_slot: None,
        }
    }

    pub fn add(&mut self, value: T) -> Index {
        // there was an open slot, put it there
        if let Some(open_slot_index) = self.next_open_slot {
            let generation = self.vec[open_slot_index].generation();
            let next_open = self.vec[open_slot_index].next_open();

            self.vec[open_slot_index] = Element::Occupied { value, generation };

            self.next_open_slot = next_open;

            Index {
                index: open_slot_index,
                generation: generation,
            }
        } else {
            // else, add it to the end
            self.vec.push(Element::Occupied {
                value,
                generation: 0,
            });

            Index {
                index: self.vec.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        match self.vec.get(index.index) {
            Some(Element::Occupied { value, generation }) => {
                if *generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.vec[index.index] {
            Element::Occupied {
                ref mut value,
                generation,
            } => {
                if generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            Element::Open { .. } => None,
        }
    }

    pub fn remove(&mut self, index: Index) -> Option<T> {
        let can_take = self.get(index).is_some();

        if can_take {
            let removed = mem::replace(
                &mut self.vec[index.index],
                Element::Open {
                    generation: index.generation + 1,
                    next: self.next_open_slot,
                },
            );

            self.next_open_slot = Some(index.index);

            Some(removed.as_occupied().expect("to exist"))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.indexes().count()
    }

    pub fn is_empty(&self) -> bool {
        let is_any = self
            .vec
            .iter()
            .any(|x| matches!(x, Element::Occupied { .. }));

        !is_any
    }

    pub fn indexes(&self) -> impl Iterator<Item = Index> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied { generation, .. } => Some(Index {
                    index: i,
                    generation: *generation,
                }),
                Element::Open { .. } => None,
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
                Element::Occupied { value, generation } => Some((
                    Index {
                        index: i,
                        generation: *generation,
                    },
                    value,
                )),
                Element::Open { .. } => None,
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Index, &mut T)> + '_ {
        self.vec
            .iter_mut()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied { value, generation } => Some((
                    Index {
                        index: i,
                        generation: *generation,
                    },
                    value,
                )),
                Element::Open { .. } => None,
            })
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Index, T)> {
        self.vec
            .into_iter()
            .enumerate()
            .filter_map(|(i, element)| match element {
                Element::Occupied { value, generation } => Some((
                    Index {
                        index: i,
                        generation,
                    },
                    value,
                )),
                Element::Open { .. } => None,
            })
    }

    pub(crate) fn remove_but_maintain_generation(&mut self, index: Index) -> Option<T> {
        let can_take = self.get(index).is_some();

        if can_take {
            let removed = mem::replace(
                &mut self.vec[index.index],
                Element::Open {
                    generation: index.generation,
                    next: self.next_open_slot,
                },
            );

            self.next_open_slot = Some(index.index);

            Some(removed.as_occupied().expect("to exist"))
        } else {
            None
        }
    }

    pub(crate) fn is_replaceable_by_index_rollback(&self, index: Index) -> bool {
        if let Element::Open { generation, .. } = self.vec[index.index] {
            generation == index.generation + 1
        } else {
            false
        }
    }

    pub(crate) fn is_replaceable_by_index_apply(&self, index: Index) -> bool {
        if let Element::Open { generation, .. } = self.vec[index.index] {
            generation == index.generation
        } else {
            false
        }
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for GenVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.iter())
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for GenVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct MapVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for MapVisitor<T> {
            type Value = GenVec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with Index for keys and T for value")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut gen_vec = GenVec::new();
                let gen_vec_ref = &mut gen_vec.vec;

                // loop through map entries
                while let Some((key, value)) = map.next_entry()? {
                    let key: Index = key;
                    let value: T = value;

                    // expand array if space is needed
                    if key.index >= gen_vec_ref.len() {
                        gen_vec_ref.resize_with(key.index + 1, || Element::Open {
                            generation: 0,
                            next: None,
                        });
                    }

                    gen_vec_ref[key.index] = Element::Occupied {
                        value: value,
                        generation: key.generation,
                    };
                }

                // reconstruct linked list of open spots
                let mut last_open_index: Option<usize> = None;

                for (index, element) in gen_vec_ref.iter_mut().enumerate() {
                    if let Element::Open { next, .. } = element {
                        *next = last_open_index;

                        last_open_index = Some(index);
                    }
                }

                gen_vec.next_open_slot = last_open_index;

                Ok(gen_vec)
            }
        }

        deserializer.deserialize_map(MapVisitor(PhantomData::<T>))
    }
}

impl<T> ops::Index<Index> for GenVec<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> ops::IndexMut<Index> for GenVec<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
