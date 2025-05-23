//! Portable Executable / Common Object File Format
//!
//! The Portable Executable format (PE, PE/COFF, PE32, PE32+) encodes
//! executable and library code on platforms like Microsoft Windows, UEFI, and
//! others. The COFF format originates in UNIX but was later adopted and
//! extended into PE by Microsoft. Apart from the latter, the format has been
//! widely abandonded and replaced by the Executable and Linker Format (ELF).
//!
//! The COFF format is still used for object files and linker input. The PE
//! format extends COFF slightly to allow MS-DOS stubs to be linked in front
//! of the COFF executable. The name Portable Executable refers to the fact
//! that the format is meant to be architecture independent.
//!
//! The format was further extended to support offsets larger than 32bit,
//! paving the way for adoption on 64 bit machines. Files encoded in the
//! extended format are called PE32+.
//!
//! XXX: This module is still incomplete.

type U8Le = osi::ffi::Integer<osi::ffi::LittleEndian<u8>, osi::align::AlignAs<1>>;
type U16Le = osi::ffi::Integer<osi::ffi::LittleEndian<u16>, osi::align::AlignAs<2>>;
type U32Le = osi::ffi::Integer<osi::ffi::LittleEndian<u32>, osi::align::AlignAs<4>>;
type U64Le = osi::ffi::Integer<osi::ffi::LittleEndian<u64>, osi::align::AlignAs<8>>;

pub const INVALID_TIMESTAMPS: [u32; 2] = [0x00000000, 0xffffffff];

pub const PE_MAGIC: [u8; 4] = [0x50, 0x45, 0x00, 0x00];
pub const PE_MAGIC_OH32: u16 = 0x010b;
pub const PE_MAGIC_OH32P: u16 = 0x010b;

pub const PE_OFFSET: usize = 0x3c;

pub const CHARACTERISTIC_RELOCS_STRIPPED: u16 = 0x0001;
pub const CHARACTERISTIC_EXECUTABLE_IMAGE: u16 = 0x0002;
pub const CHARACTERISTIC_LINE_NUMS_STRIPPED: u16 = 0x0004;
pub const CHARACTERISTIC_LOCAL_SYMS_STRIPPED: u16 = 0x0008;
pub const CHARACTERISTIC_AGGRESSIVE_WS_TRIM: u16 = 0x0010;
pub const CHARACTERISTIC_LARGE_ADDRESS_AWARE: u16 = 0x0020;
/* reserved: 0x0040 */
pub const CHARACTERISTIC_BYTES_REVERSED_LO: u16 = 0x0080;
pub const CHARACTERISTIC_32BIT_MACHINE: u16 = 0x0100;
pub const CHARACTERISTIC_DEBUG_STRIPPED: u16 = 0x0200;
pub const CHARACTERISTIC_REMOVABLE_RUN_FROM_SWAP: u16 = 0x0400;
pub const CHARACTERISTIC_NET_RUN_FROM_SWAP: u16 = 0x0800;
pub const CHARACTERISTIC_SYSTEM: u16 = 0x1000;
pub const CHARACTERISTIC_DLL: u16 = 0x2000;
pub const CHARACTERISTIC_UP_SYSTEM_ONLY: u16 = 0x4000;
pub const CHARACTERISTIC_BYTES_REVERSED_HI: u16 = 0x8000;

pub const DATA_DIRECTORY_EXPORT_TABLE: u16 = 0;
pub const DATA_DIRECTORY_IMPORT_TABLE: u16 = 1;
pub const DATA_DIRECTORY_RESOURCE_TABLE: u16 = 2;
pub const DATA_DIRECTORY_EXCEPTION_TABLE: u16 = 3;
pub const DATA_DIRECTORY_CERTIFICATE_TABLE: u16 = 4;
pub const DATA_DIRECTORY_BASE_RELOCATION_TABLE: u16 = 5;
pub const DATA_DIRECTORY_DEBUG: u16 = 6;
pub const DATA_DIRECTORY_ARCHITECTURE: u16 = 7;
pub const DATA_DIRECTORY_GLOBAL_PTR: u16 = 8;
pub const DATA_DIRECTORY_TLS_TABLE: u16 = 9;
pub const DATA_DIRECTORY_LOAD_CONFIG_TABLE: u16 = 10;
pub const DATA_DIRECTORY_BOUND_IMPORT: u16 = 11;
pub const DATA_DIRECTORY_IAT: u16 = 12;
pub const DATA_DIRECTORY_DELAY_IMPORT_DESCRIPTOR: u16 = 13;
pub const DATA_DIRECTORY_CLR_RUNTIME_HEADER: u16 = 14;

