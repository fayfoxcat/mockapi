@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

REM Mock API Server 构建脚本 - Windows版本
REM 与 build.sh 保持一致的结构和输出格式

REM 颜色定义 (Windows CMD 颜色代码)
REM 0=黑色 1=蓝色 2=绿色 3=青色 4=红色 5=紫色 6=黄色 7=白色
REM 8=灰色 9=亮蓝 A=亮绿 B=亮青 C=亮红 D=亮紫 E=亮黄 F=亮白

REM 项目信息
set PROJECT_NAME=mockapi
set VERSION=1.0.0
set DIST_DIR=dist

REM 构建优化配置
set CARGO_INCREMENTAL=0
set CARGO_NET_RETRY=10
set RUSTFLAGS=-C link-arg=-s

REM 并行构建数量（根据CPU核心数调整）
set PARALLEL_JOBS=%NUMBER_OF_PROCESSORS%
if not defined PARALLEL_JOBS set PARALLEL_JOBS=4
set CARGO_BUILD_JOBS=%PARALLEL_JOBS%

REM Windows 平台配置 - 与 build.sh 中的 windows 目标保持一致
set PLATFORMS_COUNT=0
set PLATFORM_0_TARGET=x86_64-pc-windows-gnu
set PLATFORM_0_OUTPUT=windows-amd64.exe
set /a PLATFORMS_COUNT+=1

REM 构建统计
set SUCCESS_COUNT=0
set FAILED_COUNT=0
set BUILD_TIMES_INFO=

REM 颜色输出函数
set "ESC="

call :print_header
call :check_dependencies
call :setup_build_env %*

echo 🏗️  开始多平台构建...
echo.

REM 构建所有平台
for /l %%i in (0,1,0) do (
    if defined PLATFORM_%%i_TARGET (
        call :build_target "!PLATFORM_%%i_TARGET!" "!PLATFORM_%%i_OUTPUT!"
    )
)

REM 显示摘要
call :print_summary

REM 退出状态
if %FAILED_COUNT% equ 0 (
    echo.
    echo 🚀 所有构建成功完成！
    exit /b 0
) else (
    echo.
    echo ⚠️  部分构建失败，但有 %SUCCESS_COUNT% 个成功的构建
    exit /b 0
)

:print_header
echo.
echo ==========================================
echo    🚀 开始构建 %PROJECT_NAME% v%VERSION%
echo ==========================================
echo 📅 构建时间: %date% %time%

REM 检查Git信息
git rev-parse --short HEAD >nul 2>&1
if not errorlevel 1 (
    for /f %%i in ('git rev-parse --short HEAD 2^>nul') do echo 🔗 Git提交: %%i
)
echo.
goto :eof

:check_dependencies
echo 🔍 检查构建依赖...
echo.

REM 检查基本工具
rustc --version >nul 2>&1
if errorlevel 1 (
    echo ❌ 缺少必要工具: rustc/cargo
    echo 💡 请先安装 Rust: https://rustup.rs/
    exit /b 1
)

echo 🔧 Rust版本:
rustc --version
echo 📦 Cargo版本:
cargo --version
echo ⚡ 并行任务数: %PARALLEL_JOBS%

echo.
echo 🔍 检查并安装目标平台...

REM 检查并安装目标平台
for /l %%i in (0,1,0) do (
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

REM 清理之前的构建产物
if "%~1"=="--clean" (
    echo 🧹 清理之前的构建...
    cargo clean >nul 2>&1
    del /q "%DIST_DIR%\*" >nul 2>&1
)

REM 预构建依赖
echo 📦 预构建依赖...
cargo fetch --quiet >nul 2>&1

echo.
goto :eof

:build_target
set target=%~1
set output_name=%~2
set start_time=%time%

echo ==========================================
echo    构建 %target%
echo ==========================================

REM 检查目标是否已安装
rustup target list --installed | findstr /c:"%target%" >nul 2>&1
if errorlevel 1 (
    echo   ⚠️  目标 %target% 未安装
    echo.
    goto :eof
)

set binary_name=%PROJECT_NAME%
if "%target:windows=%" neq "%target%" (
    set binary_name=%PROJECT_NAME%.exe
)

set source_path=target\%target%\release\%binary_name%
set dest_path=%DIST_DIR%\%PROJECT_NAME%-%output_name%

echo   🏗️  编译中...

cargo build --release --target=%target%
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
    
    REM 进一步压缩（如果可用）
    where upx >nul 2>&1
    if not errorlevel 1 (
        echo   📦 使用UPX压缩...
        upx --best --lzma "%dest_path%" >nul 2>&1
        if not errorlevel 1 (
            echo   ✅ UPX压缩成功
        ) else (
            echo   ⚠️  UPX压缩失败，跳过
        )
    )
    
    call :get_file_size "%dest_path%" file_size_result
    echo   ✅ 构建成功: %dest_path%
    echo   📦 文件大小: !file_size_result!
    set /a SUCCESS_COUNT+=1
    
    REM 计算构建时间
    set end_time=%time%
    echo   ⏱️  构建时间: 已完成
) else (
    echo   ❌ 构建产物未找到: %source_path%
    set /a FAILED_COUNT+=1
)

echo.
goto :eof

:get_file_size
set file_path=%~1
if exist "%file_path%" (
    for %%A in ("%file_path%") do set file_size_bytes=%%~zA
    
    if !file_size_bytes! geq 1048576 (
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
echo ==========================================
echo    📊 构建结果摘要
echo ==========================================

REM 显示构建结果
for /l %%i in (0,1,0) do (
    if defined PLATFORM_%%i_TARGET (
        set dest_path=%DIST_DIR%\%PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!
        if exist "!dest_path!" (
            call :get_file_size "!dest_path!" summary_file_size
            echo   ✅ %PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!: !summary_file_size!
        ) else (
            echo   ❌ %PROJECT_NAME%-!PLATFORM_%%i_OUTPUT!: 构建失败
        )
    )
)

echo.
echo 📈 构建统计:
echo    ✅ 成功: %SUCCESS_COUNT%
echo    ❌ 失败: %FAILED_COUNT%

if %SUCCESS_COUNT% gtr 0 (
    echo.
    echo 🎉 构建完成！可执行文件位于 %DIST_DIR%/ 目录
    echo 💡 提示: 使用 --clean 参数可以清理构建缓存
)
goto :eof

endlocal