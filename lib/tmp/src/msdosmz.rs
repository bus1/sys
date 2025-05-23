//! MS-DOS MZ Executable Format
//!
//! The MS-DOS-MZ-Executable format was introduced with Microsoft DOS-2.0 and
//! replaced the plain COM-format. It defines how an executable is to be loaded
//! into memory, its required relocations, and the initial register state when
//! starting execution. Its name was derived from the signature used by the
//! format.
//!
//! The format is rarely used today. Its most common use is as part of stub
//! programs embedded in its successor formats like PE (Portable Executable).
//!
//! Applications linked in the original MS-DOS format are often called "DOS COM
//! Programs", while applications linked with the format described here are
//! commonly referred to as "DOS EXE Programs".
//!
//! The format is highly tied to how programs were executed on old x86 machines
//! running the DOS operating system. Memory segmentation in particular plays
//! an important role in many of the design decisions. A full understanding of
//! the MS-DOS MZ Executable Format requires at least basic understanding of
//! how MS-DOS works. Followingly, a list of notes that might be of value to
//! understand the intricacies of MS-DOS executables:
//!
//!  * Memory was organized in 64KiB segments, but segments are overlapping and
//!    offset by 16 bytes from each other. Hence, the total addressable memory
//!    is 1MiB (actually based on the 20-bit address-bus used in old x86
//!    hardware).
//!    Due to architectual decisions, only the lower 640KiB were allocated to
//!    the application, while the upper memory was reserved for video hardware
//!    or other hardware management.
//!    Newer hardware supported all kinds of extensions to provide access to
//!    memory beyond 1MiB, yet those extensions have little effect on the
//!    design decisions of this format and thus are mostly ignored here.
//!
//!  * Original DOS COM applications had a hard-coded entry-point at 0x100 and
//!    only supported addressing a single segment (64KiB). The new DOS EXE
//!    applications support applications spanning multiple segments, as well as
//!    any entry-point.
//!    The initial offset of 0x100 was due to the Program Segment Prefix (PSP)
//!    being placed in front of the application. This structure contained
//!    information for the application, including its environment and
//!    command-line arguments.
//!
//!  * While MS-DOS had different styles of programs running in parallel to
//!    serve hardware interrupts and manage the system, it effectively was a
//!    single-tasking OS that only ran a single application at once. However,
//!    it is common for programs to load other programs and resume execution
//!    once this program returned. Thus, multiple programs will be loaded in
//!    memory at a time.
//!
//!  * On application startup, the loader calculates the size of the program to
//!    load into memory, as well as the minimum required size of additional
//!    memory beyond that, as noted in the file header. It then tries to find
//!    the biggest block of memory that fulfils that need, up to a maximum as
//!    specified in the file header again. This allows applications to specify
//!    a minimum required space beyond the code stored in the file, as well as
//!    ask for additional optional memory serving as heap memory.
//!
//!  * A new application is loaded into memory with its PSP directly in front
//!    of it. The start-segment is the first segment after the PSP, and thus
//!    the first segment holding the application. The start-segment is used to
//!    relocate DOS EXE executables, and allows executables to be loaded at any
//!    segment, while still spanning multiple segments.
//!
//!  * Old DOS executables assumed that any higher segment than their start
//!    segment is also available to them, and thus assumed anything in the
//!    range [CS:0x0000; 0xa000:0x0000] (i.e., anything from the start-segment
//!    up to the highest segment at 640KiB) is available to them. This is not
//!    strictly true, though, since high memory can be reserved for other
//!    reasons. Hence, an application can query the PSP for the first segment
//!    beyond the area allocated to the application to get an authoritative
//!    answer.
//!
//!  * While dealing with old MS-DOS formats, there are 3 important sizes used
//!    to measure memory areas:
//!
//!     * `WORD`: A word is a 16-bit integer. `DWORD` and `QWORD` refer to
//!       double and quadruple of that size.
//!
//!     * `PARAGRAPH`: A paragraph are 16 consecutive bytes. This is most often
//!       used to allocate blocks of memory, or perform operations on large
//!       blocks. The size reflects the offset between segments on x86.
//!
//!     * `PAGE`: A page is 512 consecutive bytes. This is not to be confused
//!       with page-sizes on modern x86 hardware (often 4k).
//!
//!  * All multi-byte integers are encoded as little-endian.
//!
//! The file-format is rather trivial. It consists of a static header, which
//! embeds its own size and thus allows for applications to store additional
//! private data trailing the static header. Additionally, the file contains a
//! relocation table with all address-relocations that are to be applied to the
//! application code before execution. This table is usually placed directly
//! after the header, including its size in the total header size.
//!
//! Anything beyond the header is considered application code, up to the total
//! size as specified in the header. Anything trailing that is not considered
//! part of the executable and ignored.
//!
//! There is a Microsoft header extension which is used to designate the file
//! as a DOS-compatibility executable. This header must directly follow the
//! static header, and the relocation-offset usually indicates that the
//! relocation table follows this extended header, thus showing that such an
//! extended header is likely present. The extended header does not affect the
//! executable, but contains metadata describing other data stored after the
//! code (which is ignored by the DOS EXE loader). This technique is usually
//! used to combine a DOS EXE executable with a more modern PE executable into
//! a single file.

