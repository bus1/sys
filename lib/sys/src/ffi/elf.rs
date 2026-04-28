//! # Definitions of the Executable and Linkable Format
//!
//! This module provides the definitions of the Executable and Linkable Format
//! (ELF) as native Rust types. It is a transpose of the reference definitions
//! in C to Rust. Please consult an ELF specification for usage information.
//! This module merely documents the differences to the reference specification
//! and how it applies to Rust.
//!
//! Note that this module is a pure import of the definitions from the
//! specification with **NO** accompanying implementation. It is suitable for
//! use in dynamic loaders and other software running in highly constrained
//! environments.
//!
//! The module is available as 32-bit and 64-bit version. Additionally, the
//! `elfn` alias refers to the module native to the target platform.
//!
//! ## Basic Types
//!
//! The ELF basic types (`Addr`, `Off`, `Half`, `Word`, ...) are not
//! provided. Their alignment requirements match their size, and thus there
//! are no suitable native integer types when accessing cross-platform ELF
//! files (e.g., `u64` on a 32bit machine would be 4-byte aligned rather than
//! 8-bytes aligned). Furthermore, terms like `Word` and `Half(-word)` have
//! lost meaning over the years and explicitly sized integers are preferred.
//! Hence, we recommend using native integer types directly.
//!
//! ## References
//!
//! Generic ABI (gABI) Specifications:
//!
//! - Published by SCO and generally most applicable:
//!   <https://www.sco.com/developers/gabi/latest/contents.html>
//! - Published by Oracle and mostly for Solaris:
//!   <https://docs.oracle.com/cd/E18752_01/pdf/817-1984.pdf>
//!
//! Platform Specific ABI (psABI) Specifications:
//!
//! - i386 as published by the Linux Foundation:
//!   <https://refspecs.linuxfoundation.org/elf/abi386-4.pdf>
//! - x86_64 as published by the Linux Foundation:
//!   <https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.99.pdf>
//!
//! Specification Registries:
//!
//! - uclibc maintained registry:
//!   <https://uclibc.org/docs/>
//! - Linux Foundation Reference Specifications:
//!   <https://refspecs.linuxfoundation.org/>

mod common {
    pub const ELFMAG0: u8 = 0x7f;
    pub const ELFMAG1: u8 = b'E';
    pub const ELFMAG2: u8 = b'L';
    pub const ELFMAG3: u8 = b'F';

    pub const ELFMAG: [u8; 4] = [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3];

    pub const ELFCLASSNONE: u8 = 0;
    pub const ELFCLASS32: u8 = 1;
    pub const ELFCLASS64: u8 = 2;

    pub const ELFDATANONE: u8 = 0;
    pub const ELFDATA2LSB: u8 = 1;
    pub const ELFDATA2MSB: u8 = 2;

    pub const ELFOSABI_NONE: u8 = 0;
    pub const ELFOSABI_SYSV: u8 = ELFOSABI_NONE;
    pub const ELFOSABI_HPUX: u8 = 1;
    pub const ELFOSABI_NETBSD: u8 = 2;
    pub const ELFOSABI_GNU: u8 = 3;
    pub const ELFOSABI_LINUX: u8 = ELFOSABI_GNU;
    pub const ELFOSABI_HURD: u8 = 4;
    pub const ELFOSABI_86OPEN: u8 = 5;
    pub const ELFOSABI_SOLARIS: u8 = 6;
    pub const ELFOSABI_AIX: u8 = 7;
    pub const ELFOSABI_MONTEREY: u8 = ELFOSABI_AIX;
    pub const ELFOSABI_IRIX: u8 = 8;
    pub const ELFOSABI_FREEBSD: u8 = 9;
    pub const ELFOSABI_TRU64: u8 = 10;
    pub const ELFOSABI_MODESTO: u8 = 11;
    pub const ELFOSABI_OPENBSD: u8 = 12;
    pub const ELFOSABI_OPENVMS: u8 = 13;
    pub const ELFOSABI_NSK: u8 = 14;
    pub const ELFOSABI_AROS: u8 = 15;
    pub const ELFOSABI_FENIXOS: u8 = 16;
    pub const ELFOSABI_CLOUDABI: u8 = 17;
    pub const ELFOSABI_OPENVOS: u8 = 18;
    pub const ELFOSABI_ARM_AEABI: u8 = 64;
    pub const ELFOSABI_ARM: u8 = 97;
    pub const ELFOSABI_STANDALONE: u8 = 255;

    pub const ET_NONE: u16 = 0;
    pub const ET_REL: u16 = 1;
    pub const ET_EXEC: u16 = 2;
    pub const ET_DYN: u16 = 3;
    pub const ET_CORE: u16 = 4;
    pub const ET_LOOS: u16 = 0xfe00;
    pub const ET_HIOS: u16 = 0xfeff;
    pub const ET_LOPROC: u16 = 0xff00;
    pub const ET_HIPROC: u16 = 0xffff;

