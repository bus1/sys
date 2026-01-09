//! # D-Bus Signatures
//!
//! D-Bus Signatures are a concatenation of D-Bus Elements as defined by the
//! [dbus::element](super::element) module. Not all concatenations are valid,
//! nor does a D-Bus Signature provide insight into the properties of compound
//! types. Therefore, this module provides a parser for D-Bus Signatures and
//! compiles an annotated and validated representation of a D-Bus Signature for
//! use during encoding and decoding.
//!
//! Every valid D-Bus Signature is a valid D-Bus Type, and vice versa. However,
//! given that `type` is a reserved keyword in Rust, this module uses
//! `Signature` and `Sig` as identifiers. A lot of D-Bus documentation might
//! refer to the same concepts as *Type*, but use *Signature* to refer to the
//! ASCII string representation. This module uses *Signature* to refer to both,
//! using *Signature String* to explicitly refer to the string representation.
//!
//! Furthermore, be aware that classic D-Bus uses type signatures that encode
//! a sequence of multiple types (which is also the value used by
//! `Element::Signature`). Such signatures do not form *Single Complete Types*
//! (as defined by the D-Bus Specification) and, as such, are not valid D-Bus
//! Types. They must be treated as a sequence of types, or enclosed in a
//! *Struct* to form a single complete type.
//!
//! ## Nomenclature
//!
//! This module uses the following terms to refer to specific properties of the
//! D-Bus type-system. These might not be well defined outside of this module.
//!
//! - *element*: An element is a single ASCII character that can be used in
//!       a type signature. This can either refer to a primitive type, or be
//!       part of a compound type signature. Hence, not all elements are
//!       valid types by themselves, but might only be valid in combination
//!       with other elements as part of a signature.
//! - *primitive type*: A type with signature length of 1 is called a
//!       primitive type.
//! - *compound type*: A non-primitive type is called a compound type.
//! - *bound container*: A bound container has an opening element, but no
//!       closing element and thus forms a compound type with its following
//!       single complete type.
//! - *unbound container*: An unbound container has both an opening and
//!       closing element and thus forms a compound type with all its
//!       enclosed types.
//! - *DVariant*: The D-Bus type encoding as introduced in the original D-Bus
//!       specification v0.8.
//! - *GVariant*: The D-Bus type encoding as introduced by glib.

// NB: This implementation avoids several standard Rust interfaces, because
//     those cannot be used in stable Rust. In particular:
//
//     - We sometimes `Option` and similar enum-based generics since they are
//       not stable in const-fn (due to their `[const] Destruct` bounds). But
//       destructuring via match expressions works, so most of the time
//       `Option` can still be used in const-fn.
//
//     - We avoid any basic traits like `Cmp` and `Eq`, as well as their
//       dependents like `cmp::max()` or `assert_eq!()`, since we cannot call
//       traits in const-fn.
//
//     - We avoid most generics, since making use of them requires trait
//       bounds, and those cannot be used in const-fn.
//
//     - We avoid custom DSTs, since their dynamic allocation is prone with UB
//       or unsupported on stable. Furthermore, DSTs have very poor integration
//       in general: We would want enum-DSTs to distinguish node types, but
//       Rust has no support for them.
//       Other than slices, DST support is pretty mediocre. Hence, we roll our
//       own custom DSTs using `[u64]`.

// XXX: We need to fix sub-slicing a signature: offset and position on the root
//      level will no longer be valid, but cannot be adjusted. Hence, we must
//      detect that when querying them.

use alloc::{borrow, boxed, string, sync};
use core::mem;

use crate::fmt::dbus;

/// This enumeration lists all possible error conditions of the signature
/// parser.
///
/// Most errors provide an index into the signature string to indicate where
/// the error occurred.
#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// The data buffer was insufficiently sized, as it is too small to
    /// fully contain all metadata for the given D-Bus Signature. This is a
    /// programming error, unless triggered explicitly.
    DataExceeded,
    /// The signature is empty, thus not a valid D-Bus Signature.
    SignatureEmpty,
    /// The signature is a sequence of multiple D-Bus Types, thus not
    /// a valid D-Bus Signature (as it is not a Single Complete Type). The
    /// break index shows the offset where the first type ends and the second
    /// type begins.
    SignatureSequence {
        break_idx: usize,
    },
    /// The signature has an open-ended container and is thus incomplete.
    SignatureIncomplete {
        container_idx: usize,
    },
    /// The signature contains a non-ASCII element.
    ElementInvalid {
        idx: usize,
        code: u8,
    },
    /// The signature contains an unknown ASCII element code.
    ElementUnknown {
        position: usize,
        code: u8,
    },
    /// The signature contains an unpaired element.
    ElementUnpaired {
        idx: usize,
    },
    /// The signature contains an incorrectly paired element.
    ElementMispaired {
        idx: usize,
        pair_idx: usize,
        expected: Option<dbus::Element>,
    },
    /// The signature contains a dictionary that is not a 2-tuple starting with
    /// a basic type.
    DictInvalid {
        position: usize,
    },
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Node8 {
    flags: dbus::element::FlagSet,
    offset_position_length: u16,
    dvar_size: u8,
    gvar_size: u8,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Node64 {
    flags: dbus::element::FlagSet,
    pad0: u32,
    offset: u64,
    position: u64,
    length: u64,
    dvar_size: u64,
    gvar_size: u64,
}

#[derive(Clone, Copy, Debug)]
enum NodeRef<'node> {
    Node8(&'node Node8),
    Node64(&'node Node64),
}

/// `Sig` is a dynamically sized type that contains validated information about
/// a D-Bus Signature.
///
/// A D-Bus Signature always represents a Single Complete Type (SCT). That is,
/// sequences of multiple types (as used by the *g* element of D-Bus
/// Signatures), or empty signatures are not valid D-Bus Signatures.
///
/// Different encodings exist for the D-Bus Type System. This type represents
/// signatures of all supported encodings, but not all encodings support all
/// signatures. Hence, signatures can be queried for compatibility to a given
/// encoding.
///
/// `Nodes` is a generic parameter that can be used to select between different
/// representations. The following representations are used by this module:
///
///   - `Sig<[u64]>`: This is the default representation, which can also be
///         referenced as `Sig`. It is a dynamically sized type (DST) and is
///         thus usually used behind a reference as `&Sig`.
///         This representation is used by default for any function that
///         required access to a D-Bus Signature.
///   - `Sig<[u64; N]>: This is a statically sized representation used to
///         create signatures on the stack. It uses const-generics to encode
///         the size of the data buffer.
///         This representation is used by default to create literals.
#[repr(transparent)]
pub struct Sig<Nodes: ?Sized = [u64]> {
    nodes: Nodes,
}

/// `Cursor` is an index into a D-Bus Signature that can be used to traverse
/// it.
///
/// Unlike iterators, the cursor can be moved freely to move and jump across
/// the different sub-types.
#[derive(Clone)]
// XXX: #[derive(Clone, Debug, Hash)]
// XXX: #[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Cursor<'sig, Owned = &'sig Sig> {
    sig: osi::mown::Mown<'sig, Sig, Owned>,
    idx: usize,
}

impl Node8 {
    // Pack offset, position, and length into a u16, 5 bits each.
    const fn opl(offset: u8, position: u8, length: u8) -> u16 {
        let o_mask: u16 = (offset as u16) << 0;
        let p_mask: u16 = (position as u16) << 5;
        let l_mask: u16 = (length as u16) << 10;

        assert!(o_mask & !(0x1f << 0) == 0);
        assert!(p_mask & !(0x1f << 5) == 0);
        assert!(l_mask & !(0x1f << 10) == 0);

        o_mask | p_mask | l_mask
    }

    const fn with(
        element: dbus::Element,
        offset: u8,
        position: u8,
    ) -> Self {
        let mut v = Self {
            flags: element.flags(),
            offset_position_length: Self::opl(offset, position, 1),
            dvar_size: element.dvar_size(),
            gvar_size: element.gvar_size(),
        };
        v.flags.set_node(Self::ID);
        v
    }

    const fn as_ref(&self) -> NodeRef<'_> {
        NodeRef::Node8(self)
    }

    const fn offset(&self) -> u8 {
        ((self.offset_position_length >> 0) & 0x1f) as u8
    }

    const fn position(&self) -> u8 {
        ((self.offset_position_length >> 5) & 0x1f) as u8
    }

    const fn length(&self) -> u8 {
        ((self.offset_position_length >> 10) & 0x1f) as u8
    }

    const fn coeff(&self) -> usize {
        Self::SIZE_COEFF
    }

    const fn set_length(&mut self, v: u8) {
        self.offset_position_length = Self::opl(self.offset(), self.position(), v);
    }
}

