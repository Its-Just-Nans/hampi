use asn_codecs_derive::AperCodec;
use bitvec::prelude::*;

#[derive(Debug, AperCodec)]
#[asn(
    type = "BITSTRING",
    sz_extensible = true,
    sz_lb = 16i128,
    sz_ub = 16i128
)]
pub struct NRencryptionAlgorithms(BitVec<Msb0, u8>);

fn main() {
    eprintln!("BIT STRING");
}