    pub const EM_NONE: u16 = 0;
    pub const EM_M32: u16 = 1;
    pub const EM_SPARC: u16 = 2;
    pub const EM_386: u16 = 3;
    pub const EM_68K: u16 = 4;
    pub const EM_88K: u16 = 5;
    pub const EM_IAMCU: u16 = 6;
    pub const EM_486: u16 = EM_IAMCU; // from: linux
    pub const EM_860: u16 = 7;
    pub const EM_MIPS: u16 = 8;
    pub const EM_S370: u16 = 9;
    pub const EM_MIPS_RS3_LE: u16 = 10;
    pub const EM_MIPS_RS4_BE: u16 = EM_MIPS_RS3_LE; // from: linux
    pub const EM_PARISC: u16 = 15;
    pub const EM_VPP500: u16 = 17;
    pub const EM_SPARC32PLUS: u16 = 18;
    pub const EM_960: u16 = 19;
    pub const EM_PPC: u16 = 20;
    pub const EM_PPC64: u16 = 21;
    pub const EM_S390: u16 = 22;
    pub const EM_SPU: u16 = 23;
    pub const EM_V800: u16 = 36;
    pub const EM_FR20: u16 = 37;
    pub const EM_RH32: u16 = 38;
    pub const EM_RCE: u16 = 39;
    pub const EM_ARM: u16 = 40;
    pub const EM_ALPHA: u16 = 41;
    pub const EM_ALPHA_STD: u16 = EM_ALPHA; // from: linux
    pub const EM_SH: u16 = 42;
    pub const EM_SPARCV9: u16 = 43;
    pub const EM_TRICORE: u16 = 44;
    pub const EM_ARC: u16 = 45;
    pub const EM_H8_300: u16 = 46;
    pub const EM_H8_300H: u16 = 47;
    pub const EM_H8S: u16 = 48;
    pub const EM_H8_500: u16 = 49;
    pub const EM_IA_64: u16 = 50;
    pub const EM_MIPS_X: u16 = 51;
    pub const EM_COLDFIRE: u16 = 52;
    pub const EM_68HC12: u16 = 53;
    pub const EM_MMA: u16 = 54;
    pub const EM_PCP: u16 = 55;
    pub const EM_NCPU: u16 = 56;
    pub const EM_NDR1: u16 = 57;
    pub const EM_STARCORE: u16 = 58;
    pub const EM_ME16: u16 = 59;
    pub const EM_ST100: u16 = 60;
    pub const EM_TINYJ: u16 = 61;
    pub const EM_X86_64: u16 = 62;
    pub const EM_AMD64: u16 = EM_X86_64; // from: bionic
    pub const EM_PDSP: u16 = 63;
    pub const EM_PDP10: u16 = 64;
    pub const EM_PDP11: u16 = 65;
    pub const EM_FX66: u16 = 66;
    pub const EM_ST9PLUS: u16 = 67;
    pub const EM_ST7: u16 = 68;
    pub const EM_68HC16: u16 = 69;
    pub const EM_68HC11: u16 = 70;
    pub const EM_68HC08: u16 = 71;
    pub const EM_68HC05: u16 = 72;
    pub const EM_SVX: u16 = 73;
    pub const EM_ST19: u16 = 74;
    pub const EM_VAX: u16 = 75;
    pub const EM_CRIS: u16 = 76;
    pub const EM_JAVELIN: u16 = 77;
    pub const EM_FIREPATH: u16 = 78;
    pub const EM_ZSP: u16 = 79;
    pub const EM_MMIX: u16 = 80;
    pub const EM_HUANY: u16 = 81;
    pub const EM_PRISM: u16 = 82;
    pub const EM_AVR: u16 = 83;
    pub const EM_FR30: u16 = 84;
    pub const EM_D10V: u16 = 85;
    pub const EM_D30V: u16 = 86;
    pub const EM_V850: u16 = 87;
    pub const EM_M32R: u16 = 88;
    pub const EM_MN10300: u16 = 89;
    pub const EM_MN10200: u16 = 90;
    pub const EM_PJ: u16 = 91;
    pub const EM_OPENRISC: u16 = 92;
    pub const EM_ARC_COMPACT: u16 = 93;
    pub const EM_XTENSA: u16 = 94;
    pub const EM_VIDEOCORE: u16 = 95;
    pub const EM_TMM_GPP: u16 = 96;
    pub const EM_NS32K: u16 = 97;
    pub const EM_TPC: u16 = 98;
    pub const EM_SNP1K: u16 = 99;
    pub const EM_ST200: u16 = 100;
    pub const EM_IP2K: u16 = 101;
    pub const EM_MAX: u16 = 102;
    pub const EM_CR: u16 = 103;
    pub const EM_F2MC16: u16 = 104;
    pub const EM_MSP430: u16 = 105;
    pub const EM_BLACKFIN: u16 = 106;
    pub const EM_SE_C33: u16 = 107;
    pub const EM_SEP: u16 = 108;
    pub const EM_ARCA: u16 = 109;
    pub const EM_UNICORE: u16 = 110;
    pub const EM_EXCESS: u16 = 111;
    pub const EM_DXP: u16 = 112;
    pub const EM_ALTERA_NIOS2: u16 = 113;
    pub const EM_CRX: u16 = 114;
    pub const EM_XGATE: u16 = 115;
    pub const EM_C166: u16 = 116;
    pub const EM_M16C: u16 = 117;
    pub const EM_DSPIC30F: u16 = 118;
    pub const EM_CE: u16 = 119;
    pub const EM_M32C: u16 = 120;
    pub const EM_TSK3000: u16 = 131;
    pub const EM_RS08: u16 = 132;
    pub const EM_SHARC: u16 = 133;
    pub const EM_ECOG2: u16 = 134;
    pub const EM_SCORE7: u16 = 135;
    pub const EM_DSP24: u16 = 136;
    pub const EM_VIDEOCORE3: u16 = 137;
    pub const EM_LATTICEMICO32: u16 = 138;
    pub const EM_SE_C17: u16 = 139;
    pub const EM_TI_C6000: u16 = 140;
    pub const EM_TI_C2000: u16 = 141;
    pub const EM_TI_C5500: u16 = 142;
    pub const EM_TI_ARP32: u16 = 143;
    pub const EM_TI_PRU: u16 = 144;
    pub const EM_MMDSP_PLUS: u16 = 160;
    pub const EM_CYPRESS_M8C: u16 = 161;
    pub const EM_R32C: u16 = 162;
    pub const EM_TRIMEDIA: u16 = 162;
    pub const EM_QDSP6: u16 = 163;
    pub const EM_8051: u16 = 164;
    pub const EM_STXP7X: u16 = 165;
    pub const EM_NDS32: u16 = 166;
    pub const EM_ECOG1: u16 = 167;
    pub const EM_ECOG1X: u16 = 168;
    pub const EM_MAXQ30: u16 = 169;
    pub const EM_XIMO16: u16 = 170;
    pub const EM_MANIK: u16 = 171;
    pub const EM_CRAYNV2: u16 = 172;
    pub const EM_RX: u16 = 173;
    pub const EM_METAG: u16 = 174;
    pub const EM_MCST_ELBRUS: u16 = 175;
    pub const EM_ECOG16: u16 = 176;
    pub const EM_CR16: u16 = 177;
    pub const EM_ETPU: u16 = 178;
    pub const EM_SLE9X: u16 = 179;
    pub const EM_L10M: u16 = 180;
    pub const EM_K10M: u16 = 181;
    pub const EM_AARCH64: u16 = 183;
    pub const EM_AVR32: u16 = 185;
    pub const EM_STM8: u16 = 186;
    pub const EM_TILE64: u16 = 187;
    pub const EM_TILEPRO: u16 = 188;
    pub const EM_MICROBLAZE: u16 = 189;
    pub const EM_CUDA: u16 = 190;
    pub const EM_TILEGX: u16 = 191;
    pub const EM_CLOUDSHIELD: u16 = 192;
    pub const EM_COREA_1ST: u16 = 193;
    pub const EM_COREA_2ND: u16 = 194;
    pub const EM_ARC_COMPACT2: u16 = 195;
    pub const EM_OPEN8: u16 = 196;
    pub const EM_RL78: u16 = 197;
    pub const EM_VIDEOCORE5: u16 = 198;
    pub const EM_78KOR: u16 = 199;
    pub const EM_56800EX: u16 = 200;
    pub const EM_BA1: u16 = 201;
    pub const EM_BA2: u16 = 202;
    pub const EM_XCORE: u16 = 203;
    pub const EM_MCHP_PIC: u16 = 204;
    pub const EM_INTEL205: u16 = 205;
    pub const EM_INTEL206: u16 = 206;
    pub const EM_INTEL207: u16 = 207;
    pub const EM_INTEL208: u16 = 208;
    pub const EM_INTEL209: u16 = 209;
    pub const EM_KM32: u16 = 210;
    pub const EM_KMX32: u16 = 211;
    pub const EM_KMX16: u16 = 212;
    pub const EM_KMX8: u16 = 213;
    pub const EM_KVARC: u16 = 214;
    pub const EM_CDP: u16 = 215;
    pub const EM_COGE: u16 = 216;
    pub const EM_COOL: u16 = 217;
    pub const EM_NORC: u16 = 218;
    pub const EM_CSR_KALIMBA: u16 = 219;
    pub const EM_Z80: u16 = 220;
    pub const EM_VISIUM: u16 = 221;
    pub const EM_FT32: u16 = 222;
    pub const EM_MOXIE: u16 = 223;
    pub const EM_AMDGPU: u16 = 224;
    pub const EM_RISCV: u16 = 243;
    pub const EM_BPF: u16 = 247; // from: linux
    pub const EM_CSKY: u16 = 252; // from: linux
    pub const EM_ARCV3_32: u16 = 255; // from: uclibc-ng
    pub const EM_KVX: u16 = 256; // from: uclibc-ng
    pub const EM_LOONGARCH: u16 = 258; // from: linux
    pub const EM_AVR_INTERIM: u16 = 0x1057; // from: uclibc-ng
    pub const EM_MSP430_INTERIM: u16 = 0x1059; // from: uclibc-ng
    pub const EM_AVR32_INTERIM: u16 = 0x18ad; // from: uclibc-ng
    pub const EM_MT_INTERIM: u16 = 0x2530; // from: uclibc-ng
    pub const EM_CYGNUS_FR30_INTERIM: u16 = 0x3330; // from: uclibc-ng
    pub const EM_OPENRISC_INTERIM: u16 = 0x3426; // from: uclibc-ng
    pub const EM_FRV_INTERIM: u16 = 0x5441; // from: linux
    pub const EM_CYGNUS_FRV_INTERIM: u16 = EM_FRV_INTERIM; // from: uclibc-ng
    pub const EM_DLX_INTERIM: u16 = 0x5aa5; // from: uclibc-ng
    pub const EM_CYGNUS_D10V_INTERIM: u16 = 0x7650; // from: uclibc-ng
    pub const EM_CYGNUS_D30V_INTERIM: u16 = 0x7676; // from: uclibc-ng
    pub const EM_IP2K_INTERIM: u16 = 0x8217; // from: uclibc-ng
    pub const EM_OR32_INTERIM: u16 = 0x8472; // from: uclibc-ng
    pub const EM_CYGNUS_POWERPC_INTERIM: u16 = 0x9025; // from: uclibc-ng
    pub const EM_ALPHA_INTERIM: u16 = 0x9026; // from: linux
    pub const EM_CYGNUS_M32R_INTERIM: u16 = 0x9041; // from: linux
    pub const EM_CYGNUS_V850_INTERIM: u16 = 0x9080; // from: uclibc-ng
    pub const EM_S390_INTERIM: u16 = 0xa390; // from: linux
    pub const EM_XTENSA_INTERIM: u16 = 0xabc7; // from: uclibc-ng
    pub const EM_XSTORMY16_INTERIM: u16 = 0xad45; // from: uclibc-ng
    pub const EM_CYGNUS_MN10300_INTERIM: u16 = 0xbeef; // from: linux
    pub const EM_CYGNUS_MN10200_INTERIM: u16 = 0xdead; // from: uclibc-ng
    pub const EM_M32C_INTERIM: u16 = 0xfeb8; // from: uclibc-ng
    pub const EM_IQ2000_INTERIM: u16 = 0xfeba; // from: uclibc-ng
    pub const EM_NIOS32_INTERIM: u16 = 0xfebb; // from: uclibc-ng

