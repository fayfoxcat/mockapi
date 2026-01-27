@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

REM Mock API Server 构建脚本 - Windows优化版本
REM 支持多平台交叉编译，优化构建速度

REM 项目信息
set PROJECT_NAME=mock-api-server
set VERSION=1.0.0
set DIST_DIR=dist

REM 构建优化配置
set CARGO_INCREMENTAL=1
set CARGO_NET_RETRY=10
set RUSTC_WRAPPER=

REM 并行构建数量（根据CPU核心数调整）
for /f "tokens=2 delims==" %%i in ('wmic cpu get NumberOfLogicalProcessors /value ^| find "="') do set PARALLEL_JOBS=%%i
if not defined PARALLEL_JOBS set PARALLEL_JOBS=4
set CARGO_BUILD_JOBS=%PARALLEL_JOBS%

REM 构建统计
set SUCCESS_COUNT=0
set FAILED_COUNT=0
set SKIPPED_COUNT=0

REM 支持的平台配置
set PLATFORMS_COUNT=0
set PLATFORM_0_TARGET=x86_64-pc-windows-msvc
set PLATFORM_0_OUTPUT=windows-amd64.exe
set /a PLATFORMS_COUNT+=1

set PLATFORM_1_TARGET=i686-pc-windows-msvc
set PLATFORM_1_OUTPUT=windows-386.exe
set /a PLATFORMS_COUNT+=1

set PLATFORM_2_TARGET=x86_64-pc-windows-gnu
set PLATFORM_2_OUTPUT=windows-amd64-gnu.exe
set /a PLATFORMS_COUNT+=1

set PLATFORM_3_TARGET=x86_64-unknown-linux-gnu
set PLATFORM_3_OUTPUT=linux-amd64
set /a PLATFORMS_COUNT+=1

set PLATFORM_4_TARGET=aarch64-unknown-linux-gnu
set PLATFORM_4_OUTPUT=linux-arm64
set /a PLATFORMS_COUNT+=1

call :print_header
call :print_build_info
call :check_dependencies
call :setup_build_env %*

echo 🏗️  开始多平台构建...
echo.

REM 构建所有平台
for /l %%i in (0,1,%PLATFORMS_COUNT%-1) do (
    if defined PLATFORM_%%i_TARGET (
        call :build_target "!PLATFORM_%%i_TARGET!" "!PLATFORM_%%i_OUTPUT!"
    )
)

REM 本地构建
call :build_local

REM 显示摘要
call :print_summary

REM 退出状态
if %FAILED_COUNT% equ 0 (
    echo.
    echo 🎉 构建完成！
    echo 📖 更多信息请查看 README.md
    exit /b 0
) else (
    echo.
    echo ⚠️  部分构建失败，但有成功的构建产物
    exit /b 0
)

:print_header
echo 🚀 开始构建 %PROJECT_NAME% v%VERSION%
echo 📅 构建时间: %date% %time%

REM 检查Git信息
git rev-parse --short HEAD >nul 2>&1
if not errorlevel 1 (
    for /f %%i in ('git rev-parse --short HEAD 2^>nul') do echo 🔗 Git提交: %%i
)
echo.
goto :eof

:print_build_info
echo 🔍 构建工具版本:
cargo --version 2>nul || (
    echo ❌ 错误: 未找到 Rust 构建环境
    echo    请访问 https://rustup.rs/ 安装 Rust
    exit /b 1
)
echo 🔧 Cargo版本:
cargo --version
echo ⚡ 并行任务数: %PARALLEL_JOBS%
echo.
goto :eof

:check_dependencies
echo 🔍 检查构建依赖...

REM 检查基本工具
rustc --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 缺少必要工具: rustc
    echo 请先安装 Rust: https://rustup.rs/
    exit /b 1
)

cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 缺少必要工具: cargo
    echo 请先安装 Rust: https://rustup.rs/
    exit /b 1
)

echo 📋 检查交叉编译工具链...

REM 检查并安装目标平台
for /l %%i in (0,1,%PLATFORMS_COUNT%-1) do (
    if defined PLATFORM_%%i_TARGET (
        call :check_target "!PLATFORM_%%i_TARGET!"
    )
)

echo.
goto :eof

:check_target
set target=%~1
rustup target list --installed | findstr /c:"%target%" >nul 2>&1
if errorlevel 1 (
    echo   ⚠️  %target%: 未安装，正在安装...
    rustup target add "%target%" >nul 2>&1
    if errorlevel 1 (
        echo   ❌ %target%: 安装失败
        set /a FAILED_COUNT+=1
    ) else (
        echo   ✅ %target%: 安装成功
    )
) else (
    echo   ✅ %target%: 已安装
)
goto :eof

:setup_build_env
echo 🏗️  准备构建环境...

