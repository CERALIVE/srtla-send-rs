//! SRT sequence numbers as a 31-bit modular serial domain.
//!
//! SRT carries sequence numbers in the low 31 bits of a 32-bit word; bit 31 is
//! reserved as a flag (the control-packet marker in a DATA header, the
//! range-start marker in a NAK loss list). Sequence numbers therefore live in
//! `0..=0x7FFF_FFFF` and **wrap** from `0x7FFF_FFFF` back to `0`.
//!
//! Naive `u32` comparison is wrong across that wrap: `0x7FFF_FFFF < 0` is
//! `false` numerically, but in the serial domain `0` is the immediate successor
//! of `0x7FFF_FFFF`. [`SrtSeq`] wraps the raw word and exposes RFC1982-style
//! modular comparison over `2^31` so wrap-crossing arithmetic is correct by
//! construction.

/// Number of significant bits in an SRT sequence number.
pub const SRT_SEQ_BITS: u32 = 31;

/// Mask selecting the 31-bit sequence-number field (bit 31 cleared).
pub const SRT_SEQ_MASK: u32 = 0x7fff_ffff;

/// Half the serial space (`2^30`). A forward distance strictly below this means
/// "ahead"; a distance strictly above means "behind"; exactly this value is the
/// RFC1982 ambiguous midpoint.
const SRT_SEQ_HALF: u32 = 1 << (SRT_SEQ_BITS - 1);

/// An SRT sequence number: a `u32` constrained to the 31-bit serial domain
/// `0..=0x7FFF_FFFF`, with modular (wrap-aware) ordering.
///
/// The invariant "bit 31 is always clear" is upheld by every constructor:
/// [`SrtSeq::new`] masks, [`SrtSeq::from_u32_checked`] rejects.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord)]
pub struct SrtSeq(u32);

impl SrtSeq {
    /// The first sequence number in the domain.
    pub const ZERO: Self = Self(0);

    /// The last sequence number before the wrap back to [`SrtSeq::ZERO`].
    pub const MAX: Self = Self(SRT_SEQ_MASK);

    /// Construct from a raw word, **masking off** bit 31.
    ///
    /// Use this when bit 31 is a known protocol flag that has already been
    /// interpreted (e.g. a NAK range-start marker). Use
    /// [`SrtSeq::from_u32_checked`] when bit 31 being set means the word is
    /// malformed.
    #[inline]
    pub const fn new(raw: u32) -> Self {
        Self(raw & SRT_SEQ_MASK)
    }

