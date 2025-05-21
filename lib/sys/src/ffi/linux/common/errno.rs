//! # Error Codes
//!
//! Linux follows the UNIX tradition in using integer error codes across all
//! its interfaces. These are untyped and can be returned by any code in the
//! kernel. Some have special meaning you can rely upon, yet this is usually
//! not enforced by the kernel in any way (with few exceptions).
//!
//! Kernel syscalls return error codes in its return value. The `errno` concept
//! is not used (it is merely emulated by the C Standard Library). The range of
//! error codes is limited to `[1-4096]` (inclusive; note that `0` is not a
//! valid error code). For this reason, we use `u16` as underlying data-type.
//! Virtually all syscalls return error codes as negative values `-1` to
//! `-4096`. Any return value outside this range is not considered an error but
//! a successful syscall invocation.
//!
//! *Note*: This representation is compatible with pointer values. This is
//! because the kernel never returns pointers for the highest memory maps, as
//! those are reserved for kernel-internal pages, anyway. This means, even
//! syscalls that return pointers will use this scheme.

use super::abi;

// Base error codes
pub const EPERM: abi::U16 = abi::num(1);
pub const ENOENT: abi::U16 = abi::num(2);
pub const ESRCH: abi::U16 = abi::num(3);
pub const EINTR: abi::U16 = abi::num(4);
pub const EIO: abi::U16 = abi::num(5);
pub const ENXIO: abi::U16 = abi::num(6);
pub const E2BIG: abi::U16 = abi::num(7);
pub const ENOEXEC: abi::U16 = abi::num(8);
pub const EBADF: abi::U16 = abi::num(9);
pub const ECHILD: abi::U16 = abi::num(10);
pub const EAGAIN: abi::U16 = abi::num(11);
pub const ENOMEM: abi::U16 = abi::num(12);
pub const EACCES: abi::U16 = abi::num(13);
pub const EFAULT: abi::U16 = abi::num(14);
pub const ENOTBLK: abi::U16 = abi::num(15);
pub const EBUSY: abi::U16 = abi::num(16);
pub const EEXIST: abi::U16 = abi::num(17);
pub const EXDEV: abi::U16 = abi::num(18);
pub const ENODEV: abi::U16 = abi::num(19);
pub const ENOTDIR: abi::U16 = abi::num(20);
pub const EISDIR: abi::U16 = abi::num(21);
pub const EINVAL: abi::U16 = abi::num(22);
pub const ENFILE: abi::U16 = abi::num(23);
pub const EMFILE: abi::U16 = abi::num(24);
pub const ENOTTY: abi::U16 = abi::num(25);
pub const ETXTBSY: abi::U16 = abi::num(26);
pub const EFBIG: abi::U16 = abi::num(27);
pub const ENOSPC: abi::U16 = abi::num(28);
pub const ESPIPE: abi::U16 = abi::num(29);
pub const EROFS: abi::U16 = abi::num(30);
pub const EMLINK: abi::U16 = abi::num(31);
pub const EPIPE: abi::U16 = abi::num(32);
pub const EDOM: abi::U16 = abi::num(33);
pub const ERANGE: abi::U16 = abi::num(34);

