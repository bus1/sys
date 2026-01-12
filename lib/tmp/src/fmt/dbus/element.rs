//! # D-Bus Elements
//!
//! Elements are the building blocks of D-Bus Type Signatures. Some of them
//! describe types, others are used to create compound types. All elements have
//! an associated ASCII character, and thus they can be combined in a string
//! to describe compound types.
//!
//! When describing D-Bus Signatures in documentation, the signature string
//! is usually the preferred representation.
//!
//! All primitive types have exactly one element that represents it, but not
//! all elements represent a primitive type. Instead, some elements are use to
//! build compound types and do not represent a type by themselves.
//!
//! This module provides information about the individual elements that make up
//! a D-Bus Signature. Yet, parts of this module are obviously only useful when
//! used with full D-Bus Signatures, rather than just single elements. The
//! [`signature`](super::signature) module provides a parser for full
//! signatures.
//!
//! ## Examples
//!
//! The following are examples of D-Bus elements. See [`Element`] for the full
//! list of elements.
//!
//! - `u` represents an unsigned integer of a fixed 32-bit length. It is a
//!   primitive element and represents a full type just by itself.
//! - `s` represents a UTF-8 encoded string of dynamic length. It also is a
//!   primitive element and represents a full type just by itself.
//! - `a` represents an array of dynamic length. It is not a primitive element
//!   but part of a compound type. It must be followed by elements that
//!   describe the type of the elements of the array. For instance, `au` would
//!   be a D-Bus Signature describe arrays of 32-bit unsigned integers.

/// `Flag` is the underlying data-type of `FlagSet`. All data that can be
/// stored in a `FlagSet` is provided as `Flag`.
///
/// `Flag` contains a binary combination of the different flags named
/// `FLAG_*`. Some flags are actually a mask/shift combination that allows
/// storing data bigger than 1-bit in the flag set.
pub type Flag = u32;

/// This 7-bit mask stores the element identifier of this flag set. It is
/// stored by its enum discriminant of `Element`. 7-bits are reserved, yet
/// only 5 are currently needed. Future additions might make greater use of
/// the ASCII range, though. This entry can never be 0, as such no `Flag` value
/// can ever be 0.
const FLAG_ELEMENT_MASK:                Flag = 0b0111_1111 << FLAG_ELEMENT_SHIFT;
const FLAG_ELEMENT_SHIFT:               Flag = 0;
/// This 2-bit mask stores the type of node used to encode the signature
/// metadata. This is an implementation detail of
/// [`Sig`](super::signature::Sig).
const FLAG_NODE_MASK:                   Flag = 0b11 << FLAG_NODE_SHIFT;
const FLAG_NODE_SHIFT:                  Flag = 7;
/// This 2-bit mask defines the DVariant alignment of the type as the exponent
/// to a power of 2. The maximum alignment is thus `2^(2^2-1) = 2^3 = 8`.
const FLAG_DVAR_ALIGNMENT_MASK:         Flag = 0b11 << FLAG_DVAR_ALIGNMENT_SHIFT;
const FLAG_DVAR_ALIGNMENT_SHIFT:        Flag = 9;
/// This is the 2-bit range for GVariant alignment, analog to DVariant
/// alignment.
const FLAG_GVAR_ALIGNMENT_MASK:         Flag = 0b11 << FLAG_GVAR_ALIGNMENT_SHIFT;
const FLAG_GVAR_ALIGNMENT_SHIFT:        Flag = 11;
/// If set, the element is a valid prefix of a signature.
pub const FLAG_PREFIX:                  Flag = 0b1 << 13;
/// If set, the element opens a container.
pub const FLAG_OPEN:                    Flag = 0b1 << 14;
/// If set, the element closes a container.
pub const FLAG_CLOSE:                   Flag = 0b1 << 15;
/// If set, the element is a basic type. Basic types are always primitive
/// types. Compound types cannot be basic.
pub const FLAG_BASIC:                   Flag = 0b1 << 16;
/// If set, the type is dynamically sized. Its size property carries no
/// meaning.
pub const FLAG_DYNAMIC:                 Flag = 0b1 << 17;
/// If set, the type is either a variant element, or it has a variant element
/// embedded.
pub const FLAG_VARIANT:                 Flag = 0b1 << 18;
/// If set, the type is either a handle element, or it has a handle element
/// embedded.
pub const FLAG_HANDLE:                  Flag = 0b1 << 19;
/// If set, the unbound container adheres to the requirements of dicts.
pub const FLAG_DICT:                    Flag = 0b1 << 20;
/// If set, the type is not supported by DVariant.
pub const FLAG_DVAR_UNSUPPORTED:        Flag = 0b1 << 21;
/// If set, the DVar encoding of this type has a lower alignment than some of
/// its contained elements, thus prone to position dependence unless aligned to
/// >=8-bytes.
pub const FLAG_DVAR_MISALIGNED:         Flag = 0b1 << 22;