    pub const EV_NONE: u8 = 0;
    pub const EV_CURRENT: u8 = 1;

    pub const SHN_UNDEF: u16 = 0;
    pub const SHN_LORESERVE: u16 = 0xff00;
    pub const SHN_LOPROC: u16 = 0xff00;
    pub const SHN_BEFORE: u16 = 0xff00; // from: solaris
    pub const SHN_AFTER: u16 = 0xff01; // from: solaris
    pub const SHN_HIPROC: u16 = 0xff1f;
    pub const SHN_LOOS: u16 = 0xff20;
    pub const SHN_FBSD_CACHED: u16 = 0xff20; // from: bionic
    pub const SHN_LIVEPATCH: u16 = 0xff20; // from: linux
    pub const SHN_HIOS: u16 = 0xff3f;
    pub const SHN_ABS: u16 = 0xfff1;
    pub const SHN_COMMON: u16 = 0xfff2;
    pub const SHN_XINDEX: u16 = 0xffff;
    pub const SHN_HIRESERVE: u16 = 0xffff;

    pub const SHN_LOSUNW: u16 = 0xff3f;
    pub const SHN_SUNW_IGNORE: u16 = 0xff3f;
    pub const SHN_HISUNW: u16 = 0xff3f;

    pub const SHN_AMD64_LCOMMON: u16 = 0xff02; // from: oracle