impl Node64 {
    const fn with(
        element: dbus::Element,
        offset: u64,
        position: u64,
    ) -> Self {
        let mut v = Self {
            flags: element.flags(),
            pad0: 0,
            offset: offset,
            position: position,
            length: 1,
            dvar_size: element.dvar_size() as u64,
            gvar_size: element.gvar_size() as u64,
        };
        v.flags.set_node(Self::ID);
        v
    }

    const fn as_ref(&self) -> NodeRef<'_> {
        NodeRef::Node64(self)
    }

    const fn offset(&self) -> u64 {
        self.offset
    }

    const fn position(&self) -> u64 {
        self.position
    }

    const fn length(&self) -> u64 {
        self.length
    }

    const fn coeff(&self) -> usize {
        Self::SIZE_COEFF
    }

    const fn set_length(&mut self, v: u64) {
        self.length = v;
    }
}

impl<'node> NodeRef<'node> {
    const fn flags(&self) -> dbus::element::FlagSet {
        match *self {
            NodeRef::Node8(v) => v.flags,
            NodeRef::Node64(v) => v.flags,
        }
    }

    const fn offset(&self) -> usize {
        match *self {
            NodeRef::Node8(v) => v.offset() as usize,
            NodeRef::Node64(v) => v.offset() as usize,
        }
    }

    const fn position(&self) -> usize {
        match *self {
            NodeRef::Node8(v) => v.position() as usize,
            NodeRef::Node64(v) => v.position() as usize,
        }
    }

    const fn length(&self) -> usize {
        match *self {
            NodeRef::Node8(v) => v.length() as usize,
            NodeRef::Node64(v) => v.length() as usize,
        }
    }

    const fn coeff(&self) -> usize {
        match *self {
            NodeRef::Node8(v) => v.coeff(),
            NodeRef::Node64(v) => v.coeff(),
        }
    }
}

