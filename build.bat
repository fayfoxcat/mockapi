chcp 65001
@echo off
setlocal enabledelayedexpansion

REM Mock API 构建脚本 (Windows版本)

set APP_NAME=mock-api
set VERSION=1.0.0

echo 🚀 开始构建 Mock API v%VERSION%
echo.

REM 检查Go环境
go version >nul 2>&1
if errorlevel 1 (
    echo ❌ 错误: 未找到Go环境，请先安装Go
    exit /b 1
)

echo 🔍 Go版本:
go version
echo.

REM 创建构建目录
if not exist dist mkdir dist

REM 下载依赖
echo 📦 下载依赖...
go mod tidy
echo.

echo 🏗️  开始多平台构建...
echo.

REM Windows构建
echo 🔨 构建 windows/amd64...
set GOOS=windows
set GOARCH=amd64
go build -o "dist\%APP_NAME%-windows-amd64.exe" main.go
if errorlevel 1 (
    echo ❌ Windows amd64 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-windows-amd64.exe 构建成功
echo.

echo 🔨 构建 windows/386...
set GOOS=windows
set GOARCH=386
go build -o "dist\%APP_NAME%-windows-386.exe" main.go
if errorlevel 1 (
    echo ❌ Windows 386 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-windows-386.exe 构建成功
echo.

REM Linux构建
echo 🔨 构建 linux/amd64...
set GOOS=linux
set GOARCH=amd64
go build -o "dist\%APP_NAME%-linux-amd64" main.go
if errorlevel 1 (
    echo ❌ Linux amd64 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-linux-amd64 构建成功
echo.

echo 🔨 构建 linux/arm64...
set GOOS=linux
set GOARCH=arm64
go build -o "dist\%APP_NAME%-linux-arm64" main.go
if errorlevel 1 (
    echo ❌ Linux arm64 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-linux-arm64 构建成功
echo.

REM macOS构建
echo 🔨 构建 darwin/amd64...
set GOOS=darwin
set GOARCH=amd64
go build -o "dist\%APP_NAME%-darwin-amd64" main.go
if errorlevel 1 (
    echo ❌ macOS amd64 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-darwin-amd64 构建成功
echo.

echo 🔨 构建 darwin/arm64...
set GOOS=darwin
set GOARCH=arm64
go build -o "dist\%APP_NAME%-darwin-arm64" main.go
if errorlevel 1 (
    echo ❌ macOS arm64 构建失败
    exit /b 1
)
echo ✅ dist\%APP_NAME%-darwin-arm64 构建成功
echo.

REM 本地构建
echo 🏠 构建本地版本...
set GOOS=
set GOARCH=
go build -o "%APP_NAME%.exe" main.go
if errorlevel 1 (
    echo ❌ 本地版本构建失败
    exit /b 1
)
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