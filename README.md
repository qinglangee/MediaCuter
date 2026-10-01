# 编译执行
项目使用 npm run tauri 来执行 Tauri 命令，以下是常用操作：

## 开发模式（热重载）：
`npm run tauri dev`
前端修改会自动热重载，Rust 修改会自动重新编译。

## 构建 Release 版本：
`npm run tauri build`
编译完成后，安装包在 src-tauri/target/release/bundle/ 目录下：
msi/ — Windows MSI 安装包
nsis/ — Windows NSIS 安装包（.exe）

构建完成后，可直接运行的 exe 文件在：
src-tauri\target\release\tauri01.exe

双击即可运行，不需要安装。

注意： 它依赖系统上的 WebView2 运行时（Windows 10/11 通常已预装）。如果目标电脑没有 WebView2，打开时会报错。这种情况下需要用 NSIS 安装包（在 src-tauri\target\release\bundle\nsis\ 下），安装包可以自动检测并提示安装 WebView2。

## 仅检查 Rust 编译是否通过：
`cd src-tauri && cargo check`
