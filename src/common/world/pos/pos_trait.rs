use std::any::Any;
use bitvec::vec::BitVec;

pub trait PosTrait: Any {
    fn serialize(&self) -> BitVec<u8>;
    fn deserialize(pos_bits : BitVec<u8>) -> Box<dyn PosTrait> where Self: Sized;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}