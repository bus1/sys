//! # Tests for the Linux FFI Definitions
//!
//! This module contains tests for all exported FFI definitions of the
//! `ffi::linux` module.
//!
//! Currently, tests are only run if `libc` is enabled. This simplifies the
//! test setup and allows testing against known libc values.

#![cfg(all(test, feature = "libc"))]

use super::*;

// Compare two type definitions for equality.
fn eq_def_type<A, B>() -> bool {
    core::mem::size_of::<A>() == core::mem::size_of::<B>()
    && core::mem::align_of::<A>() == core::mem::align_of::<B>()
}

// A 3-way variant of `eq_def_type()`.
fn eq3_def_type<A, B, C>() -> bool {
    eq_def_type::<A, B>() && eq_def_type::<A, C>()
}

// Compare two `const` definitions for equality. This will compare their type
// layout and memory content for equality.
unsafe fn eq_def_const<A, B>(a: &A, b: &B) -> bool {
    // SAFETY: Propagated to caller.
    unsafe {
        core::mem::size_of::<A>() == core::mem::size_of::<B>()
        && core::mem::align_of::<A>() == core::mem::align_of::<B>()
        && osi::mem::eq(a, b)
    }
}

// A 3-way variant of `eq_def_const()`.
unsafe fn eq3_def_const<A, B, C>(a: &A, b: &B, c: &C) -> bool {
    // SAFETY: Propagated to caller.
    unsafe { eq_def_const(a, b) && eq_def_const(a, c) }
}

// Verify that all supported platforms are available, by simply checking that
// they expose `abi::U16`.
#[test]
fn platform_availability() {
    assert_eq!(core::mem::size_of::<libc::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<aarch64::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<x86::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<x86_64::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<target::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<native::abi::U16>(), 2);
}

// Compare ABIs of target, native, and libc.
#[test]
fn target_abi() {
    assert!(eq3_def_type::<target::abi::I8, native::abi::I8, libc::abi::I8>());
    assert!(eq3_def_type::<target::abi::I16, native::abi::I16, libc::abi::I16>());
    assert!(eq3_def_type::<target::abi::I32, native::abi::I32, libc::abi::I32>());
    assert!(eq3_def_type::<target::abi::I64, native::abi::I64, libc::abi::I64>());
    assert!(eq3_def_type::<target::abi::I128, native::abi::I128, libc::abi::I128>());
    assert!(eq3_def_type::<target::abi::Isize, native::abi::Isize, libc::abi::Isize>());

    assert!(eq3_def_type::<target::abi::U8, native::abi::U8, libc::abi::U8>());
    assert!(eq3_def_type::<target::abi::U16, native::abi::U16, libc::abi::U16>());
    assert!(eq3_def_type::<target::abi::U32, native::abi::U32, libc::abi::U32>());
    assert!(eq3_def_type::<target::abi::U64, native::abi::U64, libc::abi::U64>());
    assert!(eq3_def_type::<target::abi::U128, native::abi::U128, libc::abi::U128>());
    assert!(eq3_def_type::<target::abi::Usize, native::abi::Usize, libc::abi::Usize>());

    assert!(eq3_def_type::<target::abi::F32, native::abi::F32, libc::abi::F32>());
    assert!(eq3_def_type::<target::abi::F64, native::abi::F64, libc::abi::F64>());

    assert!(eq3_def_type::<target::abi::Addr, native::abi::Addr, libc::abi::Addr>());
    assert!(eq3_def_type::<target::abi::Ptr<()>, native::abi::Ptr<()>, libc::abi::Ptr<()>>());

    // `Addr` should match `Usize` and `Ptr<()>` in layout.
    assert!(eq_def_type::<native::abi::Addr, native::abi::Usize>());
    assert!(eq_def_type::<native::abi::Addr, native::abi::Ptr<()>>());
}