    pub const SHT_NULL: u32 = 0;
    pub const SHT_PROGBITS: u32 = 1;
    pub const SHT_SYMTAB: u32 = 2;
    pub const SHT_STRTAB: u32 = 3;
    pub const SHT_RELA: u32 = 4;
    pub const SHT_HASH: u32 = 5;
    pub const SHT_DYNAMIC: u32 = 6;
    pub const SHT_NOTE: u32 = 7;
    pub const SHT_NOBITS: u32 = 8;
    pub const SHT_REL: u32 = 9;
    pub const SHT_SHLIB: u32 = 10;
    pub const SHT_DYNSYM: u32 = 11;
    pub const SHT_INIT_ARRAY: u32 = 14;
    pub const SHT_FINI_ARRAY: u32 = 15;
    pub const SHT_PREINIT_ARRAY: u32 = 16;
    pub const SHT_GROUP: u32 = 17;
    pub const SHT_SYMTAB_SHNDX: u32 = 18;
    pub const SHT_RELR: u32 = 19; // from: glibc
    pub const SHT_LOOS: u32 = 0x60000000;
    pub const SHT_CHECKSUM: u32 = 0x6ffffff8; // from: uclibc-ng
    pub const SHT_HIOS: u32 = 0x6fffffff;
    pub const SHT_LOPROC: u32 = 0x70000000;
    pub const SHT_HIPROC: u32 = 0x7fffffff;
    pub const SHT_LOUSER: u32 = 0x80000000;
    pub const SHT_HIUSER: u32 = 0xffffffff;

    pub const SHT_LOGNU: u32 = 0x6ffffff5;
    pub const SHT_GNU_ATTRIBUTES: u32 = 0x6ffffff5;
    pub const SHT_GNU_HASH: u32 = 0x6ffffff6;
    pub const SHT_GNU_LIBLIST: u32 = 0x6ffffff7;
    pub const SHT_GNU_VERDEF: u32 = 0x6ffffffd;
    pub const SHT_GNU_VERNEED: u32 = 0x6ffffffe;
    pub const SHT_GNU_VERSYM: u32 = 0x6fffffff;
    pub const SHT_HIGNU: u32 = 0x6fffffff;

    pub const SHT_LOSUNW: u32 = 0x6fffffef;
    pub const SHT_SUNW_CAPCHAIN: u32 = 0x6fffffef;
    pub const SHT_SUNW_CAPINFO: u32 = 0x6ffffff0;
    pub const SHT_SUNW_SYMSORT: u32 = 0x6ffffff1;
    pub const SHT_SUNW_TLSSORT: u32 = 0x6ffffff2;
    pub const SHT_SUNW_LDYNSYM: u32 = 0x6ffffff3;
    pub const SHT_SUNW_DOF: u32 = 0x6ffffff4;
    pub const SHT_SUNW_CAP: u32 = 0x6ffffff5;
    pub const SHT_SUNW_SIGNATURE: u32 = 0x6ffffff6;
    pub const SHT_SUNW_ANNOTATE: u32 = 0x6ffffff7;
    pub const SHT_SUNW_DEBUGSTR: u32 = 0x6ffffff8;
    pub const SHT_SUNW_DEBUG: u32 = 0x6ffffff9;
    pub const SHT_SUNW_MOVE: u32 = 0x6ffffffa;
    pub const SHT_SUNW_COMDAT: u32 = 0x6ffffffb;
    pub const SHT_SUNW_SYMINFO: u32 = 0x6ffffffc;
    pub const SHT_SUNW_VERDEF: u32 = 0x6ffffffd;
    pub const SHT_SUNW_VERNEED: u32 = 0x6ffffffe;
    pub const SHT_SUNW_VERSYM: u32 = 0x6fffffff;
    pub const SHT_HISUNW: u32 = 0x6fffffff;

    pub const SHT_AMD64_UNWIND: u32 = 0x70000001; // from: oracle

    pub const SHT_SPARC_GOTDATA: u32 = 0x70000000; // from: oracle

    pub const SHF_WRITE: u32 = 0x00000001;
    pub const SHF_ALLOC: u32 = 0x00000002;
    pub const SHF_EXECINSTR: u32 = 0x00000004;
    pub const SHF_MERGE: u32 = 0x00000010;
    pub const SHF_STRINGS: u32 = 0x00000020;
    pub const SHF_INFO_LINK: u32 = 0x00000040;
    pub const SHF_LINK_ORDER: u32 = 0x00000080;
    pub const SHF_OS_NONCONFORMING: u32 = 0x00000100;
    pub const SHF_GROUP: u32 = 0x00000200;
    pub const SHF_TLS: u32 = 0x00000400;
    pub const SHF_COMPRESSED: u32 = 0x00000800;
    pub const SHF_RELA_LIVEPATCH: u32 = 0x00100000; // from: linux
    pub const SHF_RO_AFTER_INIT: u32 = 0x00200000; // from: linux
    pub const SHF_GNU_RETAIN: u32 = 0x00200000; // from: glibc
    pub const SHF_ORDERED: u32 = 0x40000000; // from: solaris
    pub const SHF_EXCLUDE: u32 = 0x80000000; // from: solaris
    pub const SHF_MASKOS: u32 = 0x0ff00000;
    pub const SHF_MASKPROC: u32 = 0xf0000000;

    pub const SHF_AMD64_LARGE: u32 = 0x10000000; // from: oracle

    pub const PT_NULL: u32 = 0;
    pub const PT_LOAD: u32 = 1;
    pub const PT_DYNAMIC: u32 = 2;
    pub const PT_INTERP: u32 = 3;
    pub const PT_NOTE: u32 = 4;
    pub const PT_SHLIB: u32 = 5;
    pub const PT_PHDR: u32 = 6;
    pub const PT_TLS: u32 = 7;
    pub const PT_LOOS: u32 = 0x60000000;
    pub const PT_SUNW_UNWIND: u32 = 0x6464e550; // from: oracle
    pub const PT_SUNW_EH_FRAME: u32 = 0x6474e550; // from: oracle
    pub const PT_PAX_FLAGS: u32 = 0x65041580; // from: uclibc-ng
    pub const PT_DUMP_DELTA: u32 = 0x6fb5d000; // from: bionic
    pub const PT_HIOS: u32 = 0x6fffffff;
    pub const PT_LOPROC: u32 = 0x70000000;
    pub const PT_HIPROC: u32 = 0x7fffffff;