macro_rules!
    impl_node
{ ($node:ty, $int:ty, $id:expr) => {
    impl $node {
        const ID: u8 = $id;
        const SIZE_COEFF: usize = mem::size_of::<Self>() / mem::size_of::<u64>();
        const LENGTH_MAX: usize = {
            let v = <$int>::MAX / 8;
            if v as usize as $int == v {
                v as usize
            } else {
                usize::MAX
            }
        };

        // Assert the layout of `Self` is as follows:
        // - Its alignment is equal to, or lower than, the alignment of `u64`.
        // - Its size is a multiple of the size of `u64`.
        // - Its size matches the size of `[u64; Self::SIZE_COEFF]`.
        // - It has no inner padding.
        // - It has a `FlagSet` at offset 0.
        // - `Drop` is not implemented for `Self`.
        const fn assert_layout() {
            // Verify as much as Rust allows us to do.
            assert!(mem::align_of::<Self>() <= mem::align_of::<u64>());
            assert!(mem::size_of::<Self>() % mem::size_of::<u64>() == 0);
            assert!(mem::size_of::<Self>() == mem::size_of::<[u64; Self::SIZE_COEFF]>());
            assert!(mem::offset_of!(Self, flags) == 0);
        }

        const fn element(&self) -> dbus::Element {
            self.flags.element()
        }

        const fn pair(&self) -> Option<dbus::Element> {
            self.flags.element().pair()
        }

        /// Derive a node buffer from an uninitialized data buffer.
        ///
        /// This is used to turn an uninitialized `&[MaybeUninit<u64>]` into
        /// `&[MaybeUninit<Self>]`. This ensures the types are compatible and
        /// splits of trailing padding.
        ///
        /// This returns a tuple consisting of the derived slice and a slice
        /// of the trailing padding. If [`Sig::size_for_length()`] was used to
        /// calculate the data size, the trailing padding will be of length 0.
        const fn derive_uninit(
            data: &mut [mem::MaybeUninit<u64>],
        ) -> (&mut [mem::MaybeUninit<Self>], &mut [mem::MaybeUninit<u64>]) {
            let rem = data.len() % Self::SIZE_COEFF;
            let (start, end) = data.split_at_mut(data.len() - rem);
            let len = start.len() / Self::SIZE_COEFF;
            let ptr = start.as_mut_ptr() as *mut _;

            Self::assert_layout();

            // SAFETY: `Self::assert_layout()` verifies that `Self` is
            //     compatible to `[u64; Self::SIZE_COEFF]`. `MaybeUninit` is
            //     `repr(transparent)` and adds no invariants.
            (unsafe { core::slice::from_raw_parts_mut(ptr, len) }, end)
        }

        /// Transmute a data buffer to a node buffer.
        ///
        /// This is used to turn a `&[u64]` into `&[Self]`. This ensures the
        /// types are compatible.
        ///
        /// ## Safety
        ///
        /// The caller must guarantee that `data` contains a valid array of
        /// `Self`. Trailing padding is allowed.
        const unsafe fn derive(data: &[u64]) -> &[Self] {
            let len = data.len() / Self::SIZE_COEFF;
            let ptr = data.as_ptr() as *const _;

            Self::assert_layout();

            // SAFETY: `Self::assert_layout()` verifies that `Self` is
            //     compatible to `[u64; Self::SIZE_COEFF]`. The caller ensures
            //     the validity of the content.
            unsafe { core::slice::from_raw_parts(ptr, len) }
        }

        /// `parse()` helper to calculate offsets and references for a new
        /// position given an index relative to the start of the signature.
        ///
        /// SAFETY: The element at position `idx` and all elements before it
        ///         must have been initialized.
        const unsafe fn parse_goto(
            idx: $int,
            nodes: &mut [mem::MaybeUninit<Self>],
        ) -> ($int, &mut Self, Option<$int>, Option<&mut Self>) {
            // SAFETY: `nodes[idx]` is initialized by the caller.
            let this = unsafe { &mut *nodes[idx as usize].as_mut_ptr() };
            if this.offset() == 0 {
                (idx, this, None, None)
            } else {
                let up_idx = idx.strict_sub(this.offset());
                (
                    idx,
                    this,
                    Some(up_idx),
                    // SAFETY: `up_idx` must be less than `idx` since
                    //         `this.offset` is non-zero and `strict_sub()`
                    //         cannot underflow. `nodes` is fully
                    //         initialized up to `idx` by the caller.
                    Some(unsafe { &mut *nodes[up_idx as usize].as_mut_ptr() }),
                )
            }
        }

        const fn parse(
            nodes: &mut [mem::MaybeUninit<Self>],
            signature: &[u8],
        ) -> Result<(), Error> {
            let mut signature_idx: usize = 0;
            let mut position: $int = 0;
            let mut o_up_idx: Option<$int> = None;

            // Ensure indices can be tracked with `$int`.
            if signature.len() > Self::LENGTH_MAX {
                return Err(Error::DataExceeded);
            }
            if signature.len() < 1 {
                return Err(Error::SignatureEmpty);
            }

            while signature_idx < signature.len() {
                let mut this_idx: $int;
                let mut this: &mut Self;
                let mut o_up: Option<&mut Self>;

                // Perform pre-processing: Parse the next element code, verify
                // its validity and write its corresponding type-element into
                // the next entry in `nodes`.
                {
                    let element: dbus::Element;

                    (this_idx, element) = {
                        let code: u8 = signature[signature_idx];
                        if !code.is_ascii() {
                            return Err(Error::ElementInvalid {
                                idx: signature_idx,
                                code: code,
                            });
                        }
                        let Some(v) = dbus::Element::from_code(code) else {
                            return Err(Error::ElementUnknown {
                                position: signature_idx,
                                code: code,
                            });
                        };
                        (signature_idx as $int, v)
                    };

                    nodes[this_idx as usize].write(
                        Self::with(
                            element,
                            // Element position is relative to the container,
                            // or root, if uncontained.
                            {
                                if let Some(v) = o_up_idx {
                                    this_idx.strict_sub(v)
                                } else {
                                    this_idx
                                }
                            },
                            position,
                        ),
                    );
                }

                // With pre-processing done, advance the iterator and get
                // mutable references to the element and its container for
                // post-processing.
                //
                // SAFETY: We just initialized `nodes` at `this_idx`, and
                //         everything before it was already initialized.
                (_, this, _, o_up) = unsafe { Self::parse_goto(this_idx, nodes) };
                position += 1;
                signature_idx += 1;

                // If this element opens a new bound or unbound container, push
                // it as the new innermost containing element, reset relative
                // counters and perform bookkeeping. Then continue with the
                // next element.
                // Once the container is closed, all information can be
                // restored from its type-element. Only then post-processing
                // for the container as a whole will be performed.
                if this.flags.all(dbus::element::FLAG_OPEN) {
                    o_up_idx = Some(this_idx);
                    position = 0;
                    continue;
                }

                // If this element closes an unbound container, we need to
                // ensure its opening and closing elements form a valid pair.
                // We then perform validity checks on the specific type of
                // container, before exiting it so we can handle the entire
                // container as a completed type.
                if this.flags.all(dbus::element::FLAG_CLOSE) {
                    let Some(up) = o_up else {
                        return Err(Error::ElementUnpaired {
                            idx: this_idx as usize,
                        });
                    };
                    let expected = this.pair().unwrap();
                    if expected.id() != up.element().id() {
                        return Err(Error::ElementMispaired {
                            idx: this_idx.strict_sub(up.offset()) as usize,
                            pair_idx: this_idx as usize,
                            expected: Some(expected),
                        });
                    }

                    // DVar does not support empty structs.
                    if let dbus::Element::StructClose = this.element() {
                        if position == 1 {
                            this.flags.set(dbus::element::FLAG_DVAR_UNSUPPORTED);
                        }
                    }

                    // Clear dict-flag if this is not a 2-tuple.
                    if up.flags.all(dbus::element::FLAG_DICT) && position != 3 {
                        up.flags.clear(dbus::element::FLAG_DICT);
                    }

                    // Refuse dicts that have the dict-flag cleared (which
                    // means they are either not a 2-tuple or have a non-basic
                    // first element).
                    if let dbus::Element::DictClose = this.element() {
                        if !up.flags.all(dbus::element::FLAG_DICT) {
                            return Err(Error::DictInvalid {
                                position: this_idx.strict_sub(this.offset()) as usize,
                            });
                        }
                    }

                    // Include the closing element in the container length.
                    up.set_length(up.length().strict_add(this.length()));

                    #[allow(unused_assignments)]
                    {
                        // SAFETY: We only reduce `this_idx`, so `nodes` is
                        //     still initialized up to the new index.
                        (this_idx, this, o_up_idx, o_up) = unsafe {
                            Self::parse_goto(
                                this_idx.strict_sub(this.offset()),
                                nodes,
                            )
                        };
                    }
                    position = this.position() + 1;
                }

                // At this point we finished a complete type. Propagate
                // relevant information to its container.
                while let Some(up) = o_up {
                    up.set_length(up.length().strict_add(this.length()));

                    // For DVar encoding, if this element exceeds the alignment
                    // of its container, remember this. But for a maximally
                    // aligned type, we can lift it again.
                    //
                    // Note that this is an inherent property of variants in
                    // DVar. The only other situation this can happen is for
                    // arrays, which have a strict alignment of 4-bytes. Thus,
                    // there is no need for fine-grained tracking of the
                    // misalignment, but a flag is sufficient.
                    if this.flags.dvar_alignment_exp() >= 3 {
                        this.flags.clear(dbus::element::FLAG_DVAR_MISALIGNED);
                    }
                    if (
                        this.flags.dvar_alignment_exp()
                        > up.flags.dvar_alignment_exp()
                    ) {
                        up.flags.set(dbus::element::FLAG_DVAR_MISALIGNED);
                    }

                    // Propagate GVar alignment from member elements to their
                    // container.
                    let v = this.flags.gvar_alignment_exp();
                    if v > up.flags.gvar_alignment_exp() {
                        up.flags.set_gvar_alignment_exp(v);
                    }

                    // Propagate collected flags to the container.
                    up.flags.set(this.flags.get() & (
                        dbus::element::FLAG_DYNAMIC
                        | dbus::element::FLAG_HANDLE
                        | dbus::element::FLAG_DVAR_UNSUPPORTED
                        | dbus::element::FLAG_DVAR_MISALIGNED
                    ));

                    // Clear dict-flag if the first element of an unbound
                    // container is not basic.
                    if (
                        up.flags.all(dbus::element::FLAG_DICT)
                        && this.position() == 0
                        && !this.flags.all(dbus::element::FLAG_BASIC)
                    ) {
                            up.flags.clear(dbus::element::FLAG_DICT);
                    }

                    // If the immediate container is bound, this type completes
                    // it and we proceed with post-processing the complete
                    // bound container. Otherwise, we advance the unbound
                    // container and are done.
                    if up.flags.all(dbus::element::FLAG_OPEN) && up.pair().is_none() {
                        // SAFETY: `o_up_idx` was already valid before.
                        #[allow(unused_assignments)]
                        {
                            (this_idx, this, o_up_idx, o_up) = unsafe {
                                Self::parse_goto(o_up_idx.unwrap(), nodes)
                            };
                        }
                        position = this.position() + 1;
                    } else {
                        if up.flags.all(dbus::element::FLAG_OPEN) && up.pair().is_some() {
                            up.dvar_size = up.dvar_size.next_multiple_of(
                                (1 << this.flags.dvar_alignment_exp()) as $int,
                            );
                            up.dvar_size = up.dvar_size.strict_add(this.dvar_size);

                            up.gvar_size = up.gvar_size.next_multiple_of(
                                (1 << this.flags.gvar_alignment_exp()) as $int,
                            );
                            up.gvar_size = up.gvar_size.strict_add(this.gvar_size);
                        }

                        break;
                    }
                }

                // If no container is left, the type is complete. Verify that
                // we processed the entire signature and then return.
                //
                // In all other cases (currently only possible with unbound
                // containers), the container is not complete and we simply
                // proceed with the next element.
                if o_up_idx.is_none() {
                    if signature_idx != signature.len() {
                        return Err(Error::SignatureSequence {
                            break_idx: signature_idx,
                        });
                    }
                    return Ok(());
                }
            }

            Err(Error::SignatureIncomplete {
                container_idx: o_up_idx.unwrap() as usize,
            })
        }
    }
}}