use core::mem::{
    align_of,
    align_of_val,
    size_of,
    size_of_val,
};

type U16Le = osi::ffi::Integer<osi::ffi::LittleEndian<u16>, osi::align::AlignAs<2>>;
type U32Le = osi::ffi::Integer<osi::ffi::LittleEndian<u32>, osi::align::AlignAs<4>>;

/// Size of a Word
///
/// The term `WORD` is used to denote 2-byte integers in MS-DOS software, and
/// many derivatives. It reflects the size of the data-bus and registers of the
/// originally supported x86 machines.
pub const WORD_SIZE: usize = 2;

/// Size of a Paragraph
///
/// The term `PARAGRAPH` is used to denote 16-byte memory regions in MS-DOS
/// software and some derivatives. It reflects the offset of two consecutive
/// segments on x86 machines in Real Mode. Note that segments are overlapping,
/// thus a paragraph describes their relative distance but not their size.
pub const PARAGRAPH_SIZE: usize = 16;

/// Size of a Page
///
/// The term `PAGE` usually denotes 512-byte memory regions in MS-DOS software.
/// It reflects the size of hardware pages used to store memory on disk. Hence,
/// for better performance, in-memory data is often also organized in pages to
/// allow easy mapping to storage pages.
pub const PAGE_SIZE: usize = 512;

/// Magic Signature
///
/// The initial 2 bytes of every DOS-MZ file are set to `0x4d` followed by
/// `0x5a` ("MZ"). They are used to identify the format. They reflect the
/// initials of an early Microsoft employee who worked on the format.
pub const MAGIC: [u8; 2] = [0x4d, 0x5a];

/// Calculate 16-bit Sum
///
/// This function splits a byte slice into consecutive 16-bit unsigned integers
/// and calculates their sum. Little endianness is assumed. If the slice is of
/// an odd length, a missing trailing zero byte is assumed.
///
/// The sum is calculated in wrapping-mode, meaning overflows are handled
/// gracefully.
///
/// This function is tailored for checksum calculations, which commonly are
/// based on the 16-bit sum of a byte slice.
pub fn sum16(data: &[u8]) -> u16 {
    data.chunks(2).fold(
        0u16,
        |acc, values| {
            acc.wrapping_add(
                match values.len() {
                    1 => { values[0] as u16 },
                    2 => { u16::from_le_bytes(values.try_into().unwrap()) },
                    _ => { unreachable!(); },
                }
            )
        },
    )
}

/// File Header
///
/// This static structure is located at offset 0 of a DOS MZ executable. It
/// has a fixed size of 28 bytes and describes the further layout of the file.
#[repr(C)]
pub struct Header {
    /// The static signature identifying the file-format. This must match
    /// `MAGIC`. (abbr: magic-identifier).
    pub magic: [u8; 2],

    /// The number of bytes in the last page of the program (abbr:
    /// count-bytes-last-page)
    ///
    /// While `cp` describes the number of pages that make up this executable,
    /// this `cblp` field describes the number of valid bytes in the last page.
    /// The special value of `0` denotes the entire last page as part of the
    /// executable.
    ///
    /// Any other data beyond this offset is ignored. It is valid to append
    /// data for other reasons to the end of the file.
    pub cblp: U16Le,