    pub const PT_LOGNU: u32 = 0x6474e550;
    pub const PT_GNU_EH_FRAME: u32 = 0x6474e550; // from: glibc
    pub const PT_GNU_STACK: u32 = 0x6474e551; // from: glibc
    pub const PT_GNU_RELRO: u32 = 0x6474e552; // from: glibc
    pub const PT_GNU_PROPERTY: u32 = 0x6474e553; // from: glibc
    pub const PT_GNU_SFRAME: u32 = 0x6474e554; // from: glibc
    pub const PT_HIGNU: u32 = 0x6474e554;

    pub const PT_LOOPENBSD: u32 = 0x65a3dbe6;
    pub const PT_OPENBSD_RANDOMIZE: u32 = 0x65a3dbe6; // from: bionic
    pub const PT_OPENBSD_WXNEEDED: u32 = 0x65a3dbe7; // from: bionic
    pub const PT_OPENBSD_BOOTDATA: u32 = 0x65a41be6; // from: bionic
    pub const PT_HIOPENBSD: u32 = 0x65a41be6;

    pub const PT_LOSUNW: u32 = 0x6ffffffa; // from: oracle
    pub const PT_SUNWBSS: u32 = 0x6ffffffa; // from: oracle
    pub const PT_SUNWSTACK: u32 = 0x6ffffffb; // from: oracle
    pub const PT_SUNWDTRACE: u32 = 0x6ffffffc; // from: oracle
    pub const PT_SUNWCAP: u32 = 0x6ffffffd; // from: oracle
    pub const PT_HISUNW: u32 = 0x6fffffff; // from: oracle

    pub const PF_X: u32 = 0x00000001;
    pub const PF_W: u32 = 0x00000002;
    pub const PF_R: u32 = 0x00000004;
    pub const PF_PAGEEXEC: u32 = 0x00000010; // from: uclibc-ng
    pub const PF_NOPAGEEXEC: u32 = 0x00000020; // from: uclibc-ng
    pub const PF_SEGMEXEC: u32 = 0x00000040; // from: uclibc-ng
    pub const PF_NOSEGMEXEC: u32 = 0x00000080; // from: uclibc-ng
    pub const PF_MPROTECT: u32 = 0x00000100; // from: uclibc-ng
    pub const PF_NOMPROTECT: u32 = 0x00000200; // from: uclibc-ng
    pub const PF_RANDEXEC: u32 = 0x00000400; // from: uclibc-ng
    pub const PF_NORANDEXEC: u32 = 0x00000800; // from: uclibc-ng
    pub const PF_EMUTRAMP: u32 = 0x00001000; // from: uclibc-ng
    pub const PF_NOEMUTRAMP: u32 = 0x00002000; // from: uclibc-ng
    pub const PF_RANDMMAP: u32 = 0x00004000; // from: uclibc-ng
    pub const PF_NORANDMMAP: u32 = 0x00008000; // from: uclibc-ng
    pub const PF_MASKOS: u32 = 0x0ff00000;
    pub const PF_MASKPROC: u32 = 0xf0000000;

    pub const STB_LOCAL: u8 = 0;
    pub const STB_GLOBAL: u8 = 1;
    pub const STB_WEAK: u8 = 2;
    pub const STB_LOOS: u8 = 10;
    pub const STB_GNU_UNIQUE: u8 = 10;
    pub const STB_HIOS: u8 = 12;
    pub const STB_LOPROC: u8 = 13;
    pub const STB_MIPS_SPLIT_COMMON: u8 = 13;
    pub const STB_HIPROC: u8 = 15;

    pub const STT_NOTYPE: u8 = 0;
    pub const STT_OBJECT: u8 = 1;
    pub const STT_FUNC: u8 = 2;
    pub const STT_SECTION: u8 = 3;
    pub const STT_FILE: u8 = 4;
    pub const STT_COMMON: u8 = 5;
    pub const STT_TLS: u8 = 6;
    pub const STT_LOOS: u8 = 10;
    pub const STT_GNU_IFUNC: u8 = 10;
    pub const STT_HP_OPAQUE: u8 = 11;
    pub const STT_HP_STUB: u8 = 12;
    pub const STT_HIOS: u8 = 12;
    pub const STT_LOPROC: u8 = 13;
    pub const STT_ARM_TFUNC: u8 = 13;
    pub const STT_PARISC_MILLICODE: u8 = 13;
    pub const STT_SPARC_REGISTER: u8 = 13;
    pub const STT_ARM_16BIT: u8 = 15;
    pub const STT_HIPROC: u8 = 15;

    pub const STV_DEFAULT: u8 = 0;
    pub const STV_INTERNAL: u8 = 1;
    pub const STV_HIDDEN: u8 = 2;
    pub const STV_PROTECTED: u8 = 3;
    pub const STV_EXPORTED: u8 = 4; // from: bionic
    pub const STV_SINGLETON: u8 = 5; // from: bionic
    pub const STV_ELIMINATE: u8 = 6; // from: bionic

    pub const DT_NULL: u32 = 0;
    pub const DT_NEEDED: u32 = 1;
    pub const DT_PLTRELSZ: u32 = 2;
    pub const DT_PLTGOT: u32 = 3;
    pub const DT_HASH: u32 = 4;
    pub const DT_STRTAB: u32 = 5;
    pub const DT_SYMTAB: u32 = 6;
    pub const DT_RELA: u32 = 7;
    pub const DT_RELASZ: u32 = 8;
    pub const DT_RELAENT: u32 = 9;
    pub const DT_STRSZ: u32 = 10;
    pub const DT_SYMENT: u32 = 11;
    pub const DT_INIT: u32 = 12;
    pub const DT_FINI: u32 = 13;
    pub const DT_SONAME: u32 = 14;
    pub const DT_RPATH: u32 = 15;
    pub const DT_SYMBOLIC: u32 = 16;
    pub const DT_REL: u32 = 17;
    pub const DT_RELSZ: u32 = 18;
    pub const DT_RELENT: u32 = 19;
    pub const DT_PLTREL: u32 = 20;
    pub const DT_DEBUG: u32 = 21;
    pub const DT_TEXTREL: u32 = 22;
    pub const DT_JMPREL: u32 = 23;
    pub const DT_BIND_NOW: u32 = 24;
    pub const DT_INIT_ARRAY: u32 = 25;
    pub const DT_FINI_ARRAY: u32 = 26;
    pub const DT_INIT_ARRAYSZ: u32 = 27;
    pub const DT_FINI_ARRAYSZ: u32 = 28;
    pub const DT_RUNPATH: u32 = 29;
    pub const DT_FLAGS: u32 = 30;
    pub const DT_ENCODING: u32 = 32;
    pub const DT_PREINIT_ARRAY: u32 = 32;
    pub const DT_PREINIT_ARRAYSZ: u32 = 33;
    pub const DT_SYMTAB_SHNDX: u32 = 34;
    pub const DT_RELRSZ: u32 = 35;
    pub const DT_RELR: u32 = 36;
    pub const DT_RELRENT: u32 = 37;
    pub const DT_LOOS: u32 = 0x6000000d;
    pub const DT_HIOS: u32 = 0x6ffff000;
    pub const DT_LOPROC: u32 = 0x70000000;
    pub const DT_HIPROC: u32 = 0x7fffffff;