/// `FlagSet` describes the behavior of types and elements. It is part of the
/// metadata of types and elements.
///
/// Some flags describe static behavior of an element, other flags describe
/// behavior of compound types. Then some flags are internal implementation
/// details of [`element`](self) or [`signature`](super::signature).
#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(crate) struct FlagSet(core::num::NonZeroU32);

/// Elements are the primitives that make up a type. Every element has an
/// associated code, which is a printable ASCII identifier that uniquely
/// identifies the element.
///
/// This type enumerates all possible elements. Some of the elements map
/// one-to-one to basic types, but others must be combined to form proper
/// composite types like arrays, structures, and more.
///
/// The discriminant of this enumeration allows for a packed representation
/// of all elements. Note, however, that this discriminant is specific to
/// this implementation and does not match the code associated with an element.
/// Use this discriminant only when storing elements in lookup arrays or other
/// dense data structures private to your module. Discriminants can change
/// across releases.
///
/// The discriminant of an element is never 0. Hence, `Element` can be relied
/// on to be a non-zero type.
#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Element {
    U8 = 1,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F64,
    Bool,
    String,
    Object,
    Signature,
    Handle,
    Variant,
    Array,
    Maybe,
    StructOpen,
    StructClose,
    DictOpen,
    DictClose,
    // CAREFUL: Ensure that `ELEMENTS` and `test::consistency::all` are kept in
    //          sync with this enum. While the discriminants are not ABI, they
    //          all must be updated in sync.
}

/// The metadata of an element describes its behavior and properties.
///
/// There is usually no need to create this data-type at runtime. Instead, one
/// instance of this type is provided statically for all known elements.
struct ElementMeta {
    code: core::num::NonZeroU8,
    pair: Option<Element>,
    flags: FlagSet,
    dvar_size: u8,
    gvar_size: u8,
}

#[allow(clippy::too_many_arguments)]
const fn element(
    code: u8,
    element: Element,
    pair: Option<Element>,
    dvar_alignment_exp: u8,
    dvar_size: u8,
    gvar_alignment_exp: u8,
    gvar_size: u8,
    flags: Flag,
) -> ElementMeta {
    ElementMeta {
        code: core::num::NonZeroU8::new(code).unwrap(),
        pair: pair,
        flags: FlagSet::with(element, dvar_alignment_exp, gvar_alignment_exp, flags),
        dvar_size: dvar_size,
        gvar_size: gvar_size,
    }
}

