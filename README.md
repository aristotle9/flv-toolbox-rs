flv-toolbox-rs
==============

flv 文件切片工具集合

### 安装

本工具使用 Rust 语言编写。安装好最新的 Rust 编译工具后，在目录下运行以下命令：

`cargo build --release`

编译完成后，可运行的程序将出现在 `./target/release` 目录下。

### flv-info

##### flv 文件信息查看

示例:

`flv-info file.flv -m -a`

说明:

打印出 file.flv 文件的信息，包括 metadata 信息，以及所有帧信息(包括非关键帧)。

### flv-split

##### flv 分段切割工具

示例:

`flv-split bigflvfile.flv -m 6 -p small- -u "http://127.0.0.1/videos/" -c`

说明:

把 bigflvfile.flv 这个 flv 文件，按照每6钟一段切版。切片后文件为 small-1.flv, small-2.flv 等。并且生成配置文件 small-config.xml。
配置文件中视频的地址是 http://127.0.0.1/videos/small-1.flv, http://127.0.0.1/videos/small-2.flv 等。

注意:

原 flv 视频应该包括完整的 matadata 信息。可以由 yamdi 来注入。

### flv-config

##### flv 分段配置生成工具

示例:

`flv-config small-1.flv small-2.flv -c -u "http://127.0.0.1/videos/"`

说明:

生成一个播放器能使用的切片视频配置文件，视频文件包括 small-1.flv, small-2.flv。视频内的链接是 http://127.0.0.1/videos/small-1.flv, http://127.0.0.1/videos/small-2.flv 。