    /// The number of pages of this program (abbr: count-pages)
    ///
    /// This is the absolute number of pages starting from the beginning of
    /// the header that make up this executable. Any data beyond this is
    /// ignored. The `cblp` field describes how many bytes of the last page
    /// are actually part of the program.
    ///
    /// E.g., A `cp` of 2 with `cblp` of 16 means `(2-1)*512+16=528` bytes. A
    /// `cp` of 3 with `cblp` of 0 means `3*512=1536` bytes.
    pub cp: U16Le,

    /// The number of relocations in the relocation header (abbr:
    /// count-re-lo-cations)
    ///
    /// In combination with `lfarlc` this describes the location and size of
    /// the relocation table. The `crlc` field is the number of relocation
    /// entries in this table. See `lfarlc` for details.
    pub crlc: U16Le,

    /// The number of paragraphs that make up the header (abbr:
    /// count-paragraphs-header).
    pub cparhdr: U16Le,

    /// The minimum number of additional paragraphs required for program
    /// execution (abbr: minimum-allocation).
    ///
    /// When a program is loaded into memory, its program size is the file
    /// size as specified by `cp` and `cblp` minus the size of the header
    /// as specified by `cparhdr`. This size is then extended by `minalloc`
    /// to calculate the minimum amount of memory required to execute the
    /// program.
    ///
    /// This technique allows linkers to strip uninitialized static
    /// variables from the program code, and thus reduce the size of the
    /// file. The variables are then located in the allocated space
    /// trailing the program code, and thus still available at runtime.
    pub minalloc: U16Le,

    /// The maximum number of additional paragraphs required for program
    /// execution (abbr: maximum-allocation).
    ///
    /// While `minalloc` is a minimum requirement to allow execution of a
    /// program, `maxalloc` specifies how much more memory the application
    /// would like allocated. The OS is free to allocate more or less. The
    /// PSP contains information about how much was actually allocated.
    ///
    /// `maxalloc` cannot be less than `minalloc`, or it will be ignored.
    ///
    /// A value of `0xffff` simply means to allocate as much memory as
    /// possible. The application is still free to use MS-DOS software
    /// interrupts to deallocate memory again.
    pub maxalloc: U16Le,

    /// The initial stack-segment offset to use (abbr: stack-segment).
    ///
    /// The stack-segment register is set to the same value as the code-segment
    /// register at program start. The latter is set to the first segment of
    /// the program loaded in memory. In front of this first segment is usually
    /// the PSP (the `DS` and `ES` registers contain the segment of the PSP at
    /// program start, to avoid relying on this).
    ///
    /// This `ss` value is an offset added to the stack-segment register at
    /// program start, and thus allows moving the stack into later segments.
    pub ss: U16Le,

    /// The initial stack pointer to use (abbr: stack-pointer).
    ///
    /// This specifies the initial value of the `sp` register. In combination
    /// with `ss` it defines the start of the stack. In most cases, this value
    /// is set to `0`, since this will cause the first stack push to overflow
    /// the value and thus push at the end of the stack segment.
    pub sp: U16Le,

    /// The checksum of the file (abbr: check-sum).
    ///
    /// The checksum of the file is the one's complement of the summation of
    /// all words of the file (with `csum` set to 0). Note that this only
    /// applies to the data as specified by `cp` and `cblp`. Any trailing data
    /// is ignored. If `cblp` is odd, the last byte trailing it must be assumed
    /// to be 0, even if other data is trailing it.
    ///
    /// The checksum is calculated as little-endian sum of all 16-bit integers
    /// that make up the file.
    pub csum: U16Le,

    /// The initial instruction pointer to use (abbr:
    /// instruction-pointer).
    ///
    /// This is the initial value of the instruction pointer register `ip`.
    /// Since it is relative to the code-segment register `cs`, this value is
    /// taken verbatim by the loader.
    pub ip: U16Le,

    /// The initial code segment offset to use (abbr: code-segment).
    ///
    /// Similar to the stack-segment register `ss`, this value defines the
    /// offset to apply to the code-segment register `cs` before executing the
    /// code. The actual value before the offset is applied is the first
    /// segment the program was loaded into.
    pub cs: U16Le,

    /// The absolute offset to the relocation table (abbr:
    /// logical-file-address-re-lo-cation).
    ///
    /// This is the offset of the relocation table relative to the start of the
    /// header (and thus usually the start of the file). The relocation table
    /// is an array of relocations (see `Relocation`). The number of entries is
    /// specified in `crlc`.
    ///
    /// The relocation table is usually trailing the static header and included
    /// in the size of the header. However, an application is free to place the
    /// table at any other offset.
    ///
    /// The size of the static header plus the extension header is `0x40`,
    /// hence a relocation table offset of `0x40` usually designates the
    /// existance of an extension header (yet this is not a requirement). The
    /// extension header still needs its own signature verification to ensure
    /// its validity.
    pub lfarlc: U16Le,