    pub const DT_SUNW_AUXILIARY: u32 = 0x6000000d;
    pub const DT_SUNW_RTLDINF: u32 = 0x6000000e;
    pub const DT_SUNW_FILTER: u32 = 0x6000000f;
    pub const DT_SUNW_CAP: u32 = 0x60000010;
    pub const DT_SUNW_ASLR: u32 = 0x60000023;

    pub const DT_VALRNGLO: u32 = 0x6ffffd00;
    pub const DT_GNU_PRELINKED: u32 = 0x6ffffdf5;
    pub const DT_GNU_CONFLICTSZ: u32 = 0x6ffffdf6;
    pub const DT_GNU_LIBLISTSZ: u32 = 0x6ffffdf7;
    pub const DT_CHECKSUM: u32 = 0x6ffffdf8;
    pub const DT_PLTPADSZ: u32 = 0x6ffffdf9;
    pub const DT_MOVEENT: u32 = 0x6ffffdfa;
    pub const DT_MOVESZ: u32 = 0x6ffffdfb;
    pub const DT_FEATURE_1: u32 = 0x6ffffdfc;
    pub const DT_POSFLAG_1: u32 = 0x6ffffdfd;
    pub const DT_SYMINSZ: u32 = 0x6ffffdfe;
    pub const DT_SYMINENT: u32 = 0x6ffffdff;
    pub const DT_VALRNGHI: u32 = 0x6ffffdff;

    pub const DT_ADDRRNGLO: u32 = 0x6ffffe00;
    pub const DT_GNU_HASH: u32 = 0x6ffffef5;
    pub const DT_TLSDESC_PLT: u32 = 0x6ffffef6;
    pub const DT_TLSDESC_GOT: u32 = 0x6ffffef7;
    pub const DT_GNU_CONFLICT: u32 = 0x6ffffef8;
    pub const DT_GNU_LIBLIST: u32 = 0x6ffffef9;
    pub const DT_CONFIG: u32 = 0x6ffffefa;
    pub const DT_DEPAUDIT: u32 = 0x6ffffefb;
    pub const DT_AUDIT: u32 = 0x6ffffefc;
    pub const DT_PLTPAD: u32 = 0x6ffffefd;
    pub const DT_MOVETAB: u32 = 0x6ffffefe;
    pub const DT_SYMINFO: u32 = 0x6ffffeff;
    pub const DT_ADDRRNGHI: u32 = 0x6ffffeff;

    pub const DT_VERSYM: u32 = 0x6ffffff0;
    pub const DT_RELACOUNT: u32 = 0x6ffffff9;
    pub const DT_RELCOUNT: u32 = 0x6ffffffa;
    pub const DT_FLAGS_1: u32 = 0x6ffffffb;
    pub const DT_VERDEF: u32 = 0x6ffffffc;
    pub const DT_VERDEFNUM: u32 = 0x6ffffffd;
    pub const DT_VERNEED: u32 = 0x6ffffffe;
    pub const DT_VERNEEDNUM: u32 = 0x6fffffff;

    pub const DT_AUXILIARY: u32 = 0x7ffffffd;
    pub const DT_USED: u32 = 0x7ffffffe;
    pub const DT_FILTER: u32 = 0x7fffffff;

    pub const DF_ORIGIN: u32 = 0x00000001;
    pub const DF_SYMBOLIC: u32 = 0x00000002;
    pub const DF_TEXTREL: u32 = 0x00000004;
    pub const DF_BIND_NOW: u32 = 0x00000008;
    pub const DF_STATIC_TLS: u32 = 0x00000010;

    pub const DF_1_NOW: u32 = 0x00000001;
    pub const DF_1_GLOBAL: u32 = 0x00000002;
    pub const DF_1_GROUP: u32 = 0x00000004;
    pub const DF_1_NODELETE: u32 = 0x00000008;
    pub const DF_1_LOADFLTR: u32 = 0x00000010;
    pub const DF_1_INITFIRST: u32 = 0x00000020;
    pub const DF_1_NOOPEN: u32 = 0x00000040;
    pub const DF_1_ORIGIN: u32 = 0x00000080;
    pub const DF_1_DIRECT: u32 = 0x00000100;
    pub const DF_1_TRANS: u32 = 0x00000200;
    pub const DF_1_INTERPOSE: u32 = 0x00000400;
    pub const DF_1_NODEFLIB: u32 = 0x00000800;
    pub const DF_1_NODUMP: u32 = 0x00001000;
    pub const DF_1_CONFALT: u32 = 0x00002000;
    pub const DF_1_ENDFILTEE: u32 = 0x00004000;
    pub const DF_1_DISPRELDNE: u32 = 0x00008000;
    pub const DF_1_DISPRELPND: u32 = 0x00010000;
    pub const DF_1_NODIRECT: u32 = 0x00020000;
    pub const DF_1_IGNMULDEF: u32 = 0x00040000;
    pub const DF_1_NOKSYMS: u32 = 0x00080000;
    pub const DF_1_NOHDR: u32 = 0x00100000;
    pub const DF_1_EDITED: u32 = 0x00200000;
    pub const DF_1_NORELOC: u32 = 0x00400000;
    pub const DF_1_SYMINTPOSE: u32 = 0x00800000;
    pub const DF_1_GLOBAUDIT: u32 = 0x01000000;
    pub const DF_1_SINGLETON: u32 = 0x02000000;
    pub const DF_1_STUB: u32 = 0x04000000;
    pub const DF_1_PIE: u32 = 0x08000000;
    pub const DF_1_KMOD: u32 = 0x10000000;
    pub const DF_1_WEAKFILTER: u32 = 0x20000000;
    pub const DF_1_NOCOMMON: u32 = 0x40000000;