const ELEMENTS: [ElementMeta; 20] = [
    element(b'y', Element::U8, None, 0, 1, 0, 1,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'q', Element::U16, None, 1, 2, 1, 2,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'n', Element::I16, None, 1, 2, 1, 2,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'u', Element::U32, None, 2, 4, 2, 4,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'i', Element::I32, None, 2, 4, 2, 4,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b't', Element::U64, None, 3, 8, 3, 8,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'x', Element::I64, None, 3, 8, 3, 8,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'd', Element::F64, None, 3, 8, 3, 8,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b'b', Element::Bool, None, 2, 4, 0, 1,
        FLAG_PREFIX | FLAG_BASIC,
    ),
    element(b's', Element::String, None, 2, 0, 0, 0,
        FLAG_PREFIX | FLAG_BASIC | FLAG_DYNAMIC,
    ),
    element(b'o', Element::Object, None, 2, 0, 0, 0,
        FLAG_PREFIX | FLAG_BASIC | FLAG_DYNAMIC,
    ),
    element(b'g', Element::Signature, None, 0, 0, 0, 0,
        FLAG_PREFIX | FLAG_BASIC | FLAG_DYNAMIC,
    ),
    element(b'h', Element::Handle, None, 2, 4, 2, 4,
        FLAG_PREFIX | FLAG_BASIC | FLAG_HANDLE,
    ),
    element(b'v', Element::Variant, None, 0, 0, 3, 0,
        FLAG_PREFIX | FLAG_DYNAMIC | FLAG_VARIANT | FLAG_DVAR_MISALIGNED,
    ),
    element(b'a', Element::Array, None, 2, 0, 0, 0,
        FLAG_PREFIX | FLAG_DYNAMIC | FLAG_OPEN,
    ),
    element(b'm', Element::Maybe, None, 2, 0, 0, 0,
        FLAG_PREFIX | FLAG_DYNAMIC | FLAG_OPEN | FLAG_DVAR_UNSUPPORTED,
    ),
    element(b'(', Element::StructOpen, Some(Element::StructClose), 3, 0, 0, 0,
        FLAG_PREFIX | FLAG_OPEN | FLAG_DICT,
    ),
    element(b')', Element::StructClose, Some(Element::StructOpen), 0, 0, 0, 0,
        FLAG_CLOSE,
    ),
    element(b'{', Element::DictOpen, Some(Element::DictClose), 3, 0, 0, 0,
        FLAG_PREFIX | FLAG_OPEN | FLAG_DICT,
    ),
    element(b'}', Element::DictClose, Some(Element::DictOpen), 0, 0, 0, 0,
        FLAG_CLOSE,
    ),
];

impl FlagSet {
    /// Create a new `FlagSet` with the given data.
    pub(crate) const fn with(
        element: Element,
        dvar_alignment_exp: u8,
        gvar_alignment_exp: u8,
        flags: Flag,
    ) -> Self {
        let el_mask: Flag = (element.id() as Flag) << FLAG_ELEMENT_SHIFT;
        let dv_mask: Flag = (dvar_alignment_exp as Flag) << FLAG_DVAR_ALIGNMENT_SHIFT;
        let gv_mask: Flag = (gvar_alignment_exp as Flag) << FLAG_GVAR_ALIGNMENT_SHIFT;

        assert!(flags & FLAG_ELEMENT_MASK == 0);
        assert!(flags & FLAG_DVAR_ALIGNMENT_MASK == 0);
        assert!(flags & FLAG_GVAR_ALIGNMENT_MASK == 0);
        assert!((el_mask >> FLAG_ELEMENT_SHIFT) as u8 == element.id());
        assert!((dv_mask >> FLAG_DVAR_ALIGNMENT_SHIFT) as u8 == dvar_alignment_exp);
        assert!((gv_mask >> FLAG_GVAR_ALIGNMENT_SHIFT) as u8 == gvar_alignment_exp);
        assert!(el_mask & !FLAG_ELEMENT_MASK == 0);
        assert!(dv_mask & !FLAG_DVAR_ALIGNMENT_MASK == 0);
        assert!(gv_mask & !FLAG_GVAR_ALIGNMENT_MASK == 0);

        // SAFETY: `element.id()` is never 0.
        unsafe {
            Self(core::num::NonZeroU32::new_unchecked(
                flags | el_mask | dv_mask | gv_mask,
            ))
        }
    }

    /// Create a new `FlagSet` from its raw value.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the passed value is a valid `FlagSet`.
    /// That usually means it must have been acquired via another `FlagSet`.
    pub(crate) const unsafe fn from_raw(v: Flag) -> Self {
        // SAFETY: Propagated to caller.
        Self(unsafe { core::num::NonZeroU32::new_unchecked(v) })
    }