    /// The overlay number (abbr: overlay-number).
    ///
    /// This is `0` if the file is not an overlay. Otherwise, this specifies
    /// the overlay index to assign.
    ///
    /// Overlays are used to reserve programs but avoid loading them into
    /// memory. See the MS-DOS overlay support for details. For any regular
    /// application, this is set to `0`.
    pub ovno: U16Le,
}

/// Extended Header
///
/// The extended header optionally follows the static header without any
/// padding. The presence of an extended header is suggested by the relocation
/// offset being beyond the extended header, as well as the header size being
/// big enough to include the extended header.
///
/// The only meaningful field of the extended header is `lfanew`, which is a
/// 32-bit offset into the file where further header information can be found.
/// Depending on the format that uses this extended header, a different
/// signature can be found at that offset.
///
/// The other fields of this extended header are very scarcely documented and
/// thus usually set to 0.
#[repr(C)]
pub struct HeaderExt {
    /// Reserved field which must be cleared to 0, yet must not be relied on
    /// to be 0.
    pub res: [u8; 8],

    /// OEM ID, usually cleared to 0.
    pub oemid: U16Le,

    /// OEM Information, usually cleared to 0.
    pub oeminfo: U16Le,

    /// Reserved field which must be cleared to 0, yet must not be relied on
    /// to be 0.
    pub res2: [u8; 20],

    /// File offset of the new file format (abbr: logical-file-address-new)
    ///
    /// This contains an offset into the file relative to the start of the
    /// static header where to find a newer format of this file. No further
    /// information can be deduced from this.
    ///
    /// Any format using this must place its own signature at the specified
    /// offset and thus allow separate verification of its validity. In
    /// particular, Portable Executable (PE) files will place "PE\0\0" at
    /// this offset to denote a PE/COFF header.
    pub lfanew: U32Le,
}

/// Relocation Information
///
/// This structure describes an entry of the relocation table. Each entry
/// points into the program code, at a 2-byte value that must be adjusted with
/// the start-segment before the program is run. The value of the start-segment
/// is simply added to each location pointed at by the relocation table.
///
/// A single location is described by its segment relative to the start of the
/// program, as well as the offset inside that segment.
#[repr(C)]
pub struct Relocation {
    /// Offset of the relocation target relative to the specified segment.
    pub offset: U16Le,

    /// Segment of the relocation target relative to the start of the code.
    pub segment: U16Le,
}

impl Header {
    /// Import a header from a byte slice
    ///
    /// Create a new header structure from a byte slice, copying the data over.
    /// The data is copied verbatim without any conversion.
    pub fn from_bytes(data: &[u8; 28]) -> Self {
        let mut uninit: core::mem::MaybeUninit<Self> = core::mem::MaybeUninit::uninit();

        assert!(align_of_val(data) <= align_of::<Self>());
        assert!(size_of_val(data) == size_of::<Self>());

        unsafe {
            // Safety: The entire struct consists of unsigned integers and
            //         arrays of unsigned integers, which all have no invalid
            //         byte-level representations and thus can be imported
            //         directly. Even the wrong endianness is still a valid
            //         value.
            core::ptr::write(uninit.as_mut_ptr() as *mut [u8; 28], *data);
            uninit.assume_init()
        }
    }

    /// Convert to byte slice
    ///
    /// Return a byte-slice reference to the header. This can be used to export
    /// the structure into a file. No byte-order conversions are applied.
    pub fn as_bytes(&self) -> &[u8; 28] {
        assert!(align_of::<[u8; 28]>() <= align_of::<Self>());
        assert!(size_of::<[u8; 28]>() == size_of::<Self>());

        unsafe {
            core::mem::transmute::<&Self, &[u8; 28]>(self)
        }
    }
}

impl HeaderExt {
    /// Import a header extension from a byte slice
    ///
    /// Create a new header extension structure from data copied from a byte
    /// slice. No byte-order conversions are applied.
    pub fn from_bytes(data: &[u8; 36]) -> Self {
        let mut uninit: core::mem::MaybeUninit<Self> = core::mem::MaybeUninit::uninit();

        assert!(align_of_val(data) <= align_of::<Self>());
        assert!(size_of_val(data) == size_of::<Self>());

        unsafe {
            core::ptr::write(uninit.as_mut_ptr() as *mut [u8; 36], *data);
            uninit.assume_init()
        }
    }

