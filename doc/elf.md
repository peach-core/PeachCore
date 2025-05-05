# ELF
- Executable and Linkable Forma.  

可执行文件根据数据权限分为代码段, 数据段, 只读数据段, bss段等, elf保存了这些段信息, 以便操作系统加载各个段.

## 一级结构
### ELF Header
固定在 `elf` 文件其实位置, 64字节(对于64位程序), 包含了 `elf` 的总体信息:
``` Rust
pub struct HeaderPt1 {
    pub magic: [u8; 4],
    pub class: Class_,
    pub data: Data_,
    pub version: Version_,
    pub os_abi: OsAbi_,
    // Often also just padding.
    pub abi_version: u8,
    pub padding: [u8; 7],
}

pub struct HeaderPt2_<P> {
    pub type_: Type_,
    pub machine: Machine_,
    pub version: u32,
    pub entry_point: P,
    pub ph_offset: P,
    pub sh_offset: P,
    pub flags: u32,
    pub header_size: u16,
    pub ph_entry_size: u16,
    pub ph_count: u16,
    pub sh_entry_size: u16,
    pub sh_count: u16,
    pub sh_str_index: u16,
}
```
Rust `xmax-elf` 库将其分为两部分  
 `HeaderPt1` 16位, 含义如下  
*0-7 位:*
| magic | class | data |version|os_abi|
|:-:|:-:|:-:|:-:|:-:|
|0 - 3| 4 |5|6|7|
|魔数, 用于标识文件类型| 32/64 位 |大小端|elf版本|操作系统(运行环境)|

*8-15 位:*
保留位, 全为0


`HeaderPt2` 48位, 含义如下  
*16-32 位:*
| type | machine | version |entry_point|
|:-:|:-:|:-:|:-:|
|16 - 17| 18 - 19 |20 - 23|24-32|
|elf文件类型(可执行文件, 动/静态链接库)| cpu属性 |版本号|程序入口虚拟地址|

*32-64 位:*
| ph_offset | sh_offset | flags |header_size|ph_entry_size|ph_count|sh_entry_size|sh_count|sh_str_index|
|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
|32 - 49| 40 - 47 |48 - 51|52-53|54-55|56-57|58-59|60-61|62-63|
|程序头表, 表示各个段的实际数据起始位置| 段表起始位置 |cpu特殊信息|ELF Header 大小(64位为64, 32位为52)|程序头表项的大小|程序头表长度|段表项大小|段表长度|保存段名的字符串表地址|

### program header table(64位)
程序头表保存了各个段代码的信息.
``` Rust
pub struct ProgramHeader64 {
    pub type_: Type_,
    pub flags: Flags,
    pub offset: u64,
    pub virtual_addr: u64,
    pub physical_addr: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub align: u64,
}
```
1. `type`: 段类型(未定义段, 可加载段, 动态链接段等)
2. `flags`: 段属性(可读, 可写, 可执行等)
3. `offset`: 该段的偏移地址
4. `virtual_addr`: 该段加载到内存中的虚拟地址
5. `physical_addr`: 在linux上无效, 裸机程序有效
6. `file_size`: 段在磁盘上的物理大小
7. `mem_size`: 段在内存中的大小(如`.bss`段不占用磁盘空间, 仅占用内存空间)
8. `align`: 段对齐