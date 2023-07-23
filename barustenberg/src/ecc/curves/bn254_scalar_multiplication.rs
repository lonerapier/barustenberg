use ark_bn254::G1Projective;
use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::Zero;

pub(crate) type G1AffineGroup = <ark_ec::short_weierstrass::Affine<
    <ark_bn254::Config as ark_ec::bn::BnConfig>::G1Config,
> as ark_ec::AffineRepr>::Group;

pub(crate) fn is_point_at_infinity(point: &G1Projective) -> bool {
    !(point.x.is_zero() && point.y.is_zero()) && point.z.is_zero()
}