/* reserved: 0x0001, 0x0002, 0x0004, 0x0008 */
pub const DLL_CHARACTERISTIC_HIGH_ENTROPY_VA: u16 = 0x0020;
pub const DLL_CHARACTERISTIC_DYNAMIC_BASE: u16 = 0x0040;
pub const DLL_CHARACTERISTIC_FORCE_INTEGRITY: u16 = 0x0080;
pub const DLL_CHARACTERISTIC_NX_COMPAT: u16 = 0x0100;
pub const DLL_CHARACTERISTIC_NO_ISOLATION: u16 = 0x0200;
pub const DLL_CHARACTERISTIC_NO_SEH: u16 = 0x0400;
pub const DLL_CHARACTERISTIC_NO_BIND: u16 = 0x0800;
pub const DLL_CHARACTERISTIC_APPCONTAINER: u16 = 0x1000;
pub const DLL_CHARACTERISTIC_WDM_DRIVER: u16 = 0x2000;
pub const DLL_CHARACTERISTIC_GUARD_CF: u16 = 0x4000;
pub const DLL_CHARACTERISTIC_TERMINAL_SERVER_AWARE: u16 = 0x8000;

pub const MACHINE_UNKNOWN: u16 = 0x0000;
pub const MACHINE_AM33: u16 = 0x01d3;
pub const MACHINE_AMD64: u16 = 0x8664;
pub const MACHINE_ARM: u16 = 0x01c0;
pub const MACHINE_ARM64: u16 = 0xaa64;
pub const MACHINE_ARMNT: u16 = 0x01c4;
pub const MACHINE_EBC: u16 = 0x0ebc;
pub const MACHINE_I386: u16 = 0x014c;
pub const MACHINE_IA64: u16 = 0x0200;
pub const MACHINE_LOONGARCH32: u16 = 0x6232;
pub const MACHINE_LOONGARCH64: u16 = 0x6264;
pub const MACHINE_M32R: u16 = 0x9041;
pub const MACHINE_MIPS16: u16 = 0x0266;
pub const MACHINE_MIPSFPU: u16 = 0x0366;
pub const MACHINE_MIPSFPU16: u16 = 0x0466;
pub const MACHINE_POWERPC: u16 = 0x01f0;
pub const MACHINE_POWERPCFP: u16 = 0x01f1;
pub const MACHINE_R4000: u16 = 0x0166;
pub const MACHINE_RISCV32: u16 = 0x5032;
pub const MACHINE_RISCV64: u16 = 0x5064;
pub const MACHINE_RISCV128: u16 = 0x5128;
pub const MACHINE_SH3: u16 = 0x01a2;
pub const MACHINE_SH3DSP: u16 = 0x01a3;
pub const MACHINE_SH4: u16 = 0x01a6;
pub const MACHINE_SH5: u16 = 0x01a8;
pub const MACHINE_THUMB: u16 = 0x01c2;
pub const MACHINE_WCEMIPSV2: u16 = 0x0169;

