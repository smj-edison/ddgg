use alloc::vec::Vec;
use core::mem;

#[derive(Clone, Copy)]
pub struct Index {
    index: usize,
    generation: u32,
}

enum Element<T> {
    Some(T, u32),
    None(u32),
}

impl<T> Element<T> {
    pub fn as_some(self) -> Option<T> {
        match self {
            Self::Some(value, _) => Some(value),
            Self::None(_) => None,
        }
    }

    pub fn generation(&self) -> u32 {
        match self {
            Self::Some(_, generation) => *generation,
            Self::None(generation) => *generation,
        }
    }
}

pub struct GenerationalVec<T> {
    vec: Vec<Element<T>>,
}

impl<T> GenerationalVec<T> {
    pub fn push(&mut self, value: T) -> Index {
        // check for an existing spot
        let open_slot_index = self.vec.iter().position(|elem| match elem {
            Element::Some(_, _) => false,
            Element::None(_) => true,
        });

        // there was an open slot, put it there
        if let Some(open_slot_index) = open_slot_index {
            let generation = self.vec[open_slot_index].generation();

            self.vec[open_slot_index] = Element::Some(value, generation + 1);

            Index {
                index: open_slot_index,
                generation: generation + 1,
            }
        } else {
            // else, add it to the end
            self.vec.push(Element::Some(value, 0));

            Index {
                index: self.vec.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        match self.vec[index.index] {
            Element::Some(ref value, generation) => {
                if generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            Element::None(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.vec[index.index] {
            Element::Some(ref mut value, generation) => {
                if generation == index.generation {
                    Some(value)
                } else {
                    None
                }
            }
            Element::None(_) => None,
        }
    }

    pub fn remove(&mut self, index: Index) -> Option<T> {
        let can_take = match self.vec[index.index] {
            Element::Some(_, generation) => generation == index.generation,
            Element::None(_) => false,
        };

        if can_take {
            let removed = mem::replace(
                &mut self.vec[index.index],
                Element::None(index.generation + 1),
            );

            Some(removed.as_some().unwrap())
        } else {
            None
        }
    }
}