    /// Construct from a raw word, returning `None` when bit 31 is set.
    ///
    /// This is the validating constructor for words that are supposed to be
    /// bare sequence numbers (e.g. a NAK range **end** word), where a set high
    /// bit indicates a corrupt or hostile frame rather than a flag.
    #[inline]
    pub const fn from_u32_checked(raw: u32) -> Option<Self> {
        if raw & !SRT_SEQ_MASK != 0 {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// The underlying 31-bit value.
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// The immediate successor in the serial domain.
    ///
    /// Wraps `0x7FFF_FFFF` → `0`. This is the only correct way to step a
    /// sequence number: `wrapping_add(1)` on the raw `u32` would produce
    /// `0x8000_0000`, which is not a sequence number at all.
    #[inline]
    pub const fn next(self) -> Self {
        Self((self.0 + 1) & SRT_SEQ_MASK)
    }

    /// Forward modular distance from `self` to `other`, i.e. how many
    /// [`SrtSeq::next`] steps take `self` to `other`.
    ///
    /// Always in `0..2^31`; `self.distance(self) == 0`. Note this is
    /// **directional**: `a.distance(b) + b.distance(a) == 2^31` unless `a == b`.
    #[inline]
    pub const fn distance(self, other: Self) -> u32 {
        other.0.wrapping_sub(self.0) & SRT_SEQ_MASK
    }

    /// RFC1982-style modular "strictly less than": `self` precedes `other`.
    ///
    /// True when the forward distance `self → other` is non-zero and strictly
    /// below the half-space `2^30`.
    ///
    /// # Ambiguity
    ///
    /// When the two values are exactly `2^30` apart, neither precedes the
    /// other: `a.serial_lt(b)` and `b.serial_lt(a)` are **both** `false` (and
    /// likewise for [`SrtSeq::serial_le`], since they are not equal). This is
    /// the RFC1982 undefined case — the antipodal points of the ring carry no
    /// ordering information. Callers must treat it as "unknown" and pick a
    /// conservative branch rather than assuming a total order.
    #[inline]
    pub const fn serial_lt(self, other: Self) -> bool {
        let d = self.distance(other);
        d != 0 && d < SRT_SEQ_HALF
    }

    /// RFC1982-style modular "less than or equal": `self == other` or
    /// `self` precedes `other`. Inherits the ambiguity documented on
    /// [`SrtSeq::serial_lt`] (the antipodal case yields `false`).
    #[inline]
    pub const fn serial_le(self, other: Self) -> bool {
        self.0 == other.0 || self.serial_lt(other)
    }

    /// RFC1982-style modular "strictly greater than" — the mirror of
    /// [`SrtSeq::serial_lt`].
    #[inline]
    pub const fn serial_gt(self, other: Self) -> bool {
        other.serial_lt(self)
    }

    /// RFC1982-style modular "greater than or equal" — the mirror of
    /// [`SrtSeq::serial_le`].
    #[inline]
    pub const fn serial_ge(self, other: Self) -> bool {
        other.serial_le(self)
    }
}

impl From<SrtSeq> for u32 {
    #[inline]
    fn from(s: SrtSeq) -> Self {
        s.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_masks_high_bit() {
        assert_eq!(SrtSeq::new(0x8000_0000).value(), 0);
        assert_eq!(SrtSeq::new(0xffff_ffff).value(), SRT_SEQ_MASK);
        assert_eq!(SrtSeq::new(0x8000_0064).value(), 100);
        // Already-valid values pass through untouched.
        assert_eq!(SrtSeq::new(12_345).value(), 12_345);
    }

    #[test]
    fn from_u32_checked_rejects_high_bit() {
        assert_eq!(SrtSeq::from_u32_checked(0), Some(SrtSeq::ZERO));
        assert_eq!(SrtSeq::from_u32_checked(SRT_SEQ_MASK), Some(SrtSeq::MAX));
        assert_eq!(SrtSeq::from_u32_checked(0x8000_0000), None);
        assert_eq!(SrtSeq::from_u32_checked(0xffff_ffff), None);
        assert_eq!(SrtSeq::from_u32_checked(0x8000_0001), None);
    }

    #[test]
    fn next_wraps_at_max() {
        assert_eq!(SrtSeq::ZERO.next(), SrtSeq::new(1));
        assert_eq!(SrtSeq::new(0x7fff_fffe).next(), SrtSeq::MAX);
        // The wrap: 0x7FFF_FFFF -> 0, NOT 0x8000_0000.
        assert_eq!(SrtSeq::MAX.next(), SrtSeq::ZERO);
        assert_eq!(SrtSeq::MAX.next().value(), 0);
    }

    #[test]
    fn distance_is_forward_and_modular() {
        assert_eq!(SrtSeq::new(10).distance(SrtSeq::new(10)), 0);
        assert_eq!(SrtSeq::new(10).distance(SrtSeq::new(13)), 3);
        // Backwards distance is the long way round the ring.
        assert_eq!(SrtSeq::new(13).distance(SrtSeq::new(10)), (1 << 31) - 3);
        // Across the wrap: 0x7FFF_FFFE -> 2 is four steps.
        assert_eq!(SrtSeq::new(0x7fff_fffe).distance(SrtSeq::new(2)), 4);
        assert_eq!(SrtSeq::MAX.distance(SrtSeq::ZERO), 1);
    }

    #[test]
    fn serial_lt_basic_ordering() {
        assert!(SrtSeq::new(1).serial_lt(SrtSeq::new(2)));
        assert!(!SrtSeq::new(2).serial_lt(SrtSeq::new(1)));
        assert!(!SrtSeq::new(7).serial_lt(SrtSeq::new(7)));
        assert!(SrtSeq::new(0).serial_lt(SrtSeq::new(1_000_000)));
    }

    #[test]
    fn serial_lt_across_wrap() {
        // The load-bearing pair: MAX precedes ZERO in the serial domain even
        // though 0x7FFF_FFFF > 0 numerically.
        assert!(SrtSeq::MAX.serial_lt(SrtSeq::ZERO));
        assert!(!SrtSeq::ZERO.serial_lt(SrtSeq::MAX));
        assert!(SrtSeq::new(0x7fff_fffe).serial_lt(SrtSeq::new(3)));
        assert!(!SrtSeq::new(3).serial_lt(SrtSeq::new(0x7fff_fffe)));
        // Raw u32 comparison disagrees — that is exactly the bug this type fixes.
        assert!(SrtSeq::MAX.value() > SrtSeq::ZERO.value());
    }

    #[test]
    fn serial_le_and_mirrors() {
        assert!(SrtSeq::new(5).serial_le(SrtSeq::new(5)));
        assert!(SrtSeq::new(5).serial_le(SrtSeq::new(6)));
        assert!(!SrtSeq::new(6).serial_le(SrtSeq::new(5)));
        assert!(SrtSeq::MAX.serial_le(SrtSeq::ZERO));
        assert!(SrtSeq::MAX.serial_le(SrtSeq::MAX));

        assert!(SrtSeq::new(6).serial_gt(SrtSeq::new(5)));
        assert!(!SrtSeq::new(5).serial_gt(SrtSeq::new(6)));
        assert!(SrtSeq::ZERO.serial_gt(SrtSeq::MAX));
        assert!(SrtSeq::new(6).serial_ge(SrtSeq::new(6)));
        assert!(SrtSeq::ZERO.serial_ge(SrtSeq::MAX));
    }

    /// RFC1982 midpoint: values exactly `2^30` apart are unordered in BOTH
    /// directions. Documented, not "fixed" — no total order exists there.
    #[test]
    fn serial_midpoint_is_ambiguous() {
        let a = SrtSeq::ZERO;
        let b = SrtSeq::new(SRT_SEQ_HALF);
        assert_eq!(a.distance(b), SRT_SEQ_HALF);
        assert_eq!(b.distance(a), SRT_SEQ_HALF);
        assert!(!a.serial_lt(b));
        assert!(!b.serial_lt(a));
        assert!(!a.serial_le(b));
        assert!(!b.serial_le(a));
        assert!(!a.serial_gt(b));
        assert!(!b.serial_gt(a));

        // Just inside the half-space the ordering IS defined.
        let just_before = SrtSeq::new(SRT_SEQ_HALF - 1);
        assert!(a.serial_lt(just_before));
        assert!(!just_before.serial_lt(a));
    }

    #[test]
    fn next_walks_the_whole_wrap_boundary() {
        let mut s = SrtSeq::new(0x7fff_fffd);
        let walked: Vec<u32> = (0..5)
            .map(|_| {
                let v = s.value();
                s = s.next();
                v
            })
            .collect();
        assert_eq!(walked, vec![0x7fff_fffd, 0x7fff_fffe, 0x7fff_ffff, 0, 1]);
    }

    #[test]
    fn value_never_has_high_bit_set() {
        for raw in [0u32, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff, 0xdead_beef] {
            assert_eq!(SrtSeq::new(raw).value() & 0x8000_0000, 0);
            assert_eq!(SrtSeq::new(raw).next().value() & 0x8000_0000, 0);
        }
    }
}