impl_node!(Node8, u8, 0);
impl_node!(Node64, u64, 3);

impl Sig {
    /// Calculate the required signature data buffer size given a signature
    /// length.
    ///
    /// The size of the data buffer equals the length of the signature times
    /// the size of the selected node type. Different node types exist, and
    /// the selected type must be encoded in the flags of the first node. It
    /// is up to the caller to ensure this.
    ///
    /// This function is exposed for use in macros but should not be used
    /// otherwise.
    #[doc(hidden)]
    pub const fn size_for_length(length: usize) -> usize {
        assert!(mem::size_of::<Node8>() % mem::size_of::<u64>() == 0);
        assert!(mem::align_of::<Node8>() <= mem::align_of::<u64>());
        assert!(mem::size_of::<Node64>() % mem::size_of::<u64>() == 0);
        assert!(mem::align_of::<Node64>() <= mem::align_of::<u64>());

        if length <= Node8::LENGTH_MAX {
            length.strict_mul(Node8::SIZE_COEFF)
        } else {
            assert!(length <= Node64::LENGTH_MAX);
            length.strict_mul(Node64::SIZE_COEFF)
        }
    }

    /// Parse the string representation of a D-Bus Signature into a signature
    /// data buffer.
    ///
    /// This will parse the entire signature given in `signature` and write the
    /// corresponding signature metadata into the data buffer `data`. The data
    /// buffer must be suitably sized using [`Self::size_for_length()`] based
    /// on the length of the signature string.
    ///
    /// On success, the entire data buffer will be properly initialized and can
    /// be used to create a [`Sig`].
    pub const fn parse(
        data: &mut [mem::MaybeUninit<u64>],
        signature: &[u8],
    ) -> Result<(), Error> {
        let (r, pad) = if signature.len() <= Node8::LENGTH_MAX {
            let (nodes, pad) = Node8::derive_uninit(data);
            (Node8::parse(nodes, signature), pad)
        } else {
            let (nodes, pad) = Node64::derive_uninit(data);
            (Node64::parse(nodes, signature), pad)
        };

        if r.is_ok() {
            let i = 0;
            while i < pad.len() {
                pad[i].write(0);
            }
        }

        r
    }

    /// Create a dynamically allocated D-Bus Signature from its string
    /// representation.
    ///
    /// This will allocate a new data buffer suitably sized for the given
    /// signature. It then uses `Sig::parse()` to parse the string signature.
    ///
    /// Any errors from `Sig::parse()` are propagated to the caller.
    pub fn new(
        signature: &[u8],
    ) -> Result<boxed::Box<Self>, Error> {
        let size = Self::size_for_length(signature.len());
        let mut buf = boxed::Box::<[u64]>::new_uninit_slice(size);

        Self::parse(&mut buf, signature)?;

        // SAFETY: `Self::parse()` initializes the full buffer on success.
        let data = unsafe { buf.assume_init() };

        // SAFETY: `Sig<[u64]>` is `repr(transparent)` over `[u64]`, and `data`
        //         is a valid signature.
        Ok(unsafe {
            mem::transmute::<boxed::Box<[u64]>, boxed::Box<Self>>(data)
        })
    }

