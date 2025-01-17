use std::rc::Rc;

use ark_ec::AffineRepr;

use super::{MillerLines, VerifierReferenceString};

pub(crate) struct VerifierMemReferenceString<G2Affine: AffineRepr> {
    g2_x: G2Affine,
    precomputed_g2_lines: Rc<Vec<MillerLines>>,
}

impl<G2Affine: AffineRepr> VerifierMemReferenceString<G2Affine> {
    pub(crate) fn new(_g2x: &[u8]) -> Self {
        // Add the necessary code to convert g2x bytes into g2::AffineElement
        // and initialize precomputed_g2_lines
        unimplemented!()
    }
}

impl<G2Affine: AffineRepr> VerifierReferenceString<G2Affine>
    for VerifierMemReferenceString<G2Affine>
{
    fn get_g2x(&self) -> G2Affine {
        self.g2_x
    }

    fn get_precomputed_g2_lines(&self) -> Rc<Vec<MillerLines>> {
        self.precomputed_g2_lines.clone()
    }
}