    pub const DTF_1_PARINIT: u32 = 0x00000001;
    pub const DTF_1_CONFEXP: u32 = 0x00000002;

    pub const DF_P1_LAZYLOAD: u32 = 0x00000001;
    pub const DF_P1_GROUPPERM: u32 = 0x00000002;

    /// Identification Table
    ///
    /// The first 16 bytes of the ELF header contain the identification table.
    /// Typically, it is represented as a 16-byte array with open-coded byte
    /// offsets. For better readability, this implementation provides a 1-byte
    /// aligned structure with named member fields.
    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct Ident {
        pub i_magic: [u8; 4],
        pub i_class: u8,
        pub i_data: u8,
        pub i_version: u8,
        pub i_osabi: u8,
        pub i_abiversion: u8,
        pub i_pad: [u8; 7],
    }
}

macro_rules! impl_elf {
    (
        $mod:ident,
        $align:meta,
        $usize:ty,
        $isize:ty,
        ($($ty_phdr:tt)*),
        ($($ty_sym:tt)*)
        $(,)?
    ) => {
        /// ELF Header
        ///
        /// The first bytes of an ELF file are occupied by the ELF header,
        /// which itself starts with the ELF identification table.
        #[derive(Clone, Copy, Debug)]
        #[repr(C, $align)]
        pub struct Ehdr {
            pub e_ident: Ident,
            pub e_type: u16,
            pub e_machine: u16,
            pub e_version: u32,
            pub e_entry: $usize,
            pub e_phoff: $usize,
            pub e_shoff: $usize,
            pub e_flags: u32,
            pub e_ehsize: u16,
            pub e_phentsize: u16,
            pub e_phnum: u16,
            pub e_shentsize: u16,
            pub e_shnum: u16,
            pub e_shstrndx: u16,
        }

        /// Section Header
        ///
        /// A section header describes a section of an ELF file. It contains
        /// all metadata on the section and points to the relevant parts in the
        /// file for section content.
        #[derive(Clone, Copy, Debug)]
        #[repr(C, $align)]
        pub struct Shdr {
            pub sh_name: u32,
            pub sh_type: u32,
            pub sh_flags: $usize,
            pub sh_addr: $usize,
            pub sh_offset: $usize,
            pub sh_size: $usize,
            pub sh_link: u32,
            pub sh_info: u32,
            pub sh_addralign: $usize,
            pub sh_entsize: $usize,
        }

        /// Program Header
        ///
        /// A program header describes a segment of an ELF file. It contains
        /// all metadata relevant to a segment and points to the segment data
        /// in the file.
        $($ty_phdr)*

        /// Symbol Value
        ///
        /// A symbol is a named reference to data in an ELF file. The symbol
        /// header describes the metadata relevant to a symbol.
        $($ty_sym)*

        /// Dynamic Sections
        ///
        /// The dynamic section contains information needed for dynamic loading
        /// and linking of the binary. It is usually exported in the dynamic
        /// segment.
        ///
        /// # Inlined Value
        ///
        /// Traditionally, the struct has a `d_un` field, which itself is a
        /// union with two equally sized fields `d_val` and `d_ptr`. Since they
        /// have the same size and evaluate to the same integer type, they are
        /// merely documentational.
        ///
        /// This Rust struct uses `d_val` directly and does not provide any
        /// alias. This makes the struct a lot easier to work with in Rust,
        /// since a union would require a lot of manual trait implementations
        /// and unsafe operations, which seems excessive given that it is a
        /// mere alias.
        #[derive(Clone, Copy, Debug)]
        #[repr(C, $align)]
        pub struct Dyn {
            pub d_tag: $usize,
            pub d_val: $usize,
        }

        /// Relocation Information
        ///
        /// Code relocations with implicit addend use this structure to
        /// describe all relevant metadata for the relocation.
        #[derive(Clone, Copy, Debug)]
        #[repr(C, $align)]
        pub struct Rel {
            pub r_offset: $usize,
            pub r_info: $usize,
        }

        /// Relocation Information with Addend
        ///
        /// Code relocations with explicit addend use this structure to
        /// describe all relevant metadata for the relocation.
        #[derive(Clone, Copy, Debug)]
        #[repr(C, $align)]
        pub struct Rela {
            pub r_offset: $usize,
            pub r_info: $usize,
            pub r_addend: $isize,
        }
    };
}

/// # ELF for 32-bit
///
/// This module exposes the ELF API for 32-bit machines.
pub mod elf32 {
    pub use super::common::*;

    impl_elf!(
        elf32,
        align(4),
        u32,
        i32,
        (
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(4))]
            pub struct Phdr {
                pub p_type: u32,
                pub p_offset: u32,
                pub p_vaddr: u32,
                pub p_paddr: u32,
                pub p_filesz: u32,
                pub p_memsz: u32,
                pub p_flags: u32,
                pub p_align: u32,
            }
        ),
        (
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(4))]
            pub struct Sym {
                pub st_name: u32,
                pub st_value: u32,
                pub st_size: u32,
                pub st_info: u8,
                pub st_other: u8,
                pub st_shndx: u16,
            }
        ),
    );
}

/// # ELF for 64-bit
///
/// This module exposes the ELF API for 64-bit machines.
pub mod elf64 {
    pub use super::common::*;

    impl_elf!(
        elf64,
        align(8),
        u64,
        i64,
        (
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(8))]
            pub struct Phdr {
                pub p_type: u32,
                pub p_flags: u32,
                pub p_offset: u64,
                pub p_vaddr: u64,
                pub p_paddr: u64,
                pub p_filesz: u64,
                pub p_memsz: u64,
                pub p_align: u64,
            }
        ),
        (
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(8))]
            pub struct Sym {
                pub st_name: u32,
                pub st_info: u8,
                pub st_other: u8,
                pub st_shndx: u16,
                pub st_value: u64,
                pub st_size: u64,
            }
        ),
    );
}

/// # ELF for Native Access
///
/// This module is an alias for either [`elf32`] or [`elf64`], matching the
/// format used by the target platform.
#[cfg(target_pointer_width = "32")]
pub use elf32 as elfn;
#[cfg(target_pointer_width = "64")]
pub use elf64 as elfn;