// Compare errnos of target, native and libc.
#[test]
fn target_errno() {
    unsafe {
        assert!(eq3_def_const(&target::errno::EPERM, &native::errno::EPERM, &libc::errno::EPERM));
        assert!(eq3_def_const(&target::errno::ENOENT, &native::errno::ENOENT, &libc::errno::ENOENT));
        assert!(eq3_def_const(&target::errno::ESRCH, &native::errno::ESRCH, &libc::errno::ESRCH));
        assert!(eq3_def_const(&target::errno::EINTR, &native::errno::EINTR, &libc::errno::EINTR));
        assert!(eq3_def_const(&target::errno::EIO, &native::errno::EIO, &libc::errno::EIO));
        assert!(eq3_def_const(&target::errno::ENXIO, &native::errno::ENXIO, &libc::errno::ENXIO));
        assert!(eq3_def_const(&target::errno::E2BIG, &native::errno::E2BIG, &libc::errno::E2BIG));
        assert!(eq3_def_const(&target::errno::ENOEXEC, &native::errno::ENOEXEC, &libc::errno::ENOEXEC));
        assert!(eq3_def_const(&target::errno::EBADF, &native::errno::EBADF, &libc::errno::EBADF));
        assert!(eq3_def_const(&target::errno::ECHILD, &native::errno::ECHILD, &libc::errno::ECHILD));
        assert!(eq3_def_const(&target::errno::EAGAIN, &native::errno::EAGAIN, &libc::errno::EAGAIN));
        assert!(eq3_def_const(&target::errno::ENOMEM, &native::errno::ENOMEM, &libc::errno::ENOMEM));
        assert!(eq3_def_const(&target::errno::EACCES, &native::errno::EACCES, &libc::errno::EACCES));
        assert!(eq3_def_const(&target::errno::EFAULT, &native::errno::EFAULT, &libc::errno::EFAULT));
        assert!(eq3_def_const(&target::errno::ENOTBLK, &native::errno::ENOTBLK, &libc::errno::ENOTBLK));
        assert!(eq3_def_const(&target::errno::EBUSY, &native::errno::EBUSY, &libc::errno::EBUSY));
        assert!(eq3_def_const(&target::errno::EEXIST, &native::errno::EEXIST, &libc::errno::EEXIST));
        assert!(eq3_def_const(&target::errno::EXDEV, &native::errno::EXDEV, &libc::errno::EXDEV));
        assert!(eq3_def_const(&target::errno::ENODEV, &native::errno::ENODEV, &libc::errno::ENODEV));
        assert!(eq3_def_const(&target::errno::ENOTDIR, &native::errno::ENOTDIR, &libc::errno::ENOTDIR));
        assert!(eq3_def_const(&target::errno::EISDIR, &native::errno::EISDIR, &libc::errno::EISDIR));
        assert!(eq3_def_const(&target::errno::EINVAL, &native::errno::EINVAL, &libc::errno::EINVAL));
        assert!(eq3_def_const(&target::errno::ENFILE, &native::errno::ENFILE, &libc::errno::ENFILE));
        assert!(eq3_def_const(&target::errno::EMFILE, &native::errno::EMFILE, &libc::errno::EMFILE));
        assert!(eq3_def_const(&target::errno::ENOTTY, &native::errno::ENOTTY, &libc::errno::ENOTTY));
        assert!(eq3_def_const(&target::errno::ETXTBSY, &native::errno::ETXTBSY, &libc::errno::ETXTBSY));
        assert!(eq3_def_const(&target::errno::EFBIG, &native::errno::EFBIG, &libc::errno::EFBIG));
        assert!(eq3_def_const(&target::errno::ENOSPC, &native::errno::ENOSPC, &libc::errno::ENOSPC));
        assert!(eq3_def_const(&target::errno::ESPIPE, &native::errno::ESPIPE, &libc::errno::ESPIPE));
        assert!(eq3_def_const(&target::errno::EROFS, &native::errno::EROFS, &libc::errno::EROFS));
        assert!(eq3_def_const(&target::errno::EMLINK, &native::errno::EMLINK, &libc::errno::EMLINK));
        assert!(eq3_def_const(&target::errno::EPIPE, &native::errno::EPIPE, &libc::errno::EPIPE));
        assert!(eq3_def_const(&target::errno::EDOM, &native::errno::EDOM, &libc::errno::EDOM));
        assert!(eq3_def_const(&target::errno::ERANGE, &native::errno::ERANGE, &libc::errno::ERANGE));

        assert!(eq3_def_const(&target::errno::EDEADLK, &native::errno::EDEADLK, &libc::errno::EDEADLK));
        assert!(eq3_def_const(&target::errno::ENAMETOOLONG, &native::errno::ENAMETOOLONG, &libc::errno::ENAMETOOLONG));
        assert!(eq3_def_const(&target::errno::ENOLCK, &native::errno::ENOLCK, &libc::errno::ENOLCK));
        assert!(eq3_def_const(&target::errno::ENOSYS, &native::errno::ENOSYS, &libc::errno::ENOSYS));
        assert!(eq3_def_const(&target::errno::ENOTEMPTY, &native::errno::ENOTEMPTY, &libc::errno::ENOTEMPTY));
        assert!(eq3_def_const(&target::errno::ELOOP, &native::errno::ELOOP, &libc::errno::ELOOP));
        assert!(eq3_def_const(&target::errno::ENOMSG, &native::errno::ENOMSG, &libc::errno::ENOMSG));
        assert!(eq3_def_const(&target::errno::EIDRM, &native::errno::EIDRM, &libc::errno::EIDRM));
        assert!(eq3_def_const(&target::errno::ECHRNG, &native::errno::ECHRNG, &libc::errno::ECHRNG));
        assert!(eq3_def_const(&target::errno::EL2NSYNC, &native::errno::EL2NSYNC, &libc::errno::EL2NSYNC));
        assert!(eq3_def_const(&target::errno::EL3HLT, &native::errno::EL3HLT, &libc::errno::EL3HLT));
        assert!(eq3_def_const(&target::errno::EL3RST, &native::errno::EL3RST, &libc::errno::EL3RST));
        assert!(eq3_def_const(&target::errno::ELNRNG, &native::errno::ELNRNG, &libc::errno::ELNRNG));
        assert!(eq3_def_const(&target::errno::EUNATCH, &native::errno::EUNATCH, &libc::errno::EUNATCH));
        assert!(eq3_def_const(&target::errno::ENOCSI, &native::errno::ENOCSI, &libc::errno::ENOCSI));
        assert!(eq3_def_const(&target::errno::EL2HLT, &native::errno::EL2HLT, &libc::errno::EL2HLT));
        assert!(eq3_def_const(&target::errno::EBADE, &native::errno::EBADE, &libc::errno::EBADE));
        assert!(eq3_def_const(&target::errno::EBADR, &native::errno::EBADR, &libc::errno::EBADR));
        assert!(eq3_def_const(&target::errno::EXFULL, &native::errno::EXFULL, &libc::errno::EXFULL));
        assert!(eq3_def_const(&target::errno::ENOANO, &native::errno::ENOANO, &libc::errno::ENOANO));
        assert!(eq3_def_const(&target::errno::EBADRQC, &native::errno::EBADRQC, &libc::errno::EBADRQC));
        assert!(eq3_def_const(&target::errno::EBADSLT, &native::errno::EBADSLT, &libc::errno::EBADSLT));
        assert!(eq3_def_const(&target::errno::EBFONT, &native::errno::EBFONT, &libc::errno::EBFONT));
        assert!(eq3_def_const(&target::errno::ENOSTR, &native::errno::ENOSTR, &libc::errno::ENOSTR));
        assert!(eq3_def_const(&target::errno::ENODATA, &native::errno::ENODATA, &libc::errno::ENODATA));
        assert!(eq3_def_const(&target::errno::ETIME, &native::errno::ETIME, &libc::errno::ETIME));
        assert!(eq3_def_const(&target::errno::ENOSR, &native::errno::ENOSR, &libc::errno::ENOSR));
        assert!(eq3_def_const(&target::errno::ENONET, &native::errno::ENONET, &libc::errno::ENONET));
        assert!(eq3_def_const(&target::errno::ENOPKG, &native::errno::ENOPKG, &libc::errno::ENOPKG));
        assert!(eq3_def_const(&target::errno::EREMOTE, &native::errno::EREMOTE, &libc::errno::EREMOTE));
        assert!(eq3_def_const(&target::errno::ENOLINK, &native::errno::ENOLINK, &libc::errno::ENOLINK));
        assert!(eq3_def_const(&target::errno::EADV, &native::errno::EADV, &libc::errno::EADV));
        assert!(eq3_def_const(&target::errno::ESRMNT, &native::errno::ESRMNT, &libc::errno::ESRMNT));
        assert!(eq3_def_const(&target::errno::ECOMM, &native::errno::ECOMM, &libc::errno::ECOMM));
        assert!(eq3_def_const(&target::errno::EPROTO, &native::errno::EPROTO, &libc::errno::EPROTO));
        assert!(eq3_def_const(&target::errno::EMULTIHOP, &native::errno::EMULTIHOP, &libc::errno::EMULTIHOP));
        assert!(eq3_def_const(&target::errno::EDOTDOT, &native::errno::EDOTDOT, &libc::errno::EDOTDOT));
        assert!(eq3_def_const(&target::errno::EBADMSG, &native::errno::EBADMSG, &libc::errno::EBADMSG));
        assert!(eq3_def_const(&target::errno::EOVERFLOW, &native::errno::EOVERFLOW, &libc::errno::EOVERFLOW));
        assert!(eq3_def_const(&target::errno::ENOTUNIQ, &native::errno::ENOTUNIQ, &libc::errno::ENOTUNIQ));
        assert!(eq3_def_const(&target::errno::EBADFD, &native::errno::EBADFD, &libc::errno::EBADFD));
        assert!(eq3_def_const(&target::errno::EREMCHG, &native::errno::EREMCHG, &libc::errno::EREMCHG));
        assert!(eq3_def_const(&target::errno::ELIBACC, &native::errno::ELIBACC, &libc::errno::ELIBACC));
        assert!(eq3_def_const(&target::errno::ELIBBAD, &native::errno::ELIBBAD, &libc::errno::ELIBBAD));
        assert!(eq3_def_const(&target::errno::ELIBSCN, &native::errno::ELIBSCN, &libc::errno::ELIBSCN));
        assert!(eq3_def_const(&target::errno::ELIBMAX, &native::errno::ELIBMAX, &libc::errno::ELIBMAX));
        assert!(eq3_def_const(&target::errno::ELIBEXEC, &native::errno::ELIBEXEC, &libc::errno::ELIBEXEC));
        assert!(eq3_def_const(&target::errno::EILSEQ, &native::errno::EILSEQ, &libc::errno::EILSEQ));
        assert!(eq3_def_const(&target::errno::ERESTART, &native::errno::ERESTART, &libc::errno::ERESTART));
        assert!(eq3_def_const(&target::errno::ESTRPIPE, &native::errno::ESTRPIPE, &libc::errno::ESTRPIPE));
        assert!(eq3_def_const(&target::errno::EUSERS, &native::errno::EUSERS, &libc::errno::EUSERS));
        assert!(eq3_def_const(&target::errno::ENOTSOCK, &native::errno::ENOTSOCK, &libc::errno::ENOTSOCK));
        assert!(eq3_def_const(&target::errno::EDESTADDRREQ, &native::errno::EDESTADDRREQ, &libc::errno::EDESTADDRREQ));
        assert!(eq3_def_const(&target::errno::EMSGSIZE, &native::errno::EMSGSIZE, &libc::errno::EMSGSIZE));
        assert!(eq3_def_const(&target::errno::EPROTOTYPE, &native::errno::EPROTOTYPE, &libc::errno::EPROTOTYPE));
        assert!(eq3_def_const(&target::errno::ENOPROTOOPT, &native::errno::ENOPROTOOPT, &libc::errno::ENOPROTOOPT));
        assert!(eq3_def_const(&target::errno::EPROTONOSUPPORT, &native::errno::EPROTONOSUPPORT, &libc::errno::EPROTONOSUPPORT));
        assert!(eq3_def_const(&target::errno::ESOCKTNOSUPPORT, &native::errno::ESOCKTNOSUPPORT, &libc::errno::ESOCKTNOSUPPORT));
        assert!(eq3_def_const(&target::errno::EOPNOTSUPP, &native::errno::EOPNOTSUPP, &libc::errno::EOPNOTSUPP));
        assert!(eq3_def_const(&target::errno::EPFNOSUPPORT, &native::errno::EPFNOSUPPORT, &libc::errno::EPFNOSUPPORT));
        assert!(eq3_def_const(&target::errno::EAFNOSUPPORT, &native::errno::EAFNOSUPPORT, &libc::errno::EAFNOSUPPORT));
        assert!(eq3_def_const(&target::errno::EADDRINUSE, &native::errno::EADDRINUSE, &libc::errno::EADDRINUSE));
        assert!(eq3_def_const(&target::errno::EADDRNOTAVAIL, &native::errno::EADDRNOTAVAIL, &libc::errno::EADDRNOTAVAIL));
        assert!(eq3_def_const(&target::errno::ENETDOWN, &native::errno::ENETDOWN, &libc::errno::ENETDOWN));
        assert!(eq3_def_const(&target::errno::ENETUNREACH, &native::errno::ENETUNREACH, &libc::errno::ENETUNREACH));
        assert!(eq3_def_const(&target::errno::ENETRESET, &native::errno::ENETRESET, &libc::errno::ENETRESET));
        assert!(eq3_def_const(&target::errno::ECONNABORTED, &native::errno::ECONNABORTED, &libc::errno::ECONNABORTED));
        assert!(eq3_def_const(&target::errno::ECONNRESET, &native::errno::ECONNRESET, &libc::errno::ECONNRESET));
        assert!(eq3_def_const(&target::errno::ENOBUFS, &native::errno::ENOBUFS, &libc::errno::ENOBUFS));
        assert!(eq3_def_const(&target::errno::EISCONN, &native::errno::EISCONN, &libc::errno::EISCONN));
        assert!(eq3_def_const(&target::errno::ENOTCONN, &native::errno::ENOTCONN, &libc::errno::ENOTCONN));
        assert!(eq3_def_const(&target::errno::ESHUTDOWN, &native::errno::ESHUTDOWN, &libc::errno::ESHUTDOWN));
        assert!(eq3_def_const(&target::errno::ETOOMANYREFS, &native::errno::ETOOMANYREFS, &libc::errno::ETOOMANYREFS));
        assert!(eq3_def_const(&target::errno::ETIMEDOUT, &native::errno::ETIMEDOUT, &libc::errno::ETIMEDOUT));
        assert!(eq3_def_const(&target::errno::ECONNREFUSED, &native::errno::ECONNREFUSED, &libc::errno::ECONNREFUSED));
        assert!(eq3_def_const(&target::errno::EHOSTDOWN, &native::errno::EHOSTDOWN, &libc::errno::EHOSTDOWN));
        assert!(eq3_def_const(&target::errno::EHOSTUNREACH, &native::errno::EHOSTUNREACH, &libc::errno::EHOSTUNREACH));
        assert!(eq3_def_const(&target::errno::EALREADY, &native::errno::EALREADY, &libc::errno::EALREADY));
        assert!(eq3_def_const(&target::errno::EINPROGRESS, &native::errno::EINPROGRESS, &libc::errno::EINPROGRESS));
        assert!(eq3_def_const(&target::errno::ESTALE, &native::errno::ESTALE, &libc::errno::ESTALE));
        assert!(eq3_def_const(&target::errno::EUCLEAN, &native::errno::EUCLEAN, &libc::errno::EUCLEAN));
        assert!(eq3_def_const(&target::errno::ENOTNAM, &native::errno::ENOTNAM, &libc::errno::ENOTNAM));
        assert!(eq3_def_const(&target::errno::ENAVAIL, &native::errno::ENAVAIL, &libc::errno::ENAVAIL));
        assert!(eq3_def_const(&target::errno::EISNAM, &native::errno::EISNAM, &libc::errno::EISNAM));
        assert!(eq3_def_const(&target::errno::EREMOTEIO, &native::errno::EREMOTEIO, &libc::errno::EREMOTEIO));
        assert!(eq3_def_const(&target::errno::EDQUOT, &native::errno::EDQUOT, &libc::errno::EDQUOT));
        assert!(eq3_def_const(&target::errno::ENOMEDIUM, &native::errno::ENOMEDIUM, &libc::errno::ENOMEDIUM));
        assert!(eq3_def_const(&target::errno::EMEDIUMTYPE, &native::errno::EMEDIUMTYPE, &libc::errno::EMEDIUMTYPE));
        assert!(eq3_def_const(&target::errno::ECANCELED, &native::errno::ECANCELED, &libc::errno::ECANCELED));
        assert!(eq3_def_const(&target::errno::ENOKEY, &native::errno::ENOKEY, &libc::errno::ENOKEY));
        assert!(eq3_def_const(&target::errno::EKEYEXPIRED, &native::errno::EKEYEXPIRED, &libc::errno::EKEYEXPIRED));
        assert!(eq3_def_const(&target::errno::EKEYREVOKED, &native::errno::EKEYREVOKED, &libc::errno::EKEYREVOKED));
        assert!(eq3_def_const(&target::errno::EKEYREJECTED, &native::errno::EKEYREJECTED, &libc::errno::EKEYREJECTED));
        assert!(eq3_def_const(&target::errno::EOWNERDEAD, &native::errno::EOWNERDEAD, &libc::errno::EOWNERDEAD));
        assert!(eq3_def_const(&target::errno::ENOTRECOVERABLE, &native::errno::ENOTRECOVERABLE, &libc::errno::ENOTRECOVERABLE));
        assert!(eq3_def_const(&target::errno::ERFKILL, &native::errno::ERFKILL, &libc::errno::ERFKILL));
        assert!(eq3_def_const(&target::errno::EHWPOISON, &native::errno::EHWPOISON, &libc::errno::EHWPOISON));

        assert!(eq3_def_const(&target::errno::EWOULDBLOCK, &native::errno::EWOULDBLOCK, &libc::errno::EWOULDBLOCK));
        assert!(eq3_def_const(&target::errno::EDEADLOCK, &native::errno::EDEADLOCK, &libc::errno::EDEADLOCK));
    }
}