    /// Convert to byte slice
    ///
    /// Return a byte-slice reference to the header extension. This can be used
    /// to export the structure into a file. No byte-order conversions are
    /// applied.
    pub fn as_bytes(&self) -> &[u8; 36] {
        assert!(align_of::<[u8; 36]>() <= align_of::<Self>());
        assert!(size_of::<[u8; 36]>() == size_of::<Self>());

        unsafe {
            core::mem::transmute::<&Self, &[u8; 36]>(self)
        }
    }
}

impl Relocation {
    /// Import a relocation entry from a byte slice
    ///
    /// Create a new relocation structure from data copied from a byte
    /// slice. No byte-order conversions are applied.
    pub fn from_bytes(data: &[u8; 4]) -> Self {
        let mut uninit: core::mem::MaybeUninit<Self> = core::mem::MaybeUninit::uninit();

        assert!(align_of_val(data) <= align_of::<Self>());
        assert!(size_of_val(data) == size_of::<Self>());

        unsafe {
            core::ptr::write(uninit.as_mut_ptr() as *mut [u8; 4], *data);
            uninit.assume_init()
        }
    }

    /// Convert to byte slice
    ///
    /// Return a byte-slice reference to the relocation entry. This can be used
    /// to export the structure into a file. No byte-order conversions are
    /// applied.
    pub fn as_bytes(&self) -> &[u8; 4] {
        assert!(align_of::<[u8; 4]>() <= align_of::<Self>());
        assert!(size_of::<[u8; 4]>() == size_of::<Self>());

        unsafe {
            core::mem::transmute::<&Self, &[u8; 4]>(self)
        }
    }
}

