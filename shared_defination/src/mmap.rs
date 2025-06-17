#[rustfmt::skip]
#[allow(unused)]
pub mod flag {
    const PROT_READ:        usize = 0x1;        /* Page can be read.  */
    const PROT_WRITE:       usize = 0x2;        /* Page can be written.  */
    const PROT_EXEC:        usize = 0x4;        /* Page can be executed.  */
    const PROT_NONE:        usize = 0x0;        /* Page can not be accessed.  */
    const PROT_GROWSDOWN:   usize = 0x01000000; /* Extend change to start of
                                                   growsdown vma (mprotect only).  */
    const PROT_GROWSUP:     usize = 0x02000000; /* Extend change to start of
                                                   growsup vma (mprotect only).  */

    /* Sharing types (must choose one and only one of these).  */
    const MAP_SHARED:       usize = 0x01;       /* Share changes.  */
    const MAP_PRIVATE:      usize = 0x02;       /* Changes are private.  */

    /* Other flags.  */
    const MAP_FIXED:        usize = 0x10;       /* Interpret addr exactly.  */
    const MAP_FILE:         usize = 0;
    const MAP_ANONYMOUS:    usize = 0x20;       /* Don't use a file.  */
    const MAP_ANON:         usize = MAP_ANONYMOUS;
    /* When MAP_HUGETLB is set bits [26:31] encode the log2 of the huge page size.  */
    const MAP_HUGE_SHIFT:   usize = 26;
    const MAP_HUGE_MASK:    usize = 0x3f;
}