    /// Create a dynamically allocated reference counted D-Bus Signature from
    /// its string representation.
    ///
    /// This will allocate a new data buffer suitably sized for the given
    /// signature. It then uses `Sig::parse()` to parse the string signature.
    ///
    /// Any errors from `Sig::parse()` are propagated to the caller.
    pub fn arc(
        signature: &[u8],
    ) -> Result<sync::Arc<Self>, Error> {
        let size = Self::size_for_length(signature.len());
        let mut buf = sync::Arc::<[u64]>::new_uninit_slice(size);

        Self::parse(sync::Arc::get_mut(&mut buf).unwrap(), signature)?;

        // SAFETY: `Self::parse()` initializes the full buffer on success.
        let data = unsafe { buf.assume_init() };

        // SAFETY: `Sig<[u64]>` is `repr(transparent)` over `[u64]`, and `data`
        //         is a valid signature.
        Ok(unsafe {
            mem::transmute::<sync::Arc<[u64]>, sync::Arc<Self>>(data)
        })
    }

    /// Yield the flags of this signature.
    ///
    /// The flags describe the entire type represented by this signature. See
    /// the definitions of the flags for details.
    pub(crate) const fn flags(&self) -> dbus::element::FlagSet {
        assert!(mem::offset_of!(Node8, flags) == 0);
        assert!(mem::offset_of!(Node64, flags) == 0);
        // SAFETY: All nodes have the `FlagSet` at offset 0, and a signature is
        //     never empty. Hence, by fetching the first 32 bits we can always
        //     access the `FlagSet` of the root element.
        let v = unsafe { mem::transmute::<u64, [u32; 2]>(self.nodes[0])[0] };
        unsafe { dbus::element::FlagSet::from_raw(v) }
    }

    /// Return the length of this signature, which is the number of elements
    /// in the signature.
    ///
    /// The size of the data buffer of a signature is not correlated to the
    /// length of the signature.
    pub const fn len(&self) -> usize {
        match self.flags().node() {
            Node8::ID => self.nodes.len() / Node8::SIZE_COEFF,
            Node64::ID => self.nodes.len() / Node8::SIZE_COEFF,
            _ => core::unreachable!(),
        }
    }

    const fn node_at(&self, idx: usize) -> Option<NodeRef<'_>> {
        match self.flags().node() {
            Node8::ID => {
                let v = unsafe { Node8::derive(&self.nodes) };
                if idx < v.len() { Some(v[idx].as_ref()) } else { None }
            }
            Node64::ID => {
                let v = unsafe { Node64::derive(&self.nodes) };
                if idx < v.len() { Some(v[idx].as_ref()) } else { None }
            }
            _ => core::unreachable!(),
        }
    }

    pub const fn at(&self, idx: usize) -> Option<&Self> {
        let Some(node) = self.node_at(idx) else {
            return None;
        };
        if !node.flags().all(dbus::element::FLAG_PREFIX) {
            return None;
        }

        let len = node.length();
        let coeff = node.coeff();

        // SAFETY: `Sig<[u64]>` is `repr(transparent)` over `[u64]`, and we
        //         verified `idx` is a valid signature prefix.
        unsafe {
            Some(mem::transmute::<&[u64], &Self>(
                // Use `split_at()` rather than range-indexing, since the
                // latter is not available in const-fn.
                &self.nodes
                    .split_at(idx.strict_mul(coeff)).1
                    .split_at(len.strict_mul(coeff)).0
            ))
        }
    }

    pub fn cursor(&self) -> Cursor<'_> {
        Cursor::new_borrowed(self)
    }

    /// Clone the signature into a new decoupled signature.
    ///
    /// The new signature is allocated on the heap and completely independent
    /// from the original.
    pub fn clone(&self) -> boxed::Box<Self> {
        let data: boxed::Box::<[u64]> = self.nodes.into();

        // SAFETY: `Sig<[u64]>` is `repr(transparent)` over `[u64]`, and `data`
        //         is a valid signature.
        unsafe {
            mem::transmute::<boxed::Box::<[u64]>, boxed::Box::<Self>>(data)
        }
    }

    pub fn to_string(&self) -> string::String {
        string::String::from_iter(self.cursor().map(|v| v.char()))
    }
}

// NB: Try to pick unique names for any methods here, to avoid conflicts with
//     `Sig`. The vast majority of call-sites will use `Sig` (rather than
//     `Self`). Also be aware that `Self: Deref<Target=Sig>`!
impl<const SIZE: usize> Sig<[u64; SIZE]> {
    /// Create a statically sized D-Bus Signature from its string
    /// representation.
    ///
    /// This will create a new data buffer suitably sized for the given
    /// signature. It then uses `Sig::parse()` to parse the string signature.
    /// On success, the data buffer is turned into a statically sized `Sig`
    /// and is returned to the caller.
    ///
    /// Any errors from `Sig::parse()` are propagated to the caller.
    pub const fn try_make(signature: &[u8]) -> Result<Self, Error> {
        let mut buf: mem::MaybeUninit<[u64; SIZE]>;
        let buf_v: &mut [mem::MaybeUninit<u64>; SIZE];

        // SAFETY: T and MaybeUninit<T> have the same layout.
        //
        // MSRV(unknown): This is available upstream as `transpose()` in
        //     `feature(maybe_uninit_uninit_array_transpose)` (#96097).
        buf = mem::MaybeUninit::uninit();
        buf_v = unsafe { mem::transmute(&mut buf) };

        match Sig::<[u64]>::parse(buf_v, signature) {
            // SAFETY: `Sig::parse()` always initializes the entire data
            //         buffer on success.
            Ok(()) => Ok(Self { nodes: unsafe { buf.assume_init() } }),
            Err(v) => Err(v),
        }
    }