// Extended error codes
pub const EDEADLK: abi::U16 = abi::num(35);
pub const ENAMETOOLONG: abi::U16 = abi::num(36);
pub const ENOLCK: abi::U16 = abi::num(37);
pub const ENOSYS: abi::U16 = abi::num(38);
pub const ENOTEMPTY: abi::U16 = abi::num(39);
pub const ELOOP: abi::U16 = abi::num(40);
// 41 is unused (EWOULDBLOCK on UNIX).
pub const ENOMSG: abi::U16 = abi::num(42);
pub const EIDRM: abi::U16 = abi::num(43);
pub const ECHRNG: abi::U16 = abi::num(44);
pub const EL2NSYNC: abi::U16 = abi::num(45);
pub const EL3HLT: abi::U16 = abi::num(46);
pub const EL3RST: abi::U16 = abi::num(47);
pub const ELNRNG: abi::U16 = abi::num(48);
pub const EUNATCH: abi::U16 = abi::num(49);
pub const ENOCSI: abi::U16 = abi::num(50);
pub const EL2HLT: abi::U16 = abi::num(51);
pub const EBADE: abi::U16 = abi::num(52);
pub const EBADR: abi::U16 = abi::num(53);
pub const EXFULL: abi::U16 = abi::num(54);
pub const ENOANO: abi::U16 = abi::num(55);
pub const EBADRQC: abi::U16 = abi::num(56);
pub const EBADSLT: abi::U16 = abi::num(57);
// 58 is unused (EDEADLOCK on UNIX).
pub const EBFONT: abi::U16 = abi::num(59);
pub const ENOSTR: abi::U16 = abi::num(60);
pub const ENODATA: abi::U16 = abi::num(61);
pub const ETIME: abi::U16 = abi::num(62);
pub const ENOSR: abi::U16 = abi::num(63);
pub const ENONET: abi::U16 = abi::num(64);
pub const ENOPKG: abi::U16 = abi::num(65);
pub const EREMOTE: abi::U16 = abi::num(66);
pub const ENOLINK: abi::U16 = abi::num(67);
pub const EADV: abi::U16 = abi::num(68);
pub const ESRMNT: abi::U16 = abi::num(69);
pub const ECOMM: abi::U16 = abi::num(70);
pub const EPROTO: abi::U16 = abi::num(71);
pub const EMULTIHOP: abi::U16 = abi::num(72);
pub const EDOTDOT: abi::U16 = abi::num(73);
pub const EBADMSG: abi::U16 = abi::num(74);
pub const EOVERFLOW: abi::U16 = abi::num(75);
pub const ENOTUNIQ: abi::U16 = abi::num(76);
pub const EBADFD: abi::U16 = abi::num(77);
pub const EREMCHG: abi::U16 = abi::num(78);
pub const ELIBACC: abi::U16 = abi::num(79);
pub const ELIBBAD: abi::U16 = abi::num(80);
pub const ELIBSCN: abi::U16 = abi::num(81);
pub const ELIBMAX: abi::U16 = abi::num(82);
pub const ELIBEXEC: abi::U16 = abi::num(83);
pub const EILSEQ: abi::U16 = abi::num(84);
pub const ERESTART: abi::U16 = abi::num(85);
pub const ESTRPIPE: abi::U16 = abi::num(86);
pub const EUSERS: abi::U16 = abi::num(87);
pub const ENOTSOCK: abi::U16 = abi::num(88);
pub const EDESTADDRREQ: abi::U16 = abi::num(89);
pub const EMSGSIZE: abi::U16 = abi::num(90);
pub const EPROTOTYPE: abi::U16 = abi::num(91);
pub const ENOPROTOOPT: abi::U16 = abi::num(92);
pub const EPROTONOSUPPORT: abi::U16 = abi::num(93);
pub const ESOCKTNOSUPPORT: abi::U16 = abi::num(94);
pub const EOPNOTSUPP: abi::U16 = abi::num(95);
pub const EPFNOSUPPORT: abi::U16 = abi::num(96);
pub const EAFNOSUPPORT: abi::U16 = abi::num(97);
pub const EADDRINUSE: abi::U16 = abi::num(98);
pub const EADDRNOTAVAIL: abi::U16 = abi::num(99);
pub const ENETDOWN: abi::U16 = abi::num(100);
pub const ENETUNREACH: abi::U16 = abi::num(101);
pub const ENETRESET: abi::U16 = abi::num(102);
pub const ECONNABORTED: abi::U16 = abi::num(103);
pub const ECONNRESET: abi::U16 = abi::num(104);
pub const ENOBUFS: abi::U16 = abi::num(105);
pub const EISCONN: abi::U16 = abi::num(106);
pub const ENOTCONN: abi::U16 = abi::num(107);
pub const ESHUTDOWN: abi::U16 = abi::num(108);
pub const ETOOMANYREFS: abi::U16 = abi::num(109);
pub const ETIMEDOUT: abi::U16 = abi::num(110);
pub const ECONNREFUSED: abi::U16 = abi::num(111);
pub const EHOSTDOWN: abi::U16 = abi::num(112);
pub const EHOSTUNREACH: abi::U16 = abi::num(113);
pub const EALREADY: abi::U16 = abi::num(114);
pub const EINPROGRESS: abi::U16 = abi::num(115);
pub const ESTALE: abi::U16 = abi::num(116);
pub const EUCLEAN: abi::U16 = abi::num(117);
pub const ENOTNAM: abi::U16 = abi::num(118);
pub const ENAVAIL: abi::U16 = abi::num(119);
pub const EISNAM: abi::U16 = abi::num(120);
pub const EREMOTEIO: abi::U16 = abi::num(121);
pub const EDQUOT: abi::U16 = abi::num(122);
pub const ENOMEDIUM: abi::U16 = abi::num(123);
pub const EMEDIUMTYPE: abi::U16 = abi::num(124);
pub const ECANCELED: abi::U16 = abi::num(125);
pub const ENOKEY: abi::U16 = abi::num(126);
pub const EKEYEXPIRED: abi::U16 = abi::num(127);
pub const EKEYREVOKED: abi::U16 = abi::num(128);
pub const EKEYREJECTED: abi::U16 = abi::num(129);
pub const EOWNERDEAD: abi::U16 = abi::num(130);
pub const ENOTRECOVERABLE: abi::U16 = abi::num(131);
pub const ERFKILL: abi::U16 = abi::num(132);
pub const EHWPOISON: abi::U16 = abi::num(133);

// Aliases to the canonical linux codes.
pub const EWOULDBLOCK: abi::U16 = abi::num(11); // EAGAIN
pub const EDEADLOCK: abi::U16 = abi::num(35); // EDEADLCK