/* reserved: 0x00000001, 0x00000002, 0x00000004 */
pub const SECTION_CHARACTERISTIC_TYPE_NO_PAD: u32 = 0x00000008;
/* reserved: 0x00000010 */
pub const SECTION_CHARACTERISTIC_CNT_CODE: u32 = 0x00000020;
pub const SECTION_CHARACTERISTIC_CNT_INITIALIZED_DATA: u32 = 0x00000040;
pub const SECTION_CHARACTERISTIC_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
pub const SECTION_CHARACTERISTIC_LNK_OTHER: u32 = 0x00000100;
pub const SECTION_CHARACTERISTIC_LNK_INFO: u32 = 0x00000200;
/* reserved: 0x00000400 */
pub const SECTION_CHARACTERISTIC_LNK_REMOVE: u32 = 0x00000800;
pub const SECTION_CHARACTERISTIC_LNK_COMDAT: u32 = 0x00001000;
/* reserved: 0x00002000, 0x00004000 */
pub const SECTION_CHARACTERISTIC_GPREL: u32 = 0x00008000;
pub const SECTION_CHARACTERISTIC_MEM_PURGEABLE: u32 = 0x00020000;
pub const SECTION_CHARACTERISTIC_MEM_16BIT: u32 = 0x00020000;
pub const SECTION_CHARACTERISTIC_MEM_LOCKED: u32 = 0x00040000;
pub const SECTION_CHARACTERISTIC_MEM_PRELOAD: u32 = 0x00080000;
pub const SECTION_CHARACTERISTIC_ALIGN_1BYTES: u32 = 0x00100000;
pub const SECTION_CHARACTERISTIC_ALIGN_2BYTES: u32 = 0x00200000;
pub const SECTION_CHARACTERISTIC_ALIGN_4BYTES: u32 = 0x00300000;
pub const SECTION_CHARACTERISTIC_ALIGN_8BYTES: u32 = 0x00400000;
pub const SECTION_CHARACTERISTIC_ALIGN_16BYTES: u32 = 0x00500000;
pub const SECTION_CHARACTERISTIC_ALIGN_32BYTES: u32 = 0x00600000;
pub const SECTION_CHARACTERISTIC_ALIGN_64BYTES: u32 = 0x00700000;
pub const SECTION_CHARACTERISTIC_ALIGN_128BYTES: u32 = 0x00800000;
pub const SECTION_CHARACTERISTIC_ALIGN_256BYTES: u32 = 0x00900000;
pub const SECTION_CHARACTERISTIC_ALIGN_512BYTES: u32 = 0x00a00000;
pub const SECTION_CHARACTERISTIC_ALIGN_1024BYTES: u32 = 0x00b00000;
pub const SECTION_CHARACTERISTIC_ALIGN_2048BYTES: u32 = 0x00c00000;
pub const SECTION_CHARACTERISTIC_ALIGN_4096BYTES: u32 = 0x00d00000;
pub const SECTION_CHARACTERISTIC_ALIGN_8192BYTES: u32 = 0x00e00000;
pub const SECTION_CHARACTERISTIC_LNK_NRELOC_OVFL: u32 = 0x01000000;
pub const SECTION_CHARACTERISTIC_MEM_DISCARDABLE: u32 = 0x02000000;
pub const SECTION_CHARACTERISTIC_MEM_NOT_CACHED: u32 = 0x04000000;
pub const SECTION_CHARACTERISTIC_MEM_NOT_PAGED: u32 = 0x08000000;
pub const SECTION_CHARACTERISTIC_MEM_SHARED: u32 = 0x10000000;
pub const SECTION_CHARACTERISTIC_MEM_EXECUTE: u32 = 0x20000000;
pub const SECTION_CHARACTERISTIC_MEM_READ: u32 = 0x40000000;
pub const SECTION_CHARACTERISTIC_MEM_WRITE: u32 = 0x80000000;

pub const SUBSYSTEM_UNKNOWN: u16 = 0x0000;
pub const SUBSYSTEM_NATIVE: u16 = 0x0001;
pub const SUBSYSTEM_WINDOWS_GUI: u16 = 0x0002;
pub const SUBSYSTEM_WINDOWS_CUI: u16 = 0x0003;
pub const SUBSYSTEM_OS2_CUI: u16 = 0x0005;
pub const SUBSYSTEM_POSIX_CUI: u16 = 0x0007;
pub const SUBSYSTEM_NATIVE_WINDOWS: u16 = 0x0008;
pub const SUBSYSTEM_WINDOWS_CE_GUI: u16 = 0x0009;
pub const SUBSYSTEM_EFI_APPLICATION: u16 = 0x0010;
pub const SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER: u16 = 0x0011;
pub const SUBSYSTEM_EFI_RUNTIME_DRIVER: u16 = 0x0012;
pub const SUBSYSTEM_EFI_ROM: u16 = 0x0013;
pub const SUBSYSTEM_XBOX: u16 = 0x0014;
pub const SUBSYSTEM_WINDOWS_BOOT_APPLICATION: u16 = 0x0016;