#[cfg(test)]
mod test {
    use super::*;

    // Verify that `elfn` represents the native platform.
    #[test]
    fn native() {
        // `Rel` contains two addresses, so it must be aligned respectively.
        assert_eq!(align_of::<elfn::Rel>(), align_of::<usize>());
        assert_eq!(size_of::<elfn::Rel>(), size_of::<usize>() * 2);

        // Similarly, ensure that `Rel` matches the target pointer width. We
        // use this to figure out the native ELF style.
        osi::cfg::cond! {
            (target_pointer_width = "32") {
                assert_eq!(align_of::<elfn::Rel>(), 4);
                assert_eq!(size_of::<elfn::Rel>(), 8);
            },
            (target_pointer_width = "64") {
                assert_eq!(align_of::<elfn::Rel>(), 8);
                assert_eq!(size_of::<elfn::Rel>(), 16);
            },
        }
    }

    // Verify that the different ELF structures are aligned and sized as
    // expected.
    #[test]
    fn typeinfo() {
        assert_eq!(align_of::<common::Ident>(), 1);
        assert_eq!(size_of::<common::Ident>(), 16);

        assert_eq!(align_of::<elf32::Ident>(), 1);
        assert_eq!(size_of::<elf32::Ident>(), 16);
        assert_eq!(align_of::<elf32::Ehdr>(), 4);
        assert_eq!(size_of::<elf32::Ehdr>(), 52);
        assert_eq!(align_of::<elf32::Shdr>(), 4);
        assert_eq!(size_of::<elf32::Shdr>(), 40);
        assert_eq!(align_of::<elf32::Phdr>(), 4);
        assert_eq!(size_of::<elf32::Phdr>(), 32);
        assert_eq!(align_of::<elf32::Sym>(), 4);
        assert_eq!(size_of::<elf32::Sym>(), 16);
        assert_eq!(align_of::<elf32::Dyn>(), 4);
        assert_eq!(size_of::<elf32::Dyn>(), 8);
        assert_eq!(align_of::<elf32::Rel>(), 4);
        assert_eq!(size_of::<elf32::Rel>(), 8);
        assert_eq!(align_of::<elf32::Rela>(), 4);
        assert_eq!(size_of::<elf32::Rela>(), 12);

        assert_eq!(align_of::<elf64::Ident>(), 1);
        assert_eq!(size_of::<elf64::Ident>(), 16);
        assert_eq!(align_of::<elf64::Ehdr>(), 8);
        assert_eq!(size_of::<elf64::Ehdr>(), 64);
        assert_eq!(align_of::<elf64::Shdr>(), 8);
        assert_eq!(size_of::<elf64::Shdr>(), 64);
        assert_eq!(align_of::<elf64::Phdr>(), 8);
        assert_eq!(size_of::<elf64::Phdr>(), 56);
        assert_eq!(align_of::<elf64::Sym>(), 8);
        assert_eq!(size_of::<elf64::Sym>(), 24);
        assert_eq!(align_of::<elf64::Dyn>(), 8);
        assert_eq!(size_of::<elf64::Dyn>(), 16);
        assert_eq!(align_of::<elf64::Rel>(), 8);
        assert_eq!(size_of::<elf64::Rel>(), 16);
        assert_eq!(align_of::<elf64::Rela>(), 8);
        assert_eq!(size_of::<elf64::Rela>(), 24);
    }

    // Compare the ELF structures to their libc counterparts.
    #[cfg(feature = "libc")]
    #[test]
    fn compare_libc() {
        assert_eq!(size_of::<elf32::Ehdr>(), size_of::<libc::Elf32_Ehdr>());
        assert_eq!(size_of::<elf32::Shdr>(), size_of::<libc::Elf32_Shdr>());
        assert_eq!(size_of::<elf32::Phdr>(), size_of::<libc::Elf32_Phdr>());
        assert_eq!(size_of::<elf32::Sym>(), size_of::<libc::Elf32_Sym>());
        assert_eq!(size_of::<elf32::Dyn>(), 8);
        assert_eq!(size_of::<elf32::Rel>(), size_of::<libc::Elf32_Rel>());
        assert_eq!(size_of::<elf32::Rela>(), size_of::<libc::Elf32_Rela>());

        assert_eq!(size_of::<elf64::Ehdr>(), size_of::<libc::Elf64_Ehdr>());
        assert_eq!(size_of::<elf64::Shdr>(), size_of::<libc::Elf64_Shdr>());
        assert_eq!(size_of::<elf64::Phdr>(), size_of::<libc::Elf64_Phdr>());
        assert_eq!(size_of::<elf64::Sym>(), size_of::<libc::Elf64_Sym>());
        assert_eq!(size_of::<elf64::Dyn>(), 16);
        assert_eq!(size_of::<elf64::Rel>(), size_of::<libc::Elf64_Rel>());
        assert_eq!(size_of::<elf64::Rela>(), size_of::<libc::Elf64_Rela>());

        assert_eq!(align_of::<elf32::Ehdr>(), align_of::<libc::Elf32_Ehdr>());
        assert_eq!(align_of::<elf32::Shdr>(), align_of::<libc::Elf32_Shdr>());
        assert_eq!(align_of::<elf32::Phdr>(), align_of::<libc::Elf32_Phdr>());
        assert_eq!(align_of::<elf32::Sym>(), align_of::<libc::Elf32_Sym>());
        assert_eq!(align_of::<elf32::Dyn>(), 4);
        assert_eq!(align_of::<elf32::Rel>(), align_of::<libc::Elf32_Rel>());
        assert_eq!(align_of::<elf32::Rela>(), align_of::<libc::Elf32_Rela>());

        assert_eq!(align_of::<elf64::Ehdr>(), align_of::<libc::Elf64_Ehdr>());
        assert_eq!(align_of::<elf64::Shdr>(), align_of::<libc::Elf64_Shdr>());
        assert_eq!(align_of::<elf64::Phdr>(), align_of::<libc::Elf64_Phdr>());
        assert_eq!(align_of::<elf64::Sym>(), align_of::<libc::Elf64_Sym>());
        assert_eq!(align_of::<elf64::Dyn>(), 8);
        assert_eq!(align_of::<elf64::Rel>(), align_of::<libc::Elf64_Rel>());
        assert_eq!(align_of::<elf64::Rela>(), align_of::<libc::Elf64_Rela>());
    }
}
