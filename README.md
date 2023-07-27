# 笔记本
## ------------ 引导启动 ------------
当启动系统时, 主板ROM内 存储的固件将会运行: 它将负责电脑的加电自检, 可用内存的检测, 以及CPU和其他硬件的预加载
之后, 它将寻找一个 "可引导的存储介质", 并开始引导启动其中的 "内核"

x86 架构支持两种固件标准: BIOS 和 UEFI

+ BIOS 标准显得陈旧而过时，但实现简单，并为 1980 年代后的所有 x86 设备所支持
+ 相反, UEFI 更加现代化, 功能也更全面, 但开发和构建更复杂

> 目前先以BIOS固件的引导启动方式

## ------------ 引导启动 ------------

### ------------ BIOS启动 ------------
几乎所有的 x86 硬件系统都支持 BIOS 启动，这也包含新型的、基于 UEFI、用模拟 BIOS（emulated BIOS）的方式向后兼容的硬件系统。

对开发人员来说这是极好的, 因为无论是上世纪还是现在的硬件系统, 都只需要编写相同的引导启动逻辑

但是这也带来了最大的缺点, 意味着在系统启动前, CPU必须先进入一个16位系统兼容的 "实模式"(real mode)

### 启动过程
1. 加载主板闪存中存储的BIOS固件
2. BIOS固件加电自检、初始化硬件
3. 固件寻找可引导的存储介质
4. if 引导介质存在 -> 引导程序(bootloader: 一段存储在存储介质开头的、512字节长度的程序片段)控制cpu
    - 目前大多数 bootloader 都已经大于512byte了, 但是会进行切片, 将优先级最高的片段优先启动, 且体积控制到512byte
    - 首先启动存储在介质开头的 第一段引导介质(first stage bootloader)
    - 第一段引导介质加载其他位置的 第二段引导程序(second stage bootloader)

引导程序必须决定内核的位置，并将内核加载到内存. 同时需要将CPU从16位的实模式, 切换到32位的"保护模式(protected mode)" 最终切换到 64位"长模式(long mode)"
到了长模式, 所有的64位寄存器和整个主内存(main memory)才能被访问。

同时引导程序还需要从BIOS查询特定的信息, 并将其传递到内核; 如查询和传递内存映射表

编写一个引导程序并不是一个简单的任务, 因为需要使用到汇编语言, 而且需要经过许多意图并不明显的步骤, 比如将 魔数 写入某个寄存器

一般情况下会使用 bootimage 工具, 能够自动并且方便的为内核准备一个引导程序

## Multiboot

由于操作系统引导程序的混乱, 导致通用性极差, 因此1995年 自由软件基金会 颁布了开源的引导程序标准 -- Multiboot
标准 定义了 bootloader 和 os之间的统一接口, 所以任何适配 Multiboot 的引导程序, 都能用来加载任何同样适配了 Multiboot 的操作系统。

编写一款适配 Multiboot 的内核, 只需要在内核文件开头, 插入被称作 "Multiboot头" 的数据片段, 这让 GRUB 很容易引导任何操作系统。

但是 GRUB 和 Multiboot 标准也有一些可预知的问题:

1. 只支持32位的保护模式。意味着鹅仔引导之后, 依然需要配置CPU, 让它切换到64位长模式
2. 他们被设计为精简引导程序, 但不是精简内核
   - 内核额需要以调整过的"默认页长度(default page size)"被链接, 否则 GRUB 将无法找到内核的 Multiboot 头
   - 另外, 引导信息(boot information)中包含大量与架构有关的数据, 会在引导启动时, 被直接传到操作系统, 而不会经过一层清晰的抽象。
3. GRUB和Multiboot 标准并没有详细的解释, 阅读文档困难
4. 为了创建一个能够被引导的磁盘映像, 开发时必须安装 GRUB。 加大了基于Windows或者macos开发内核的难度


## 内存相关函数

目前来说, Rust编译器假定所有内置函数(built-in functions)在所有系统内都是存在且可用的。

事实上这个前提只对了一半, 绝大多数内置函数都可以被 `compiler_builtins` 提供, 而这个 create 刚刚已经被重新编译过了, 然而部分内存相关函数是需要操作系统相关的标准C库提供的。

比如, `memset`(该函数可以为一个内存块内的所有比特进行赋值)、`memcpy`(将一个内存块里的数据拷贝到另一个内存块)以及`memcmp`(比较两个内存块的数据)

好在现在的小内核不需要这些函数, 但当编写数据更加丰富的功能时(比如拷贝数据结构)时就会用到了


现在无法提供操作系统相关的标准C库, 所以需要使用其他办法来提供这些东西。
一个显而易见的途径就是自己实现 `memset`这些函数, 但是不要忘了加上`#![no_mangle]` 语句, 避免编译时被自动重命名。
当然, 这样做很危险, 底层函数中最细微的错误也会将程序导向不可预知的未来。比如, 可能在实现 `memcpy`时使用了一个 for循环, 然后for循环本身又会调用 `IntoIterator::into_iter`这个`trait`的方法
而这个方法又会继续调用`memcpy`, 此时一个无限递归就产生了, 所以还是使用经过良好测试的既存实现更加可靠。

不过`compiler_builtins`自带了所有相关函数的实现, 只是在默认情况下, 由于避免和标准C库发生冲突的考量被禁用了
此时需要将 `build-std-features`配置项设置为`["compiler_builtins-mem"]`来启用这个特性。如同`build-std`配置项一样, 该特性可以使用 `-Z`参数启用。
也可以在`.cargo/config.toml`中使用`unstable`配置集启用。

## 向屏幕打印字符串

要做到这一步, 最简单的方式是写入 `VGA字符缓冲区(VGA Text buffer)`: 这是一段映射到VGA硬件的特殊内存片段, 包含着显示在屏幕上的内容。通常情况下, 它能够存储25行、80列共2000个`字符单元`
每个`字符单元`能够显示一个`ASCII`字符, 也能够设置这个字符的`前景色`和`背景色`

这段缓冲区的地址是: 0xb8000, 每个字符单元包含一个ASCII码字节和一个颜色字节

## 启动内核

## 创建引导镜像

1. 直接使用现有的 `bootloader`包
2. 将内核编译完成之后, 将内核和引导程序组合在一起
3. 为了组合内核与引导程序, 建议使用 `bootimage` 工具, 用于在内核编译完成后, 将它和引导程序组合在一起, 最终创建一个能够引导的磁盘映像
4. 安装 `bootimage`: `cargo install bootimage`
5. 运行`bootimage`以及编译引导程序, 需要安装 rustup模块 `llvm-tools-preview`: `rustup component add llvm-tools-preview`
6. 使用 bootimage 创建磁盘映像: `cargo bootimage`
7. 最终会将内核和引导程序组合成一个可引导的磁盘映像

### bootimage 工具执行了三个步骤:

1. 编译我们的内核为一个`ELF(Executable and Linkable Format)`文件
2. 编译引导程序为独立的可执行文件
3. 将内核ELF文件`按字节拼接(append by bytes)`到引导程序的末端

当机器启动时, 引导程序将会读取并解析拼接在其后的ELF文件, 这之后, 它将把程序片段映射到`分页表(page table)`中的`虚拟地址(virtual address)`, 清零 `BSS段(BSS segment)`

同时还将创建一个栈。

最终它将读取`入口点地址(entry point address)` ———— 我们程序中`_start`函数的位置, 并跳转到这个位置

## 在QEMU中启动内核