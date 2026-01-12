@echo off
chcp 65001
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

echo 🏗️  开始多平台构建...
echo.

REM Windows构建
echo 🔨 构建 x86_64-pc-windows-msvc...
rustup target add x86_64-pc-windows-msvc >nul 2>&1
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows x86_64 构建失败
    exit /b 1
)
copy "target\x86_64-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-amd64.exe" >nul
echo ✅ dist\%APP_NAME%-windows-amd64.exe 构建成功
echo.

echo 🔨 构建 i686-pc-windows-msvc...
rustup target add i686-pc-windows-msvc >nul 2>&1
cargo build --release --target i686-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows i686 构建失败
    exit /b 1
)
copy "target\i686-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-386.exe" >nul
echo ✅ dist\%APP_NAME%-windows-386.exe 构建成功
echo.

REM Linux构建
echo 🔨 构建 x86_64-unknown-linux-gnu...
rustup target add x86_64-unknown-linux-gnu >nul 2>&1
cargo build --release --target x86_64-unknown-linux-gnu
if errorlevel 1 (
    echo ❌ Linux x86_64 构建失败
    exit /b 1
)
copy "target\x86_64-unknown-linux-gnu\release\%APP_NAME%" "dist\%APP_NAME%-linux-amd64" >nul
echo ✅ dist\%APP_NAME%-linux-amd64 构建成功
echo.

echo 🔨 构建 aarch64-unknown-linux-gnu...
rustup target add aarch64-unknown-linux-gnu >nul 2>&1
cargo build --release --target aarch64-unknown-linux-gnu
if errorlevel 1 (
    echo ❌ Linux arm64 构建失败
    exit /b 1
)
copy "target\aarch64-unknown-linux-gnu\release\%APP_NAME%" "dist\%APP_NAME%-linux-arm64" >nul
echo ✅ dist\%APP_NAME%-linux-arm64 构建成功
echo.

REM macOS构建
echo 🔨 构建 x86_64-apple-darwin...
rustup target add x86_64-apple-darwin >nul 2>&1
cargo build --release --target x86_64-apple-darwin
if errorlevel 1 (
    echo ❌ macOS x86_64 构建失败
    exit /b 1
)
copy "target\x86_64-apple-darwin\release\%APP_NAME%" "dist\%APP_NAME%-darwin-amd64" >nul
echo ✅ dist\%APP_NAME%-darwin-amd64 构建成功
echo.

echo 🔨 构建 aarch64-apple-darwin...
rustup target add aarch64-apple-darwin >nul 2>&1
cargo build --release --target aarch64-apple-darwin
if errorlevel 1 (
    echo ❌ macOS arm64 构建失败
    exit /b 1
)
copy "target\aarch64-apple-darwin\release\%APP_NAME%" "dist\%APP_NAME%-darwin-arm64" >nul
echo ✅ dist\%APP_NAME%-darwin-arm64 构建成功
echo.

REM 本地构建
echo 🏠 构建本地版本...
cargo build --release
if errorlevel 1 (
    echo ❌ 本地版本构建失败
    exit /b 1
)
copy "target\release\%APP_NAME%.exe" "%APP_NAME%.exe" >nul
echo ✅ %APP_NAME%.exe 构建成功
echo.

echo 🎉 所有构建完成！
echo.
echo 📁 构建产物:
dir dist\
echo.
echo 🚀 本地运行:
echo   %APP_NAME%.exe
echo.
echo 📖 更多信息请查看 README.md

endlocal