    /// Create a statically sized D-Bus Signature literal from its string
    /// representation.
    ///
    /// This is a convenience wrapper around `Self::new()` which panics on
    /// error. This is meant for compile-time execution and will do its best
    /// to provide good compile-time diagnostics.
    pub const fn make(signature: &[u8]) -> Self {
        // MSRV(unknown): Ideally, we would print more helpful messages that
        //     use the extended data from `Error` to show where exactly an
        //     error happened. However, this requires the `Display` trait,
        //     and other formatting helpers, to work in const-fn.
        match Self::try_make(signature) {
            Ok(v) => v,
            Err(Error::DataExceeded) => panic!("invalid D-Bus signature data type: signature exceeds the data type"),
            Err(Error::SignatureEmpty) => panic!("invalid D-Bus signature: signature is empty"),
            Err(Error::SignatureSequence { .. }) => panic!("invalid D-Bus signature: signature is a sequence of multiple types"),
            Err(Error::SignatureIncomplete { .. }) => panic!("invalid D-Bus signature: signature is incomplete"),
            Err(Error::ElementInvalid { .. }) => panic!("invalid D-Bus signature: non-ASCII element in the signature"),
            Err(Error::ElementUnknown { .. }) => panic!("invalid D-Bus signature: unknown element in the signature"),
            Err(Error::ElementUnpaired { .. }) => panic!("invalid D-Bus signature: unpaired element"),
            Err(Error::ElementMispaired { .. }) => panic!("invalid D-Bus signature: mispaired element"),
            Err(Error::DictInvalid { .. }) => panic!("invalid D-Bus signature: invalid dictionary"),
        }
    }
}

impl<'sig, Owned> Cursor<'sig, Owned>
where
    Owned: borrow::Borrow<Sig>,
{
    /// Create a new cursor with full control over its signature and current
    /// position.
    ///
    /// Note that any position past the last element of the signature is
    /// clamped to one past the last element.
    pub fn with(
        sig: osi::mown::Mown<'sig, Sig, Owned>,
        idx: usize,
    ) -> Self {
        let len = match sig {
            osi::mown::Mown::Borrowed(v) => v.len(),
            osi::mown::Mown::Owned(ref v) => v.borrow().len(),
        };
        Self {
            sig: sig,
            idx: if idx < len { idx } else { len },
        }
    }

    /// Create a new cursor for a maybe-owned signature. The cursor will point
    /// to the first element of the signature.
    pub fn new(sig: osi::mown::Mown<'sig, Sig, Owned>) -> Self {
        Self::with(sig, 0)
    }

    /// Create a new cursor for a borrowed signature. The cursor will point to
    /// the first element of the signature.
    pub fn new_borrowed(sig: &'sig Sig) -> Self {
        Self::new(osi::mown::Mown::new_borrowed(sig))
    }

    /// Create a new cursor for an owned signature. The cursor will point to
    /// the first element of the signature.
    pub fn new_owned(sig: Owned) -> Self {
        Self::new(osi::mown::Mown::new_owned(sig))
    }

    /// Borrow the full signature that this cursor operates on.
    pub fn root(&self) -> &Sig {
        &*self.sig
    }

    /// Return the current index of the cursor relative to the start of the
    /// full signature.
    ///
    /// The index will always be less than, or equal to, the length of the full
    /// signature.
    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn raw(&mut self) -> (&Sig, &mut usize) {
        (&*self.sig, &mut self.idx)
    }

    pub fn sig_at(&self, idx: usize) -> Option<&Sig> {
        self.sig.at(idx)
    }

    fn flags_at(&self, idx: usize) -> Option<dbus::element::FlagSet> {
        self.sig.node_at(idx).map(|v| v.flags())
    }

    pub fn element_at(&self, idx: usize) -> Option<dbus::Element> {
        self.sig.node_at(idx).map(|v| v.flags().element())
    }

    fn offset_at(&self, idx: usize) -> Option<usize> {
        // Sub-slicing produces random root offsets. Force it to 0.
        if idx == 0 {
            Some(0)
        } else {
            self.sig.node_at(idx).map(|v| v.offset())
        }
    }

    pub fn position_at(&self, idx: usize) -> Option<usize> {
        // Sub-slicing produces random root positions. Force it to 0.
        if idx == 0 {
            Some(0)
        } else {
            self.sig.node_at(idx).map(|v| v.position())
        }
    }

    pub fn length_at(&self, idx: usize) -> Option<usize> {
        self.sig.node_at(idx).map(|v| v.length())
    }

    pub fn idx_next(&self) -> Option<usize> {
        if self.idx < self.sig.len() {
            Some(self.idx.strict_add(1))
        } else {
            None
        }
    }

    pub fn idx_prev(&self) -> Option<usize> {
        if self.idx > 0 {
            Some(self.idx.strict_sub(1))
        } else {
            None
        }
    }

    pub fn idx_step(&self) -> Option<usize> {
        match self.offset_at(self.idx) {
            None => {
                // If already past the end, stepping is blocked.
                None
            },
            Some(v) => {
                let flags = self.flags_at(self.idx).unwrap();
                let len = self.length_at(self.idx).unwrap();
                let up = self.element_at(self.idx.strict_sub(v)).unwrap();

                if v == 0 {
                    // Uncontained root can always be stepped once.
                    Some(self.idx.strict_add(len))
                } else if up.pair().is_none() {
                    // Bound containers cannot be stepped.
                    None
                } else if flags.all(dbus::element::FLAG_CLOSE) {
                    // Got to the end of the unbound container, so stepping is
                    // blocked.
                    None
                } else {
                    // There are still steps left in this unbound container.
                    Some(self.idx.strict_add(len))
                }
            },
        }
    }

    pub fn idx_down(&self) -> Option<usize> {
        self.flags_at(self.idx).and_then(|v| {
            v.all(dbus::element::FLAG_OPEN)
                .then(|| self.idx.strict_add(1))
        })
    }

    pub fn idx_up(&self) -> Option<usize> {
        self.offset_at(self.idx).and_then(|v| {
            (v > 0).then(|| self.idx.strict_sub(v))
        })
    }

    pub fn move_to(&mut self, idx: usize) -> &mut Self {
        self.idx = idx;
        self
    }

    pub fn maybe_move(&mut self, idx: Option<usize>) -> &mut Self {
        if let Some(v) = idx { self.move_to(v) } else { self }
    }

    pub fn move_next(&mut self) -> &mut Self {
        self.maybe_move(self.idx_next())
    }

    pub fn move_prev(&mut self) -> &mut Self {
        self.maybe_move(self.idx_prev())
    }

    pub fn move_step(&mut self) -> &mut Self {
        self.maybe_move(self.idx_step())
    }

    pub fn move_down(&mut self) -> &mut Self {
        self.maybe_move(self.idx_down())
    }

    pub fn move_up(&mut self) -> &mut Self {
        self.maybe_move(self.idx_up())
    }

    /// Return the signature at the current index.
    ///
    /// If the current index is at the end of the signature, or at a non-prefix
    /// element, this will return `None`. Otherwise, the signature is subsliced
    /// at the current position.
    pub fn sig(&self) -> Option<&Sig> {
        self.sig_at(self.idx)
    }

    /// Return the element at the current index.
    ///
    /// If the current index is at the end of the signature, this will return
    /// `None`.
    pub fn element(&self) -> Option<dbus::Element> {
        self.element_at(self.idx)
    }

    /// Return the contained element below the current index.
    ///
    /// If the current index is at the end of the signature, or is not a
    /// container, this will return `None`.
    pub fn down(&self) -> Option<dbus::Element> {
        self.idx_down().and_then(|v| self.element_at(v))
    }

    /// Return the containing element above the current index.
    ///
    /// If the current index is at the end of the signature, or is uncontained,
    /// this will return `None`.
    pub fn up(&self) -> Option<dbus::Element> {
        self.idx_up().and_then(|v| self.element_at(v))
    }

    /// Return the container position at the current index.
    ///
    /// For every container, all directly contained elements are enumerated
    /// starting with 0. This is called the container position. Note that
    /// closing elements have a position as well (which so happens to equal the
    /// number of fully contained types of this container).
    ///
    /// If the current index is at the end of the signature, this will return
    /// `None`.
    pub fn position(&self) -> Option<usize> {
        self.position_at(self.idx)
    }

    /// Return the type length at the current index.
    ///
    /// Every element in a signature has an assigned length, which encodes how
    /// long the single complete type starting with this element it. For
    /// non-prefix elements, this is arbitrarily set to 1.
    ///
    /// If the current index is at the end of the signature, this will return
    /// `None`.
    pub fn length(&self) -> Option<usize> {
        self.length_at(self.idx)
    }

    /// Return the DVar alignment exponent at the current index.
    ///
    /// The returned value is the exponent to a power of 2 describing the
    /// minimum alignment requirements of the type at the current position.
    ///
    /// If the current index is at the end of the signature, this will return
    /// `None`.
    pub fn dvar_alignment_exp(&self) -> Option<u8> {
        self.flags_at(self.idx)
            .and_then(|v| Some(v.dvar_alignment_exp()))
    }

    /// Return the GVar alignment exponent at the current index.
    ///
    /// The returned value is the exponent to a power of 2 describing the
    /// minimum alignment requirements of the type at the current position.
    ///
    /// If the current index is at the end of the signature, this will return
    /// `None`.
    pub fn gvar_alignment_exp(&self) -> Option<u8> {
        self.flags_at(self.idx)
            .and_then(|v| Some(v.gvar_alignment_exp()))
    }
}