/// X86 Stub Program
///
/// This array contains a full MS-DOS EXE program that prints the following
/// line on startup and then exits with an error code of 1:
///
///   "This program cannot be run in DOS mode.\r\r\n"
///
/// A stub program like this is typically used with extended file-formats like
/// PE/COFF.
///
/// This stub is a fully functioning DOS program of size 128 bytes. It contains
/// an extended DOS header with the `lfanew` offset set to 128 (directly after
/// this stub). Hence, you can prepend this 128-byte stub to any PE program
/// without any modifications required. If required, the `lfanew` offset can
/// be adjusted after copying it.
pub const STUB_X86: [u8; 128] = [
    // Header:
    0x4d, 0x5a, //              MAGIC
    0x80, 0x00, //              cblp: 128
    0x01, 0x00, //              cp: 1
    0x00, 0x00, //              crlc: 0 (no relocations)
    0x04, 0x00, //              cparhdr: 4 (64 bytes; header+ext)
    0x00, 0x00, //              minalloc: 0
    0xff, 0xff, //              maxalloc: 0xffff
    0x00, 0x00, //              ss: 0
    0x80, 0x00, //              sp: 0x80 (128; 64 code + 64 stack)
    0x68, 0xa7, //              csum: 0xa768
    0x00, 0x00, //              ip: 0
    0x00, 0x00, //              cs: 0
    0x40, 0x00, //              lfarlc: 0x40 (64; directly after the ext-header)
    0x00, 0x00, //              ovno: 0

    // HeaderExt:
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, //              oemid: 0
    0x00, 0x00, //              oeminfo: 0
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x00, 0x00, 0x00, 0x00, //  reserved
    0x80, 0x00, 0x00, 0x00, //  lfanew: 0x80 (128; directly after this stub)

    // Program:

    0x0e, //                    push cs         (save CS)
    0x1f, //                    pop ds          (set DS=CS)
    0xba, 0x0e, 0x00, //        mov dx,0xe      (point to string)
    0xb4, 0x09, //              mov ah,0x9      (number of "print"-syscall)
    0xcd, 0x21, //              int 0x21        (invoke syscall)
    0xb8, 0x01, 0x4c, //        mov ax,0x4c01   (number of "exit-1"-syscall)
    0xcd, 0x21, //              int 0x21        (invoke syscall)

    //                          "This program cannot be run in DOS mode."
    //                          "\r\r\n$"
    0x54, 0x68, 0x69, 0x73, 0x20, 0x70, 0x72, 0x6f,
    0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e,
    0x6e, 0x6f, 0x74, 0x20, 0x62, 0x65, 0x20, 0x72,
    0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f,
    0x53, 0x20, 0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d,
    0x0d, 0x0a, 0x24,

    // Alignment to 8-byte boundary and 128-byte total.
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[cfg(test)]
mod tests {
    use super::*;

    // Verify alignment and size of our protocol types match the values
    // provided by the specification.
    #[test]
    fn verify_types() {
        assert_eq!(size_of::<Header>(), 28);
        assert_eq!(align_of::<Header>(), 2);

        assert_eq!(size_of::<HeaderExt>(), 36);
        assert_eq!(align_of::<HeaderExt>(), 4);

        assert_eq!(size_of::<Header>() + size_of::<HeaderExt>(), 64);

        assert_eq!(size_of::<Relocation>(), 4);
        assert_eq!(align_of::<Relocation>(), 2);
    }

    // Basic test for the Header API.
    #[test]
    fn verify_header() {
        let h = Header::from_bytes((&STUB_X86[..28]).try_into().unwrap());

        assert_eq!(h.magic, MAGIC);

        assert_eq!(h.as_bytes(), &STUB_X86[..28]);
    }

    // Basic test for the HeaderExt API.
    #[test]
    fn verify_headerext() {
        let e = HeaderExt::from_bytes((&STUB_X86[28..64]).try_into().unwrap());

        assert_eq!(e.lfanew.to_native(), 0x0080);

        assert_eq!(e.as_bytes(), &STUB_X86[28..64]);
    }

    // Basic test for the Relocation API.
    #[test]
    fn verify_relocation() {
        let r_slice: [u8; 4] = [0x10, 0x00, 0x20, 0x00];
        let r = Relocation::from_bytes(&r_slice);

        assert_eq!(r.offset.to_native(), 0x0010);
        assert_eq!(r.segment.to_native(), 0x0020);

        assert_eq!(r.as_bytes(), &r_slice);
    }

    // Test the `sum16()` helper, including overflow checks, endianness
    // verification, and correct slice splitting.
    #[test]
    fn verify_sum16() {
        // Sum up 0+1+2+3.
        let data = [0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00];
        assert_eq!(sum16(&data), 6);

        // Sum up 0+1+2+3 with a missing trailing byte.
        let data = [0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03];
        assert_eq!(sum16(&data), 6);

        // Verify high values are correctly added.
        let data = [0x01, 0x02, 0x02, 0x04];
        assert_eq!(sum16(&data), 0x0603);

        // Verify an overflow is calculated correctly.
        let data = [0xff, 0xff, 0x02, 0x00];
        assert_eq!(sum16(&data), 1);
    }

    // Verify the contents of the x86-stub and make sure the decoder produces
    // the expected values.
    #[test]
    fn verify_stub_x86() {
        let stub = &STUB_X86;
        let h = Header::from_bytes((&stub[..28]).try_into().unwrap());
        let e = HeaderExt::from_bytes((&stub[28..64]).try_into().unwrap());

        // Verify expected header values.
        assert_eq!(h.magic, MAGIC);
        assert_eq!(h.cblp.to_native(), 128);
        assert_eq!(h.cp.to_native(), 1);
        assert_eq!(h.crlc.to_native(), 0);
        assert_eq!(h.cparhdr.to_native(), 4);
        assert_eq!(h.minalloc.to_native(), 0x0000);
        assert_eq!(h.maxalloc.to_native(), 0xffff);
        assert_eq!(h.ss.to_native(), 0x00);
        assert_eq!(h.sp.to_native(), 0x80);
        assert_eq!(h.csum.to_native(), 0xa768);
        assert_eq!(h.ip.to_native(), 0);
        assert_eq!(h.cs.to_native(), 0);
        assert_eq!(h.lfarlc.to_native(), 0x40);
        assert_eq!(h.ovno.to_native(), 0x00);

        // Verify expected extended header.
        assert_eq!(e.res, [0; 8]);
        assert_eq!(e.oemid.to_native(), 0x00);
        assert_eq!(e.oeminfo.to_native(), 0x00);
        assert_eq!(e.res2, [0; 20]);
        assert_eq!(e.lfanew.to_native(), 0x0080);

        // Verify the checksum-field is correct.
        assert_eq!(!sum16(stub), 0);
    }
}
