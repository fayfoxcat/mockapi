#!/bin/bash

APP_NAME="mockapi"
OUTPUT_DIR="dist"

rm -rf $OUTPUT_DIR
mkdir -p $OUTPUT_DIR

# CGO_ENABLED=0 禁用CGO，生成静态链接的二进制文件，无需依赖系统GLIBC

echo "编译 Linux amd64..."
CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o $OUTPUT_DIR/${APP_NAME}-linux-amd64 .

echo "编译 Linux arm64..."
CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -ldflags="-s -w" -o $OUTPUT_DIR/${APP_NAME}-linux-arm64 .

echo "编译 macOS amd64..."
CGO_ENABLED=0 GOOS=darwin GOARCH=amd64 go build -ldflags="-s -w" -o $OUTPUT_DIR/${APP_NAME}-darwin-amd64 .

echo "编译 macOS arm64..."
CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -ldflags="-s -w" -o $OUTPUT_DIR/${APP_NAME}-darwin-arm64 .

echo "编译 Windows amd64..."
CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -ldflags="-s -w" -o $OUTPUT_DIR/${APP_NAME}-windows-amd64.exe .

echo ""
echo "打包完成:"
ls -lh $OUTPUT_DIR/
