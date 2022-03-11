use std::marker::PhantomData;

use crate::types::CoordinateSpace;

pub type CMILMatrix  = CMatrix<CoordinateSpace::Shape,CoordinateSpace::Device>;
#[derive(Default, Clone)]
pub struct CMatrix<InCoordSpace, OutCoordSpace> {
    in_coord: PhantomData<InCoordSpace>,
    out_coord: PhantomData<OutCoordSpace>
}

impl<InCoordSpace, OutCoordSpace> CMatrix<InCoordSpace, OutCoordSpace> {
    pub fn Identity() -> Self { todo!() }
    pub fn GetM11(&self) -> f32 { todo!()}
    pub fn GetM12(&self) -> f32 { todo!()}
    pub fn GetM21(&self) -> f32 { todo!()}
    pub fn GetM22(&self) -> f32 { todo!()}
    pub fn GetDx(&self) -> f32 { todo!()}
    pub fn GetDy(&self) -> f32 { todo!()}

    pub fn SetM11(&mut self, r: f32) { todo!()}
    pub fn SetM12(&mut self, r: f32) { todo!()}
    pub fn SetM21(&mut self, r: f32) { todo!()}
    pub fn SetM22(&mut self, r: f32) { todo!()}
    pub fn SetDx(&mut self, r: f32) { todo!()}
    pub fn SetDy(&mut self, r: f32) { todo!()}

    pub fn SetToIdentity(&mut self) { todo!() }
}