    /// Check whether a `FlagSet` has all flags in `flags` set.
    ///
    /// This will return `false` if `flags` is only partially set.
    ///
    /// This can be called with one of the masks set in `flags`, yet it will
    /// only perform a bitwise check and thus unlikely to be useful.
    pub(crate) const fn all(&self, flags: Flag) -> bool {
        (self.0.get() & flags) == flags
    }

    /// Check whether a `FlagSet` has any flags in `flags` set.
    ///
    /// This will return `false` only if none of `flags` is set.
    ///
    /// This can be called with one of the masks set in `flags`, yet it will
    /// only perform a bitwise check and thus unlikely to be useful.
    pub(crate) const fn any(&self, flags: Flag) -> bool {
        (self.0.get() & flags) != 0
    }

    /// Yield the raw flags of the `FlagSet`.
    ///
    /// This value will contain both flags as well as masks of multi-bit data
    /// of the `FlagSet`.
    pub(crate) const fn get(&self) -> Flag {
        self.0.get()
    }

    /// Set the given flags on the `FlagSet`.
    ///
    /// Set all bits given in `flags` on this `FlagSet`. Bits of the element
    /// mask are cleared from `flags`.
    pub(crate) const fn set(&mut self, flags: Flag) {
        // SAFETY: `element.id()` is never 0.
        self.0 = unsafe {
            core::num::NonZeroU32::new_unchecked(
                self.0.get() | (flags & !FLAG_ELEMENT_MASK),
            )
        };
    }

    /// Clear the given flags from the `FlagSet`.
    ///
    /// Clear all bits given in `flags` on this `FlagSet`. Bits of the element
    /// mask are cleared from `flags`.
    pub(crate) const fn clear(&mut self, flags: Flag) {
        // SAFETY: `element.id()` is never 0.
        self.0 = unsafe {
            core::num::NonZeroU32::new_unchecked(
                self.0.get() & !(flags & !FLAG_ELEMENT_MASK),
            )
        };
    }

    /// Yield the element stored in the `FlagSet`.
    pub(crate) const fn element(&self) -> Element {
        let v = (self.0.get() & FLAG_ELEMENT_MASK) >> FLAG_ELEMENT_SHIFT;

        // SAFETY: A `FlagSet` always has a valid element-id stored.
        unsafe { Element::from_id(v as u8) }
    }

    /// Yield the node data stored in the `FlagSet`.
    pub(crate) const fn node(&self) -> u8 {
        ((self.0.get() & FLAG_NODE_MASK) >> FLAG_NODE_SHIFT) as u8
    }

    /// Yield the node data stored in the `FlagSet`.
    pub(crate) const fn set_node(&mut self, v: u8) {
        let mask: Flag = (v as Flag) << FLAG_NODE_SHIFT;

        assert!((mask >> FLAG_NODE_SHIFT) as u8 == v);
        assert!(mask & !FLAG_NODE_MASK == 0);

        // SAFETY: `element.id()` is never 0.
        self.0 = unsafe {
            core::num::NonZeroU32::new_unchecked(
                (self.0.get() & !FLAG_NODE_MASK) | mask,
            )
        };
    }

    /// Yield the DVar alignment exponent stored in the `FlagSet`.
    pub(crate) const fn dvar_alignment_exp(&self) -> u8 {
        ((self.0.get() & FLAG_DVAR_ALIGNMENT_MASK) >> FLAG_DVAR_ALIGNMENT_SHIFT) as u8
    }

    /// Yield the GVar alignment exponent stored in the `FlagSet`.
    pub(crate) const fn gvar_alignment_exp(&self) -> u8 {
        ((self.0.get() & FLAG_GVAR_ALIGNMENT_MASK) >> FLAG_GVAR_ALIGNMENT_SHIFT) as u8
    }

