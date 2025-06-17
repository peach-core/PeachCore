#[rustfmt::skip]
pub mod flag {
    pub const CSIGNAL               : usize = 0x000000ff;   /* Signal mask to be sent at exit.  */
    pub const CLONE_VM              : usize = 0x00000100;   /* Set if VM shared between processes.  */
    pub const CLONE_FS              : usize = 0x00000200;   /* Set if fs info shared between processes.  */
    pub const CLONE_FILES           : usize = 0x00000400;   /* Set if open files shared between processes.  */
    pub const CLONE_SIGHAND         : usize = 0x00000800;   /* Set if signal handlers shared.  */
    pub const CLONE_PIDFD           : usize = 0x00001000;   /* Set if a pidfd should be placed
                                                               in parent.  */
    pub const CLONE_PTRAC           : usize = 0x00002000;   /* Set if tracing continues on the child.  */
    pub const CLONE_VFORK           : usize = 0x00004000;   /* Set if the parent wants the child to
                                                               wake it up on mm_release.  */
    pub const CLONE_PARENT          : usize = 0x00008000;   /* Set if we want to have the same
                                                               parent as the cloner.  */
    pub const CLONE_THREAD          : usize = 0x00010000;   /* Set to add to same thread group.  */
    pub const CLONE_NEWNS           : usize = 0x00020000;   /* Set to create new namespace.  */
    pub const CLONE_SYSVSEM         : usize = 0x00040000;   /* Set to shared SVID SEM_UNDO semantics.  */
    pub const CLONE_SETTLS          : usize = 0x00080000;   /* Set TLS info.  */
    pub const CLONE_PARENT_SETTID   : usize = 0x00100000;   /* Store TID in userlevel buffer
                                                               before MM copy.  */
    pub const CLONE_CHILD_CLEARTID  : usize = 0x00200000;   /* Register exit futex and memory
                                                               location to clear.  */
    pub const CLONE_DETACHED        : usize = 0x00400000;   /* Create clone detached.  */
    pub const CLONE_UNTRACED        : usize = 0x00800000;   /* Set if the tracing process can't
                                                               force CLONE_PTRACE on this clone.  */
    pub const CLONE_CHILD_SETTID    : usize = 0x01000000;   /* Store TID in userlevel buffer in
                                                               the child.  */
    pub const CLONE_NEWCGROUP       : usize = 0x02000000;   /* New cgroup namespace.  */
    pub const CLONE_NEWUT           : usize = 0x04000000;   /* New utsname group.  */
    pub const CLONE_NEWIP           : usize = 0x08000000;   /* New ipcs.  */
    pub const CLONE_NEWUS           : usize = 0x10000000;   /* New user namespace.  */
    pub const CLONE_NEWPI           : usize = 0x20000000;   /* New pid namespace.  */
    pub const CLONE_NEWNE           : usize = 0x40000000;   /* New network namespace.  */
    pub const CLONE_I               : usize = 0x80000000;   /* Clone I/O context.  */
}
