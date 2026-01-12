@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

REM Mock API Server 构建脚本 (Windows版本)

set APP_NAME=mock-api-server

echo 🚀 开始构建 Mock API Server
echo.

REM 检查构建环境
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 错误: 未找到构建环境，请先安装构建工具
    echo    访问 https://rustup.rs/ 获取构建工具
    exit /b 1
)

echo 🔍 构建工具版本:
cargo --version
echo.

REM 创建构建目录
if not exist dist mkdir dist

echo 🏗️  开始构建...
echo.

REM 检查是否有完整工具链
echo 检查构建环境...
cargo build --release --target x86_64-pc-windows-msvc --dry-run >nul 2>&1
if errorlevel 1 (
    echo ⚠️  警告: 未检测到完整构建工具链
    echo    将使用默认配置进行构建
    echo    如需完整的跨平台构建，请安装 Visual Studio Build Tools
    echo.
    goto :local_build
)

echo ✅ 检测到完整工具链，开始多平台构建...
echo.

REM Windows构建
echo 🔨 构建 Windows x64 版本...
rustup target add x86_64-pc-windows-msvc >nul 2>&1
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows x64 构建失败，回退到默认构建
    goto :local_build
)
copy "target\x86_64-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-amd64.exe" >nul
echo ✅ dist\%APP_NAME%-windows-amd64.exe 构建成功
echo.

echo 🔨 构建 Windows x86 版本...
rustup target add i686-pc-windows-msvc >nul 2>&1
cargo build --release --target i686-pc-windows-msvc
if errorlevel 1 (
    echo ❌ Windows x86 构建失败，跳过
) else (
    copy "target\i686-pc-windows-msvc\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-386.exe" >nul
    echo ✅ dist\%APP_NAME%-windows-386.exe 构建成功
)
echo.

goto :final_build

:local_build
echo 🏠 使用默认配置构建...
cargo build --release
if errorlevel 1 (
    echo ❌ 构建失败
    exit /b 1
)
copy "target\release\%APP_NAME%.exe" "dist\%APP_NAME%-windows-current.exe" >nul
echo ✅ dist\%APP_NAME%-windows-current.exe 构建成功
echo.

:final_build
REM 生成主程序
echo 🏠 生成主程序...
cargo build --release
if errorlevel 1 (
    echo ❌ 主程序构建失败
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
echo   %APP_NAME%.exe (主程序)
echo.
echo 🚀 运行方式:
echo   %APP_NAME%.exe
echo   %APP_NAME%.exe --help
echo   %APP_NAME%.exe -p 9000
echo.
echo 💡 提示:
echo   - 如需完整跨平台构建，请安装 Visual Studio Build Tools
echo   - 访问: https://visualstudio.microsoft.com/visual-cpp-build-tools/
echo.
echo 📖 更多信息请查看 README.md

endlocal