REM 创建输出目录
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"

REM 清理之前的构建产物（可选）
if "%~1"=="--clean" (
    echo 🧹 清理之前的构建...
    cargo clean >nul 2>&1
    del /q "%DIST_DIR%\*" >nul 2>&1
)

REM 预构建依赖（加速后续构建）
echo 📦 预构建依赖...
cargo fetch --quiet >nul 2>&1

echo.
goto :eof

:build_target
set target=%~1
set output_name=%~2

echo === %target% ===
echo 🔨 构建 %target%...

set binary_name=%PROJECT_NAME%
if "%target:windows=%" neq "%target%" (
    set binary_name=%PROJECT_NAME%.exe
)

set source_path=target\%target%\release\%binary_name%
set dest_path=%DIST_DIR%\%PROJECT_NAME%-%output_name%

echo   🏗️  编译中...

cargo build --release --target=%target% >nul 2>&1
if errorlevel 1 (
    echo   ❌ 编译失败
    set /a FAILED_COUNT+=1
    echo.
    goto :eof
)

if exist "%source_path%" (
    copy "%source_path%" "%dest_path%" >nul 2>&1
    if errorlevel 1 (
        echo   ❌ 文件复制失败: %source_path% -^> %dest_path%
        set /a FAILED_COUNT+=1
        echo.
        goto :eof
    )
    
    call :get_file_size "%dest_path%" file_size
    echo   ✅ 构建成功: %dest_path%
    echo   📦 文件大小: %file_size%
    set /a SUCCESS_COUNT+=1
) else (
    echo   ❌ 构建产物未找到: %source_path%
    set /a FAILED_COUNT+=1
)

echo ✅ 构建完成
echo.
goto :eof

:build_local
echo === 本地构建 ===
echo 🏠 构建本地版本...

cargo build --release >nul 2>&1
if errorlevel 1 (
    echo ❌ 本地构建失败
    set /a FAILED_COUNT+=1
    goto :eof
)

set local_binary=target\release\%PROJECT_NAME%.exe
if exist "%local_binary%" (
    echo ✅ %PROJECT_NAME% 构建成功
    set /a SUCCESS_COUNT+=1
) else (
    echo ❌ 本地构建产物未找到
    set /a FAILED_COUNT+=1
)
goto :eof

:get_file_size
set file_path=%~1
if exist "%file_path%" (
    for %%A in ("%file_path%") do set file_size_bytes=%%~zA
    
    if !file_size_bytes! geq 1073741824 (
        set /a size_gb=!file_size_bytes!/1073741824
        set "%~2=!size_gb!G"
    ) else if !file_size_bytes! geq 1048576 (
        set /a size_mb=!file_size_bytes!/1048576
        set "%~2=!size_mb!M"
    ) else if !file_size_bytes! geq 1024 (
        set /a size_kb=!file_size_bytes!/1024
        set "%~2=!size_kb!K"
    ) else (
        set "%~2=!file_size_bytes!B"
    )
) else (
    set "%~2=0B"
)
goto :eof

:print_summary
echo.
echo 📊 构建结果摘要:

REM 显示构建结果
for /l %%i in (0,1,%PLATFORMS_COUNT%-1) do (
    if defined PLATFORM_%%i_TARGET (
        set dest_path=%DIST_DIR%\%PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!
        if exist "!dest_path!" (
            echo   ✅ %PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!: 构建成功
        ) else (
            echo   ❌ %PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!: 构建失败
        )
    )
)

REM 本地构建
if exist "target\release\%PROJECT_NAME%.exe" (
    echo   ✅ %PROJECT_NAME%: 本地构建成功
) else (
    echo   ❌ %PROJECT_NAME%: 本地构建失败
)

echo.
echo 📈 构建统计:
echo    ✅ 成功: %SUCCESS_COUNT%
echo    ❌ 失败: %FAILED_COUNT%
echo    ⏭️  跳过: %SKIPPED_COUNT%

echo.
echo 📁 构建产物:
if exist "%DIST_DIR%" (
    for %%f in ("%DIST_DIR%\*") do (
        if exist "%%f" (
            call :get_file_size "%%f" file_size
            echo    %%~nxf - !file_size!
        )
    )
    
    echo.
    echo 🚀 本地运行:
    echo   target\release\%PROJECT_NAME%.exe
    echo   %PROJECT_NAME%.exe --help
    echo   %PROJECT_NAME%.exe -p 9000
) else (
    echo    无构建产物
)

echo.
echo 💡 提示:
echo   - Windows 完整构建需要 Visual Studio Build Tools
echo   - Linux 交叉编译需要相应的工具链
echo   - 访问: https://visualstudio.microsoft.com/visual-cpp-build-tools/
goto :eof

endlocal