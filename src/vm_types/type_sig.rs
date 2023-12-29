#[non_exhaustive]
pub struct TypeSig;

// the size is set to 'usize' for alignment convenience, technically this can be set to u8
// but then the alignment would be 1, in some cases this indeed saves the space, for example
// an i32 will cost 4 bytes, where
// 1 _ _ _ i i i i
// in which those 'i' represent the 4 bytes of the int, the '1' means the signature of the
// int type, the three '_' is the alignment padding
// however, as we uses usize, the layout will effectively become:
// 1 _ _ _ _ _ _ _ i i i i
// which wastes 4 bytes and will presumably waste more if the following field is of alignment
// 8:
// 1 _ _ _ _ _ _ _ i i i i _ _ _ _ x x x x
// however, such problems are meant to be counted as premature optimizations and are supposed
// to be handled after the main functionalities.

// To handle generics, we use reference
// the generics exist only at compile time to perform static analysis
// for example, if we have a function that accepts t: T, and calls to
// f(t), then we can
impl TypeSig {
    pub const NAT: usize = 1;
    pub const INT: usize = 2;
    pub const DOUBLE: usize = 3;
    pub const CHAR: usize = 4;
    pub const BOOL: usize = 5;
    pub const REFERENCE: usize = 6;
    pub const PRODUCT: usize = 7;
    pub const RECORD: usize = 8;
    pub const SUM: usize = 9;

    pub fn type_sig_to_string(sig: usize) -> &'static str {
        match sig {
            Self::NAT => "Nat",
            Self::INT => "Int",
            Self::DOUBLE => "Double",
            Self::CHAR => "Char",
            Self::BOOL => "Bool",
            Self::REFERENCE => "Reference",
            Self::PRODUCT => "$Product",
            Self::RECORD => "$Record",
            Self::SUM => "$Sum",
            _ => unreachable!()
        }
    }
}