/// Create a D-Bus Signature literal from its string representation.
///
/// Internally, this calls [`Sig::make()`] but ensures the data buffer is
/// choosen sufficiently for the given signature.
#[doc(hidden)]
#[macro_export]
macro_rules!
    crate_fmt_dbus_signature_make
{ ($signature:expr) => {
    const {
        use $crate::fmt::dbus::signature::Sig;
        const N: usize = Sig::size_for_length($signature.len());
        Sig::<[u64; N]>::make($signature)
    }
}}

#[doc(inline)]
pub use crate_fmt_dbus_signature_make as make;

#[doc(hidden)]
#[macro_export]
macro_rules!
    crate_fmt_dbus_signature_sig
{ ($signature:literal) => {
    const {
        use $crate::fmt::dbus::signature::Sig;
        const N: usize = Sig::size_for_length($signature.len());
        &Sig::<[u64; N]>::make($signature) as &'static Sig
    }
}}

#[doc(inline)]
pub use crate_fmt_dbus_signature_sig as sig;

#[allow(non_upper_case_globals)]
pub mod builtin {
    pub const s: &super::Sig = &super::make!(b"s");
    pub const asv: &super::Sig = &super::make!(b"a{sv}");
}

impl core::clone::Clone for boxed::Box::<Sig> {
    fn clone(&self) -> Self {
        Sig::clone(self)
    }
}

impl core::fmt::Debug for Sig {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        for v in self {
            fmt.write_str(v.str())?;
        }
        Ok(())
    }
}

impl core::fmt::Display for Sig {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        self.into_iter().try_for_each(
            |v| core::fmt::Write::write_char(fmt, v.char()),
        )
    }
}

impl core::cmp::Eq for Sig {
}

impl core::hash::Hash for Sig {
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        // D-Bus Signatures are a prefix-code, so no prefix collissions can
        // happen (no signature is ever a prefix of another).
        self.into_iter().for_each(|v| v.hash(state))
    }
}

impl core::cmp::Ord for Sig {
    fn cmp(&self, v: &Self) -> core::cmp::Ordering {
        // Sort by code to ensure stable order.
        self.into_iter()
            .map(|v| v.code())
            .cmp(
                v.into_iter().map(|v| v.code()),
            )
    }
}

impl core::cmp::PartialEq for Sig {
    fn eq(&self, v: &Self) -> bool {
        self.into_iter().eq(v)
    }
}

impl core::cmp::PartialOrd for Sig {
    fn partial_cmp(&self, v: &Self) -> Option<core::cmp::Ordering> {
        // Sort by code to ensure stable order.
        self.into_iter()
            .map(|v| v.code())
            .partial_cmp(
                v.into_iter().map(|v| v.code()),
            )
    }
}

impl<'sig> core::iter::IntoIterator for &'sig Sig {
    type Item = dbus::Element;
    type IntoIter = Cursor<'sig>;

    fn into_iter(self) -> Self::IntoIter {
        self.cursor()
    }
}

impl borrow::ToOwned for Sig {
    type Owned = boxed::Box::<Sig>;

    fn to_owned(&self) -> Self::Owned {
        self.clone()
    }
}

impl<'sig, Owned> core::iter::Iterator for Cursor<'sig, Owned>
where
    Owned: borrow::Borrow<Sig>,
{
    type Item = dbus::Element;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.element();
        self.move_next();
        v
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.root().len() - self.idx;
        (n, Some(n))
    }
}

impl<'sig, Owned> core::iter::ExactSizeIterator for Cursor<'sig, Owned>
where
    Owned: borrow::Borrow<Sig>,
{
}

