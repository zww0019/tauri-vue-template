#!/bin/bash

# 智文比对 - 本地跨平台构建脚本
# 使用方法: ./scripts/build.sh [platform]
# platform: macos-arm64, macos-intel, macos-universal, windows, linux, all

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    print_info "检查构建依赖..."
    
    if ! command -v pnpm &> /dev/null; then
        print_error "pnpm 未安装，请先安装: npm install -g pnpm"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust 未安装，请先安装: https://rustup.rs"
        exit 1
    fi
    
    # 检查并安装 tauri-cli
    if ! cargo tauri --version &> /dev/null; then
        print_info "安装 tauri-cli..."
        cargo install tauri-cli
    fi
    
    print_success "依赖检查通过"
}

# 安装 Rust target
install_target() {
    local target=$1
    print_info "安装 Rust target: $target"
    rustup target add "$target" 2>/dev/null || true
}

# 构建前端
build_frontend() {
    print_info "构建前端..."
    pnpm install
    pnpm build
    print_success "前端构建完成"
}

# macOS ARM64 构建 (Apple Silicon)
build_macos_arm64() {
    print_info "构建 macOS ARM64 (Apple Silicon)..."
    install_target "aarch64-apple-darwin"
    cd src-tauri
    cargo tauri build --target aarch64-apple-darwin
    cd ..
    print_success "macOS ARM64 构建完成"
    print_info "输出目录: src-tauri/target/aarch64-apple-darwin/release/bundle/"
}

# macOS Intel 构建
build_macos_intel() {
    print_info "构建 macOS Intel (x86_64)..."
    install_target "x86_64-apple-darwin"
    cd src-tauri
    cargo tauri build --target x86_64-apple-darwin
    cd ..
    print_success "macOS Intel 构建完成"
    print_info "输出目录: src-tauri/target/x86_64-apple-darwin/release/bundle/"
}

# macOS Universal 构建 (同时支持 ARM64 和 Intel)
build_macos_universal() {
    print_info "构建 macOS Universal..."
    install_target "aarch64-apple-darwin"
    install_target "x86_64-apple-darwin"
    cd src-tauri
    cargo tauri build --target universal-apple-darwin
    cd ..
    print_success "macOS Universal 构建完成"
    print_info "输出目录: src-tauri/target/universal-apple-darwin/release/bundle/"
}

# 检查并安装 Windows 交叉编译工具链
setup_windows_cross_compile() {
    print_info "检查 Windows 交叉编译环境..."
    
    # 安装 Rust target
    install_target "x86_64-pc-windows-gnu"
    
    # 检查 mingw-w64
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        print_warning "mingw-w64 未安装"
        
        if [[ "$OSTYPE" == "darwin"* ]]; then
            print_info "正在通过 Homebrew 安装 mingw-w64..."
            if command -v brew &> /dev/null; then
                brew install mingw-w64
            else
                print_error "请先安装 Homebrew，然后运行: brew install mingw-w64"
                exit 1
            fi
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            print_info "请运行以下命令安装 mingw-w64:"
            echo "  sudo apt install mingw-w64  # Debian/Ubuntu"
            echo "  sudo dnf install mingw64-gcc  # Fedora"
            exit 1
        fi
    fi
    
    # 检查 NSIS (用于生成 Windows 安装程序)
    if ! command -v makensis &> /dev/null; then
        print_info "安装 NSIS (Windows 安装程序生成工具)..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            if command -v brew &> /dev/null; then
                brew install nsis
            fi
        fi
    fi
    
    print_success "Windows 交叉编译环境就绪"
}

# Windows 构建
build_windows() {
    local portable=${1:-false}
    print_info "构建 Windows x64..."
    
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        # Windows 原生构建
        install_target "x86_64-pc-windows-msvc"
        cd src-tauri
        if [[ "$portable" == "true" ]]; then
            cargo tauri build --target x86_64-pc-windows-msvc --no-bundle
        else
            cargo tauri build --target x86_64-pc-windows-msvc
        fi
        cd ..
        print_success "Windows x64 构建完成"
    else
        # macOS/Linux 交叉编译
        setup_windows_cross_compile
        cd src-tauri
        if [[ "$portable" == "true" ]]; then
            # 便携式版本：只构建 exe，不打包安装程序
            cargo tauri build --target x86_64-pc-windows-gnu --no-bundle
        else
            cargo tauri build --target x86_64-pc-windows-gnu
        fi
        cd ..
        print_success "Windows x64 (交叉编译) 构建完成"
    fi
    
    # 显示输出路径
    local target_dir="x86_64-pc-windows-gnu"
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        target_dir="x86_64-pc-windows-msvc"
    fi
    
    if [[ "$portable" == "true" ]]; then
        print_info "便携式 exe: src-tauri/target/$target_dir/release/智文比对.exe"
    else
        print_info "输出目录: src-tauri/target/$target_dir/release/bundle/"
    fi
}

