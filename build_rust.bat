@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

REM Mock API Server Rust版本构建脚本 (Windows版本)

set APP_NAME=mock-api-server

echo 🚀 开始构建 Mock API Server Rust版本
echo.

REM 检查Rust环境
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 错误: 未找到Rust环境，请先安装Rust
    echo    访问 https://rustup.rs/ 安装Rust
    exit /b 1
)

echo 🔍 Rust版本:
cargo --version
echo.

REM 创建构建目录
if not exist dist mkdir dist

echo 🏗️  开始构建...
echo.

REM 检查是否有MSVC工具链
echo 检查构建环境...
cargo build --release --target x86_64-pc-windows-msvc --dry-run >nul 2>&1
if errorlevel 1 (
    echo ⚠️  警告: 未检测到Visual Studio C++构建工具
    echo    将使用默认工具链进行构建
    echo    如需完整的跨平台构建，请安装Visual Studio Build Tools
    echo.
    goto :local_build
)

echo ✅ 检测到MSVC工具链，开始多平台构建...
echo.

REM Windows构建
echo 🔨 构建 x86_64-pc-windows-msvc...
rustup target add x86_64-pc-windows-msvc >nul 2>&1
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows x86_64 构建失败，回退到本地构建
    goto :local_build
)
copy "target\x86_64-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-amd64.exe" >nul
echo ✅ dist\%APP_NAME%-windows-amd64.exe 构建成功
echo.

echo 🔨 构建 i686-pc-windows-msvc...
rustup target add i686-pc-windows-msvc >nul 2>&1
cargo build --release --target i686-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows i686 构建失败，跳过
) else (
    copy "target\i686-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-386.exe" >nul
    echo ✅ dist\%APP_NAME%-windows-386.exe 构建成功
)
echo.

goto :final_build

:local_build
echo 🏠 使用默认工具链构建本地版本...
cargo build --release
if errorlevel 1 (
    echo ❌ 本地版本构建失败
    exit /b 1
)
copy "target\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-current.exe" >nul
echo ✅ dist\%APP_NAME%-windows-current.exe 构建成功
echo.

:final_build
REM 本地构建
echo 🏠 构建本地可执行文件...
cargo build --release
if errorlevel 1 (
    echo ❌ 本地版本构建失败
    exit /b 1
)
copy "target\release\%APP_NAME%.exe" "%APP_NAME%.exe" >nul
echo ✅ %APP_NAME%.exe 构建成功
echo.

echo 🎉 构建完成！
echo.
echo 📁 构建产物:
if exist "dist\%APP_NAME%-windows-amd64.exe" (
    echo   dist\%APP_NAME%-windows-amd64.exe
)
if exist "dist\%APP_NAME%-windows-386.exe" (
    echo   dist\%APP_NAME%-windows-386.exe
)
if exist "dist\%APP_NAME%-windows-current.exe" (
    echo   dist\%APP_NAME%-windows-current.exe
)
echo   %APP_NAME%.exe (根目录)
echo.
echo 🚀 本地运行:
echo   %APP_NAME%.exe
echo   %APP_NAME%.exe --help
echo   %APP_NAME%.exe -p 9000
echo.
echo 💡 提示:
echo   - 如需完整跨平台构建，请安装 Visual Studio Build Tools
echo   - 访问: https://visualstudio.microsoft.com/visual-cpp-build-tools/
echo.
echo 📖 更多信息请查看 README_RUST.md

endlocal