    /// Set the GVar alignment exponent stored in the `FlagSet`.
    pub(crate) const fn set_gvar_alignment_exp(&mut self, v: u8) {
        let mask: Flag = (v as Flag) << FLAG_GVAR_ALIGNMENT_SHIFT;

        assert!((mask >> FLAG_GVAR_ALIGNMENT_SHIFT) as u8 == v);
        assert!(mask & !FLAG_GVAR_ALIGNMENT_MASK == 0);

        // SAFETY: `element.id()` is never 0.
        self.0 = unsafe {
            core::num::NonZeroU32::new_unchecked(
                (self.0.get() & !FLAG_GVAR_ALIGNMENT_MASK) | mask,
            )
        };
    }
}

impl Element {
    /// Create a new element from its id. This is a simple transmute that does
    /// not modify the value.
    ///
    /// ## Safety
    ///
    /// `id` must be a valid element id gained from a call to `Self::id()`.
    pub const unsafe fn from_id(id: u8) -> Self {
        // SAFETY: The caller guarantees `id` was acquired via `Self::id()`,
        //         hence we can rely on `repr(u8)` to guarantee that we can
        //         restore the original.
        unsafe { core::mem::transmute(id) }
    }

    /// Create a new element from its code. If the code is not a valid element
    /// code, this will yield `None`. Otherwise, the element is returned.
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            b'y' => Some(Element::U8),
            b'q' => Some(Element::U16),
            b'n' => Some(Element::I16),
            b'u' => Some(Element::U32),
            b'i' => Some(Element::I32),
            b't' => Some(Element::U64),
            b'x' => Some(Element::I64),
            b'd' => Some(Element::F64),
            b'b' => Some(Element::Bool),
            b's' => Some(Element::String),
            b'o' => Some(Element::Object),
            b'g' => Some(Element::Signature),
            b'h' => Some(Element::Handle),
            b'v' => Some(Element::Variant),
            b'a' => Some(Element::Array),
            b'm' => Some(Element::Maybe),
            b'(' => Some(Element::StructOpen),
            b')' => Some(Element::StructClose),
            b'{' => Some(Element::DictOpen),
            b'}' => Some(Element::DictClose),
            _ => None,
        }
    }

    /// Yield the id of the element. This is the discriminant of the enum of
    /// the backing type. This value fits into a u8 and is never 0.
    pub const fn id(&self) -> u8 {
        *self as u8
    }

    const fn id_sz(&self) -> usize {
        self.id() as usize
    }

    const fn meta(&self) -> &ElementMeta {
        &ELEMENTS[self.id_sz().strict_sub(1)]
    }

    /// Yield the code associated with this element.
    pub const fn code(&self) -> u8 {
        self.meta().code.get()
    }

    pub const fn char(&self) -> char {
        // SAFETY: All element codes are ASCII codes.
        unsafe { char::from_u32_unchecked(self.code() as u32) }
    }

    pub const fn str(&self) -> &str {
        // SAFETY: `NonZeroU8` is compatible to `u8`, thus also `[u8; 1]`.
        let s = unsafe {
            core::mem::transmute::<
                &core::num::NonZeroU8,
                &[u8; 1],
            >(&self.meta().code)
        };
        // SAFETY: All element codes are ASCII, thus single-byte UTF-8.
        unsafe { str::from_utf8_unchecked(s) }
    }

    /// Yield the paired element, if any.
    pub const fn pair(&self) -> Option<Self> {
        self.meta().pair
    }

    /// Yield the `FlagSet` describing this element.
    pub(crate) const fn flags(&self) -> FlagSet {
        self.meta().flags
    }

    /// Check whether the element has all of the given flags set.
    pub const fn all(&self, flags: Flag) -> bool {
        self.flags().all(flags)
    }

    /// Check whether the element has any of the given flags set.
    pub const fn any(&self, flags: Flag) -> bool {
        self.flags().any(flags)
    }

    /// Yield the DVar alignment exponent of this element.
    pub const fn dvar_alignment_exp(&self) -> u8 {
        self.flags().dvar_alignment_exp()
    }

    /// Yield the GVar alignment exponent of this element.
    pub const fn gvar_alignment_exp(&self) -> u8 {
        self.flags().gvar_alignment_exp()
    }

    /// Yield the DVar size of this element.
    pub const fn dvar_size(&self) -> u8 {
        self.meta().dvar_size
    }

    /// Yield the GVar size of this element.
    pub const fn gvar_size(&self) -> u8 {
        self.meta().gvar_size
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem;

    // Verify data layout expectations.
    #[test]
    fn layout() {
        // We want `FlagSet` to expose its non-zero'ness, so we can wrap it in
        // `Option` for free.
        assert_eq!(
            mem::size_of::<FlagSet>(),
            mem::size_of::<Option<FlagSet>>(),
        );

        // We want `Element` to expose its non-zero'ness, so we can wrap it in
        // `Option` for free.
        assert_eq!(
            mem::size_of::<Element>(),
            mem::size_of::<Option<Element>>(),
        );

        // Since `ElementMeta` embeds `NonZeroU8` directly, it should also
        // be treated as a non-zero type and thus embeddable in an `Option` for
        // free.
        assert_eq!(
            mem::size_of::<ElementMeta>(),
            mem::size_of::<Option<ElementMeta>>(),
        );

        // Ensure that `FlagSet` has the same layout as a `u32`.
        assert_eq!(mem::align_of::<FlagSet>(), 4);
        assert_eq!(mem::size_of::<FlagSet>(), 4);

        // Ensure elements are limited to u8. We use them as dense alternative
        // to their ASCII representation and store it in `Flag`.
        assert_eq!(mem::size_of::<Element>(), 1);
    }

    // Verify the basic behavior of the `FlagSet` type.
    #[test]
    fn flags() {
        let mut v = FlagSet::with(Element::String, 1, 2, FLAG_BASIC | FLAG_DYNAMIC);

        assert_eq!(v.element(), Element::String);
        assert_eq!(v.dvar_alignment_exp(), 1);
        assert!(v.all(FLAG_BASIC) && v.all(FLAG_DYNAMIC) && v.all(FLAG_BASIC | FLAG_DYNAMIC));
        assert!(!v.all(FLAG_HANDLE) && !v.all(FLAG_VARIANT));
        assert!(!v.all(FLAG_BASIC | FLAG_HANDLE));

        let mask = v.get();

        v.set(FLAG_VARIANT);
        assert!(v.all(FLAG_VARIANT));
        assert_eq!(v.get(), mask | FLAG_VARIANT);
        v.clear(FLAG_VARIANT);
        assert!(!v.all(FLAG_VARIANT));
        assert_eq!(v.get(), mask);

        v.set(FLAG_VARIANT | FLAG_HANDLE);
        assert!(v.all(FLAG_VARIANT | FLAG_HANDLE));
        assert_eq!(v.get(), mask | FLAG_VARIANT | FLAG_HANDLE);
        v.clear(FLAG_VARIANT | FLAG_HANDLE);
        assert!(!v.all(FLAG_VARIANT | FLAG_HANDLE));
        assert_eq!(v.get(), mask);

        assert_eq!(v.node(), 0);
        v.set_node(3);
        assert_eq!(v.node(), 3);
        v.set_node(0);
        assert_eq!(v.get(), mask);

        assert_eq!(v.gvar_alignment_exp(), 2);
        v.set_gvar_alignment_exp(3);
        assert_eq!(v.gvar_alignment_exp(), 3);
        v.set_gvar_alignment_exp(2);
        assert_eq!(v.gvar_alignment_exp(), 2);
        assert_eq!(v.get(), mask);

        assert_eq!(unsafe { FlagSet::from_raw(mask) }, v);
        assert_ne!(unsafe { FlagSet::from_raw(mask | FLAG_VARIANT) }, v);
        v.set(FLAG_VARIANT);
        assert_ne!(unsafe { FlagSet::from_raw(mask) }, v);
        assert_eq!(unsafe { FlagSet::from_raw(mask | FLAG_VARIANT) }, v);
        v.clear(FLAG_VARIANT);
        assert_eq!(unsafe { FlagSet::from_raw(mask) }, v);
        assert_ne!(unsafe { FlagSet::from_raw(mask | FLAG_VARIANT) }, v);
        assert_eq!(v.get(), mask);
    }

    // Verify flags do not overlap.
    #[test]
    fn flags_overlap() {
        let flags: [Flag; _] = [
            FLAG_PREFIX,
            FLAG_OPEN,
            FLAG_CLOSE,
            FLAG_BASIC,
            FLAG_DYNAMIC,
            FLAG_VARIANT,
            FLAG_HANDLE,
            FLAG_DICT,
            FLAG_DVAR_UNSUPPORTED,
            FLAG_DVAR_MISALIGNED,
        ];
        let masks: [Flag; _] = [
            FLAG_ELEMENT_MASK,
            FLAG_NODE_MASK,
            FLAG_DVAR_ALIGNMENT_MASK,
            FLAG_GVAR_ALIGNMENT_MASK,
        ];
        let flag_first = FLAG_PREFIX;
        let flags_all = flags.iter().fold(0, |acc, v| acc | v);
        let masks_all = masks.iter().fold(0, |acc, v| acc | v);

        // Ensure flags are mutually exclusive.
        assert_eq!(flags_all.count_ones() as usize, flags.len());

        // Ensure masks are mutually exclusive.
        for i in masks {
            for j in masks {
                assert!(i == j || (i & j) == 0);
            }
        }

        // Ensure masks make up the entire space up to the first flag.
        assert_eq!(masks_all.count_ones(), (flag_first - 1).count_ones());
    }

    // Verify that masks are limited to a maximum width. First test the data
    // with valid parameters, then one test that panics on invalid data.
    #[test]
    fn flags_mask_limit() {
        let mut v = FlagSet::with(Element::U64, 0, 0, 0);
        v.set_node(0);
        v.set_node(1);
        v.set_node(2);
        v.set_node(3);
        let _v = FlagSet::with(Element::U64, 0, 0, 0);
        let _v = FlagSet::with(Element::U64, 3, 0, 0);
        let _v = FlagSet::with(Element::U64, 0, 3, 0);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_node() {
        let mut v = FlagSet::with(Element::U64, 0, 0, 0);
        v.set_node(4);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_node_high() {
        let mut v = FlagSet::with(Element::U64, 0, 0, 0);
        v.set_node(1 << 7);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_dvar() {
        let _v = FlagSet::with(Element::U64, 4, 0, 0);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_dvar_high() {
        let _v = FlagSet::with(Element::U64, 1 << 7, 0, 0);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_gvar() {
        let _v = FlagSet::with(Element::U64, 0, 4, 0);
    }
    #[should_panic]
    #[test]
    fn flags_mask_limit_gvar_high() {
        let _v = FlagSet::with(Element::U64, 0, 1 << 7, 0);
    }

    // Verify that elements reserved by the specification are not accidentally
    // resolved (e.g., because we used them for extensions).
    #[test]
    fn elements_reserved() {
        // reserved unconditionally: sentinal value for signatures
        assert!(Element::from_code(b'\0').is_none());

        // reserved for bindings: any complete struct
        assert!(Element::from_code(b'r').is_none());
        // reserved for bindings: any complete dict
        assert!(Element::from_code(b'e').is_none());

        // reserved for bindings: any complete type
        assert!(Element::from_code(b'*').is_none());
        // reserved for bindings: any basic type
        assert!(Element::from_code(b'?').is_none());

        // reserved for bindings; used by glib to encode calling conventions
        assert!(Element::from_code(b'@').is_none());
        assert!(Element::from_code(b'&').is_none());
        assert!(Element::from_code(b'^').is_none());
    }

    // Verify that the enumerations and mappings are consistent.
    #[test]
    fn elements_consistency() {
        let all = [
            Element::U8, Element::U16, Element::I16, Element::U32,
            Element::I32, Element::U64, Element::I64, Element::F64,
            Element::Bool, Element::String,
            Element::Object, Element::Signature,
            Element::Handle, Element::Variant,
            Element::Array, Element::Maybe,
            Element::StructOpen, Element::StructClose,
            Element::DictOpen, Element::DictClose,
        ];
        match all[0] {
            Element::U8 | Element::U16 | Element::I16 | Element::U32
            | Element::I32 | Element::U64 | Element::I64 | Element::F64
            | Element::Bool | Element::String
            | Element::Object | Element::Signature
            | Element::Handle | Element::Variant
            | Element::Array | Element::Maybe
            | Element::StructOpen | Element::StructClose
            | Element::DictOpen | Element::DictClose => {
                // This exhaustive match is used to ensure a compiler error
                // whenever the `Element` enum is modified. `all` must be
                // updated to reflect any changes.
            }
        };

        // Ensure `ELEMENTS` contains all elements (uniqueness is
        // checked later).
        assert_eq!(all.len(), ELEMENTS.len());

        // Ensure all elements...
        for (i, v) in all.iter().enumerate() {
            // ...have positive IDs
            assert_ne!(v.id(), 0);
            // ...have IDs equal to their discriminant
            assert_eq!(v.id(), *v as u8);
            // ...can be transmuted to their `repr(u8)`
            assert_eq!(v.id(), unsafe { mem::transmute::<Element, u8>(*v) });
            // ...can be created from their ID
            assert_eq!(unsafe { Element::from_id(v.id()) }, *v);
            // ...can be created from their code
            assert_eq!(Element::from_code(v.code()), Some(*v));
            // ...have ASCII codes
            assert_eq!(v.char() as u32, v.code() as u32);
            // ...have valid string representations
            assert_eq!(v.str().as_bytes(), &[v.code(); 1]);
            // ...are present in `ELEMENTS`
            assert_eq!(ELEMENTS[v.id_sz().strict_sub(1)].flags.element(), *v);
            // ...are stored densely in `ELEMENTS`
            assert_eq!(i.strict_add(1), v.id_sz());
        }

        // Ensure all element metadata...
        for (i, v) in ELEMENTS.iter().enumerate() {
            // ...is indexed correctly by its own element.
            assert_eq!(v.flags.element() as u8 as usize, i.strict_add(1));
            // ...has either no pairs, or pairs that are...
            if let Some(p) = v.pair {
                // ..non-reflexive
                assert_ne!(p, v.flags.element());
                // ..mutual
                assert_eq!(p.pair(), Some(v.flags.element()));
            }
        }

        // Ensure an element...
        for v in all.iter() {
            // ...has FLAG_PREFIX if, and only if, it has no FLAG_CLOSE.
            assert_eq!(v.all(FLAG_PREFIX), !v.all(FLAG_CLOSE));
            // ...has FLAG_OPEN if, and only if, it is one of "am({"
            assert!(
                !v.all(FLAG_OPEN)
                || *v == Element::Array
                || *v == Element::Maybe
                || *v == Element::StructOpen
                || *v == Element::DictOpen
            );
            // ...has FLAG_CLOSE if, and only if, it is one of ")}"
            assert!(
                !v.all(FLAG_CLOSE)
                || *v == Element::StructClose
                || *v == Element::DictClose
            );
            // ...never has both FLAG_OPEN and FLAG_CLOSE.
            assert!(!v.all(FLAG_OPEN | FLAG_CLOSE));
            // ...has FLAG_OPEN or FLAG_CLOSE if it has a pair.
            assert!(
                !v.pair().is_some()
                || v.any(FLAG_OPEN | FLAG_CLOSE)
            );
            // ...has FLAG_CLOSE only if it has a pair.
            assert!(
                !v.all(FLAG_CLOSE)
                || v.pair().is_some()
            );
        }
    }
}
