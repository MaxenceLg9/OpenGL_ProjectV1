use bitvec::vec::BitVec;
use std::any::Any;

pub trait PosTrait: Any {
    fn serialize(&self) -> BitVec<u8>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