impl<'sig, Owned> core::iter::FusedIterator for Cursor<'sig, Owned>
where
    Owned: borrow::Borrow<Sig>,
{
}

#[cfg(test)]
mod test {
    use super::*;

    // Verify basic API behavior.
    #[test]
    fn api() {
        let t0 = Sig::new(b"a{sv}").unwrap();
        assert_eq!(t0.to_string(), "a{sv}");
        let t1 = Sig::arc(b"a{sv}").unwrap();
        assert_eq!(t1.to_string(), "a{sv}");
    }

    // Verify a set of common signatures.
    #[test]
    fn sig_common() {
        let types: [&[u8]; _] = [
            // primitives
            b"y",
            b"q",
            b"n",
            b"u",
            b"i",
            b"t",
            b"x",
            b"d",
            b"b",
            b"s",
            b"o",
            b"g",
            b"h",
            b"v",

            // bound containers
            b"ay",
            b"my",

            // unbound containers
            b"(yy)",
            b"{sv}",

            // compound types
            b"a{sv}",
            b"(uuta{sv}a{sv})",
            b"a(uu(t)(t))",
            b"({uu}{tt})",
            b"amamy",
            b"(ayata{yy}a(vv)v)",
        ];

        for t in types {
            let v = Sig::new(t).unwrap();
            assert_eq!(v.to_string().as_bytes(), t);
        }
    }

    // Verify a set of invalid signatures.
    #[test]
    fn sig_invalid() {
        let types: [&[u8]; _] = [
            b"",

            // SignatureIncomplete
            b"a",
            b"m",
            b"(",
            b"{",
            b"aa",
            b"(tt",
            b"(tt(ay)",

            // SignatureSequence
            b"tt",
            b"ayay",
            b"()()",
            b"()()()()",

            // ElementInvalid
            b"\xff",

            // ElementUnknown
            b"\0",
            b"c",

            // ElementUnpaired
            b")",
            b"}",
            b"a)",
            b"a}",

            // ElementMispaired
            b"(}",
            b"{)",
            b"{sv)",
            b"(sv}",

            // DictInvalid
            b"{}",
            b"{t}",
            b"{ttt}",
            b"{vv}",
            b"{(t)t}",
        ];

        for t in types {
            assert!(Sig::new(t).is_err());
        }
    }

    #[test]
    fn subslicing() {
        let t = sig!(b"a{sv}");
        let mut c = t.cursor();

        assert_eq!(c.root().to_string(), "a{sv}");
        assert_eq!(c.sig().unwrap().to_string(), "a{sv}");
        c.move_down();
        assert_eq!(c.sig().unwrap().to_string(), "{sv}");
        c.move_down();
        assert_eq!(c.sig().unwrap().to_string(), "s");
        c.move_step();
        assert_eq!(c.sig().unwrap().to_string(), "v");
        c.move_step();
        assert_eq!(c.sig(), None);
        c.move_up();
        assert_eq!(c.sig().unwrap().to_string(), "{sv}");
        c.move_up();
        assert_eq!(c.sig().unwrap().to_string(), "a{sv}");
        c.move_step();
        assert_eq!(c.sig(), None);
    }

    // Verify the signature cursor movement works as intended.
    #[test]
    fn cursor_movement() {
        // Basic cursor movement on a primitive type. Verify next/prev is
        // clamped to the extents of the type. Then verify the same for a
        // non-primitive type.
        {
            let t = sig!(b"s");
            let mut c = t.cursor();

            assert_eq!(c.element(), Some(dbus::Element::String));
            assert_eq!(c.move_next().element(), None);
            assert_eq!(c.move_next().element(), None);
            assert_eq!(c.move_prev().element(), Some(dbus::Element::String));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::String));
            assert_eq!(c.move_next().element(), None);

            let t = sig!(b"(tui)");
            let mut c = t.cursor();

            assert_eq!(c.element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_next().element(), Some(dbus::Element::U64));
            assert_eq!(c.move_next().element(), Some(dbus::Element::U32));
            assert_eq!(c.move_next().element(), Some(dbus::Element::I32));
            assert_eq!(c.move_next().element(), Some(dbus::Element::StructClose));
            assert_eq!(c.move_next().element(), None);
            assert_eq!(c.move_next().element(), None);
            assert_eq!(c.move_prev().element(), Some(dbus::Element::StructClose));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::I32));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::U32));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::U64));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_prev().element(), Some(dbus::Element::StructOpen));
        }

        // Verify movement up/down/step in a nested type. Unlike next/prev this
        // will always take steps based on entire types. Moving into
        // non-container types and moving out of uncontained types is a no-op.
        {
            let t = sig!(b"a(a{sv}()()aay)");
            let mut c = t.cursor();

            assert_eq!(c.element(), Some(dbus::Element::Array));
            assert_eq!(c.move_down().element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_down().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_down().element(), Some(dbus::Element::DictOpen));
            assert_eq!(c.move_down().element(), Some(dbus::Element::String));
            // Moving down into primitives is a no-op:
            assert_eq!(c.move_down().element(), Some(dbus::Element::String));
            assert_eq!(c.move_step().element(), Some(dbus::Element::Variant));
            assert_eq!(c.move_step().element(), Some(dbus::Element::DictClose));
            assert_eq!(c.move_up().element(), Some(dbus::Element::DictOpen));
            // Stepping the member of an array is a no-op:
            assert_eq!(c.move_step().element(), Some(dbus::Element::DictOpen));
            assert_eq!(c.move_up().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_step().element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_step().element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_step().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_down().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_down().element(), Some(dbus::Element::U8));
            assert_eq!(c.move_up().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_up().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_step().element(), Some(dbus::Element::StructClose));
            assert_eq!(c.move_up().element(), Some(dbus::Element::StructOpen));
            assert_eq!(c.move_up().element(), Some(dbus::Element::Array));
            // Moving up when uncontained is a no-op:
            assert_eq!(c.move_up().element(), Some(dbus::Element::Array));
            assert_eq!(c.move_step().element(), None);
        }
    }

    // Verify trait implementations, if possible.
    #[test]
    fn traits() {
        let t0 = Sig::new(b"(tt)").unwrap();
        let t1 = Sig::new(b"(tv)").unwrap();

        assert_eq!(std::format!("{:?}", t0), "(tt)");
        assert_eq!(std::format!("{}", t0), "(tt)");

        assert!(t0 == t0);
        assert!(t0 <= t0);
        assert!(t0 < t1);
    }
}