# Linux 构建
build_linux() {
    print_info "构建 Linux x64..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux 原生构建
        install_target "x86_64-unknown-linux-gnu"
        cd src-tauri
        cargo tauri build --target x86_64-unknown-linux-gnu
        cd ..
        print_success "Linux x64 构建完成"
        print_info "输出目录: src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/"
    else
        # 使用 Docker 交叉编译
        build_linux_docker
    fi
}

# Docker Linux 构建
build_linux_docker() {
    print_info "使用 Docker 构建 Linux x64..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装，请先安装 Docker"
        print_info "安装地址: https://www.docker.com/get-started"
        exit 1
    fi
    
    # 检查 Docker 是否运行
    if ! docker info &> /dev/null; then
        print_error "Docker 未运行，请先启动 Docker"
        exit 1
    fi
    
    print_info "正在拉取构建镜像..."
    
    # 使用 Dockerfile 构建
    docker run --rm -v "$(pwd)":/app -w /app \
        --platform linux/amd64 \
        rust:latest \
        bash -c '
            apt-get update && \
            apt-get install -y \
                libwebkit2gtk-4.1-dev \
                libappindicator3-dev \
                librsvg2-dev \
                patchelf \
                nodejs \
                npm && \
            npm install -g pnpm && \
            pnpm install && \
            pnpm build && \
            cargo install tauri-cli && \
            cd src-tauri && \
            cargo tauri build --target x86_64-unknown-linux-gnu
        '
    
    print_success "Linux x64 (Docker) 构建完成"
    print_info "输出目录: src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/"
}

# 显示帮助
show_help() {
    echo "智文比对 - 本地构建脚本"
    echo ""
    echo "使用方法: ./scripts/build.sh [选项]"
    echo ""
    echo "选项:"
    echo "  macos-arm64      构建 macOS ARM64 (Apple Silicon M1/M2/M3)"
    echo "  macos-intel      构建 macOS Intel (x86_64)"
    echo "  macos-universal  构建 macOS Universal (同时支持 ARM64 和 Intel)"
    echo "  macos            构建当前 macOS 架构"
    echo "  windows          构建 Windows x64 (含安装程序)"
    echo "  windows-portable 构建 Windows x64 便携式版本 (免安装)"
    echo "  linux            构建 Linux x64"
    echo "  all              构建所有支持的平台"
    echo "  help             显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  ./scripts/build.sh macos-arm64"
    echo "  ./scripts/build.sh windows-portable  # 推荐：免安装版本"
    echo ""
    echo "输出位置: src-tauri/target/[target]/release/bundle/"
}

# 主函数
main() {
    local platform=${1:-help}
    
    case $platform in
        macos-arm64)
            check_dependencies
            build_frontend
            build_macos_arm64
            ;;
        macos-intel)
            check_dependencies
            build_frontend
            build_macos_intel
            ;;
        macos-universal)
            check_dependencies
            build_frontend
            build_macos_universal
            ;;
        macos)
            check_dependencies
            build_frontend
            if [[ $(uname -m) == "arm64" ]]; then
                build_macos_arm64
            else
                build_macos_intel
            fi
            ;;
        windows)
            check_dependencies
            build_frontend
            build_windows false
            ;;
        windows-portable)
            check_dependencies
            build_frontend
            build_windows true
            ;;
        linux)
            check_dependencies
            build_frontend
            build_linux
            ;;
        all)
            check_dependencies
            build_frontend
            
            # macOS 构建
            if [[ "$OSTYPE" == "darwin"* ]]; then
                build_macos_arm64
                build_macos_intel
            fi
            
            # Windows 构建
            build_windows
            
            # Linux 构建
            build_linux
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "未知选项: $platform"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
