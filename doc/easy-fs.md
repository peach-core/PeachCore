# Easy File System (EFS)

## 文件系统的功能

### VFS
对于磁盘来说, 基本结构是块, 而对于操作系统来说, 基本单元是文件. 因此, 为了方便操作系统管理磁盘设备. 需要抽象出一层 虚拟文件系统层 (VFS), VFS 下层为磁盘驱动, 上层为操作系统的文件操作 (file_operations).

### 缓存
为了加速对磁盘的访问, VFS 提供对磁盘块的缓存. 控制缓存的交换和写回, 由于所有上层的访问申请都通过文件系统, 因此, 不存在缓存一致性问题, 对于一个磁盘的物理块, 至多存在一个缓存块. 缓存提供 `get_ref`, `get_mut`, `modify`, `read`, `sync` 接口.

## 磁盘空间管理

### 位图
磁盘硬件本身不知道自己哪些块被使用了, 因此需要一个位图来记录每个块的状态.

### 块资源的组织
如内存管理一样, 如果采用连续空间管理, 会产生内存碎片, 回收碎片需要移动数据位置, 并反向修改受影响的文件信息(记录了对应的数据块号). 因此我们采用类似页表的多级索引表方式管理, 对于少量数据, 我们存在直接索引表中, 对于大量的数据, 我们依次存到一级索引和二级索引中. 这么做是为了方便扩容, 当文件大于直接索引能管理的范围时, 我们不需要修改直接索引表的内容, 只需要将超出的部分, 写到一级索引表中即可.

### inode域
磁盘的索引为块, 文件的索引为 inode 和文件名. 为了能通过 inode 找到 对应的物理磁盘块, 我们需要在磁盘中保存一片区域用于保存 每个 `inode` 的信息. 包含该 文件大小, 该文件数据所在的 数据块位置 和 该文件类型(可以为文件和目录).

``` rust
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_MAX_SIZE],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}
```

`direct`, `indirect1`, `indirect2` 为索引表. `DiskInode` 大小按 128 字节对齐, 存在 `Inode` 域中, `indirect` 存在 `data` 域中. 这样通过 `Inode` 域的大小就能确定, 该文件系统最多能包含多少个文件(和目录). `indirect` 可以动态分配, 不影响 `Inode` 数量.  

`Inode` 和 `data` 域大小在文件系统初始化(格式化) 的时候就确定了, 并写入 超级块中(0号块, 记录文件系统信息). `Inode` 和 `data` 域中也分别有 `bitmap` 用于管理空闲的 `Inode` 和 空闲的数据块.

### 资源分配
由 `vfs manager` 管理磁盘, `manager` 通过 `open` 方法获取文件系统的信息 (如 `bitmap` 位置), 分配 `inode` 时, `vfs` 向 `struct bitmap` 申请获取一个空的 `inode`, `bitmap` 遍历所有位, 返回一个为 `0` 的位的 索引值.
`vfs` 再通过这个索引值, 在 `Inode Area` 中找到对应的 `inode`.  

分配 `data block` 时同理, 只不过 `data block` 的大小和 `inode` 不同.

- 注意, `vfs` 不能向上层提供 `inode` 在缓存中的一个引用, 因为一个进程拿到引用后, 可能被换出CPU, 此时, 另一个进程有可能导致该 `inode` 所在的块被换出缓存. 除非我们单独定义设计 `inode` 引用类型, 改引用类型需要通过 `RAII` 管理该 `inode` 所在块的共享所有权 (Arc), 但是 这么做会增加我们设计的复杂度. 因此, 我们只返回 `inode` 所在的块和偏移, 这样通过 `get_block_cache` 方法获取到的一定是正确值.

## 缓存

### 缓存管理器
缓存管理器用于控制缓存的交换, 通过某种数据结构将缓存组织起来方便换出操作, 显而易见, 该数据结构属于临界资源, 因此, 对于缓存管理器的操作均需要互斥访问.  

方案, 给全局缓存管理器加一个 `Mutex`.

### 缓存块
一个进程正在写缓存块时, 不允许别的块对这个块进行 读/写, 允许多个块同时读. 方便起见, 我们多读写操作均做互斥. 即对每个缓存块加锁后访问, 被加锁的块不允许被操作(也不允许被换出内存).


``` c
struct critical_resource_t {
    spinlock_t lock;
    /* member */
};
```

``` rust
struct CriticalResource {
    type Inner = CriticalResourceInner,
    inner: Arc<Mutex<Inner>>,
}

struct CriticalResourceInner {
    /* member */
}

impl CriticalResource {
    pub fn new(/*args*/) -> Self {
        Arc::new(Mutex::new(CriticalResourceInner::new(/*args*/)))
    }
}

impl CriticalResourceInner {
    pub fn new(/*args*/) -> Self;
}
```

``` rust
struct CriticalResource {
    /* member */
}

impl CriticalResource {
    pub fn create(/*args*/) -> Arc<Mutex<Self>> {
        let inner = Self {
            /*args*/
        };

        Arc::new(Mutex::new(inner))
    }
}
```

## Layout

### 驻留于磁盘的结构

#### `DiskInode`
用于表示一个 驻留在磁盘上的 `inode` 信息, 包括 大小, 包含的数据块号, inode 类型(文件/文件夹). 一般用在 将一个磁盘缓存(位于内存中)地址 转换成 `DiskInode&` 用于获取 `inode` 信息.
```rust
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_MAX_SIZE],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}
```

#### DirEntry
`DiskInode` 分为两种, 其中 `DirInode` 的数据块包含的信息为 `DirEntry`(目录成员的 `inode` 入口信息), 其组织形式如下. 也同样用在读取磁盘时, 将某一个磁盘块的缓存的某个偏移地址转换为 `DirEntry&` 方便读写.
```rust
#[repr(C)]
pub struct DirEntry {
    name: [u8; FILE_NAME_LEN_LIMIT + 1],
    inode_number: u32,
}
```


### 驻留于磁盘的结构

#### `Inode`
用于表示一个磁盘上的 `Inode`, 由于 `DiskInode` 驻留在磁盘中, 而且不同文件系统的 `DiskInode` 格式不同, 因此, 一方面我们需要一个在内存中的数据结构用于间接表示 `DiskInode`, 另一方面, 将操作系统实现与文件系统隔离开.   
`Inode` 记录了 对应 `DiskInode` 采用的文件系统和块设备指针(文件系统的设计和块设备无关, 因此两者都需要在 `Inode` 中记录), 同时还有对应的 `DiskInode` 在文件系统中的位置(块号, 块内偏移). 有了这四个数据, 我们就可以读取 `DiskInode` 信息了.

#### `OSInode`
`OSInode` 是操作系统管理文件的最小单元, 即操作系统视角下的文件(`OSInode` 实现了 `File Trait`), 包含 `Inode` 信息, 读写权限, 当前文件的偏移指针(seek, read, write等系统调用依赖该指针).

#### `File Trait`
类比于 `linux kernel` 的 `FileOperations`, 是一个泛化的文件接口, 对于一切 `Unix` 世界的东西, 都可以通过这个接口表示.