// aligned on 8-byte boundary
#[repr(C)]
pub struct Header {
    pub machine: U16Le,
    pub number_of_sections: U16Le,
    pub time_date_stamp: U32Le,
    pub pointer_to_symbol_table: U32Le,
    pub number_of_symbols: U32Le,
    pub size_of_optional_header: U16Le,
    pub characteristics: U16Le,
}

#[repr(C)]
pub struct OptionalHeader<FORMAT: format::Type = format::Pe> {
    pub magic: U16Le,
    pub major_linker_version: U8Le,
    pub minor_linker_version: U8Le,
    pub size_of_code: U32Le,
    pub size_of_initialized_data: U32Le,
    pub size_of_uninitialized_data: U32Le,
    pub address_of_entry_point: U32Le,
    pub base_of_code: U32Le,
    pub base_of_data: FORMAT::BaseOfData,
}

pub type OptionalHeader32P = OptionalHeader::<format::Pe32P>;

#[repr(C)]
pub struct OptionalHeaderExt<FORMAT: format::Type = format::Pe> {
    pub image_base: FORMAT::AddressSpace,
    pub section_alignment: U32Le,
    pub file_alignment: U32Le,
    pub major_operating_system_version: U16Le,
    pub minor_operating_system_version: U16Le,
    pub major_image_version: U16Le,
    pub minor_image_version: U16Le,
    pub major_subsystem_version: U16Le,
    pub minor_subsystem_version: U16Le,
    pub win32_version_value: U32Le,
    pub size_of_image: U32Le,
    pub size_of_headers: U32Le,
    pub check_sum: U32Le,
    pub subsystem: U16Le,
    pub dll_characteristics: U16Le,
    pub size_of_stack_reserve: FORMAT::AddressSpace,
    pub size_of_stack_commit: FORMAT::AddressSpace,
    pub size_of_heap_reserve: FORMAT::AddressSpace,
    pub size_of_heap_commit: FORMAT::AddressSpace,
    pub loader_flags: U32Le,
    pub number_of_rva_and_sizes: U32Le,
}

pub type OptionalHeaderExt32P = OptionalHeaderExt::<format::Pe32P>;

#[repr(C)]
pub struct DataDirectory {
    pub virtual_address: U32Le,
    pub size: U32Le,
}

#[repr(C)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub virtual_size: U32Le,
    pub virtual_address: U32Le,
    pub size_of_raw_data: U32Le,
    pub pointer_to_raw_data: U32Le,
    pub pointer_to_relocations: U32Le,
    pub pointer_to_linenumbers: U32Le,
    pub number_of_relocations: U16Le,
    pub number_of_linenumbers: U16Le,
    pub characteristics: U32Le,
}

/// Format Parameter Customization
///
/// The PE format comes in multiple types. This module provides a trait named
/// `format::Type` which describes all the possible customizations for the
/// PE format. The PE types are parameterized with this trait if they allow
/// customizations.
///
/// The `Pe` and `Pe32P` types are predefined instances for the Pe and Pe32+
/// types of the format.
pub mod format {
    pub trait Type {
        type AddressSpace;
        type BaseOfData;
    }

    pub struct Pe {}

    impl Type for Pe {
        type AddressSpace = super::U32Le;
        type BaseOfData = super::U32Le;
    }

    pub struct Pe32P {}

    impl Type for Pe32P {
        type AddressSpace = super::U64Le;
        type BaseOfData = ();
    }
}

#[cfg(test)]
mod tests {
    use core::mem::{
        align_of,
        size_of,
    };
    use super::*;

    #[test]
    fn verify_types() {
        assert_eq!(size_of::<Header>(), 20);
        assert_eq!(align_of::<Header>(), 4);

        assert_eq!(size_of::<OptionalHeader>(), 28);
        assert_eq!(align_of::<OptionalHeader>(), 4);
        assert_eq!(size_of::<OptionalHeader32P>(), 24);
        assert_eq!(align_of::<OptionalHeader32P>(), 4);

        assert_eq!(size_of::<OptionalHeaderExt>(), 68);
        assert_eq!(align_of::<OptionalHeaderExt>(), 4);
        assert_eq!(size_of::<OptionalHeaderExt32P>(), 88);
        assert_eq!(align_of::<OptionalHeaderExt32P>(), 8);
    }
}
