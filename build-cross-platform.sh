#!/bin/bash
# ================================
# 智能网络转发器 - 跨平台构建脚本 (Linux/macOS)
# ================================
# 支持平台: Windows, macOS, Linux
# 构建目标: x86_64, ARM64
# ================================

set -e

# 默认参数
PLATFORM="all"
ARCH="all"
RELEASE=false
DOCKER=false
CLEAN=false

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# 输出函数
print_color() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_error() {
    print_color $RED "❌ 错误: $1"
    exit 1
}

print_success() {
    print_color $GREEN "✅ $1"
}

print_info() {
    print_color $CYAN "🔍 $1"
}

print_warning() {
    print_color $YELLOW "⚠️  $1"
}

print_build() {
    print_color $BLUE "🔨 $1"
}

# 显示帮助
show_help() {
    echo "智能网络转发器 - 跨平台构建脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -p, --platform PLATFORM  构建平台 (all|windows|macos|linux) [默认: all]"
    echo "  -a, --arch ARCH          架构 (all|x86_64|aarch64) [默认: all]"
    echo "  -r, --release            发布构建"
    echo "  -d, --docker             构建Docker镜像"
    echo "  -c, --clean              清理构建产物"
    echo "  -h, --help               显示帮助"
    echo ""
    echo "示例:"
    echo "  $0                        # 构建所有平台"
    echo "  $0 -p linux -r            # 构建Linux发布版本"
    echo "  $0 -p windows -a x86_64   # 构建Windows x86_64"
    echo "  $0 -d                     # 构建Docker镜像"
    echo "  $0 -c                     # 清理构建产物"
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -r|--release)
            RELEASE=true
            shift
            ;;
        -d|--docker)
            DOCKER=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "未知参数: $1"
            ;;
    esac
done

# 检查依赖
check_dependencies() {
    print_info "检查构建依赖..."
    
    # 检查 Rust
    if ! command -v rustc &> /dev/null; then
        print_error "未找到 Rust，请先安装: https://rustup.rs/"
    fi
    local rust_version=$(rustc --version)
    print_success "Rust: $rust_version"
    
    # 检查 Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "未找到 Cargo"
    fi
    local cargo_version=$(cargo --version)
    print_success "Cargo: $cargo_version"
    
    # 检查交叉编译工具链
    if [[ "$PLATFORM" == "all" || "$PLATFORM" != "linux" ]]; then
        if ! rustup target list --installed &> /dev/null; then
            print_warning "需要安装交叉编译工具链"
        else
            print_success "交叉编译工具链已安装"
        fi
    fi
}

# 安装交叉编译工具链
install_cross_compile_targets() {
    print_info "安装交叉编译工具链..."
    
    local targets=()
    
    if [[ "$PLATFORM" == "all" || "$PLATFORM" == "macos" ]]; then
        targets+=("x86_64-apple-darwin" "aarch64-apple-darwin")
    fi
    
    if [[ "$PLATFORM" == "all" || "$PLATFORM" == "linux" ]]; then
        targets+=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
    fi
    
    for target in "${targets[@]}"; do
        print_info "安装目标: $target"
        rustup target add "$target"
    done
}

# 构建目标
build_target() {
    local target=$1
    local output_dir=$2
    local binary_name=$3
    
    print_build "构建目标: $target"
    
    # 构建参数
    local build_args=("build" "--target" "$target")
    if [[ "$RELEASE" == "true" ]]; then
        build_args+=("--release")
    fi
    
    # 执行构建
    cargo "${build_args[@]}"
    
    # 确定源文件路径
    local source_path
    if [[ "$RELEASE" == "true" ]]; then
        source_path="target/$target/release/$binary_name"
    else
        source_path="target/$target/debug/$binary_name"
    fi
    
    # 确定目标文件路径
    local target_path="$output_dir/$binary_name"
    if [[ "$target" == *"windows"* ]]; then
        target_path="$target_path.exe"
    fi
    
    # 复制文件
    if [[ -f "$source_path" ]]; then
        cp "$source_path" "$target_path"
        print_success "构建完成: $target_path"
    else
        print_error "构建产物未找到: $source_path"
    fi
}

# 清理函数
clean_build_artifacts() {
    print_info "清理构建产物..."
    
    if [[ -d "target" ]]; then
        rm -rf target
    fi
    
    if [[ -d "dist" ]]; then
        rm -rf dist
    fi
    
    print_success "清理完成"
}

# Docker构建
build_docker() {
    print_info "构建 Docker 镜像..."
    
    # 构建镜像
    docker build -t smart-forward:latest .
    
    # 创建多架构镜像 (如果支持)
    if command -v docker buildx &> /dev/null; then
        docker buildx create --use --name multiarch-builder 2>/dev/null || true
        docker buildx build --platform linux/amd64,linux/arm64 -t smart-forward:latest --push .
    fi
    
    print_success "Docker 镜像构建完成"
}

# 主函数
main() {
    print_color $MAGENTA "🚀 智能网络转发器 - 跨平台构建"
    print_color $MAGENTA "================================="
    
    # 清理
    if [[ "$CLEAN" == "true" ]]; then
        clean_build_artifacts
        return
    fi
    
    # 检查依赖
    check_dependencies
    
    # 安装交叉编译工具链
    if [[ "$PLATFORM" != "windows" ]]; then
        install_cross_compile_targets
    fi
    
    # 创建输出目录
    local dist_dir="dist"
    mkdir -p "$dist_dir"
    
    # 构建目标
    local build_targets=()
    
    if [[ "$PLATFORM" == "all" || "$PLATFORM" == "windows" ]]; then
        build_targets+=("x86_64-pc-windows-msvc|$dist_dir/windows-x86_64|smart-forward")
    fi
    
    if [[ "$PLATFORM" == "all" || "$PLATFORM" == "macos" ]]; then
        if [[ "$ARCH" == "all" || "$ARCH" == "x86_64" ]]; then
            build_targets+=("x86_64-apple-darwin|$dist_dir/macos-x86_64|smart-forward")
        fi
        if [[ "$ARCH" == "all" || "$ARCH" == "aarch64" ]]; then
            build_targets+=("aarch64-apple-darwin|$dist_dir/macos-aarch64|smart-forward")
        fi
    fi
    
    if [[ "$PLATFORM" == "all" || "$PLATFORM" == "linux" ]]; then
        if [[ "$ARCH" == "all" || "$ARCH" == "x86_64" ]]; then
            build_targets+=("x86_64-unknown-linux-gnu|$dist_dir/linux-x86_64|smart-forward")
        fi
        if [[ "$ARCH" == "all" || "$ARCH" == "aarch64" ]]; then
            build_targets+=("aarch64-unknown-linux-gnu|$dist_dir/linux-aarch64|smart-forward")
        fi
    fi
    
    # 执行构建
    for build_target in "${build_targets[@]}"; do
        IFS='|' read -r target output_dir binary_name <<< "$build_target"
        
        # 创建输出目录
        mkdir -p "$output_dir"
        
        # 构建
        build_target "$target" "$output_dir" "$binary_name"
        
        # 复制配置文件
        cp config.yaml.example "$output_dir/config.yaml"
        cp README.md "$output_dir/README.md" 2>/dev/null || true
    done
    
    # Docker构建
    if [[ "$DOCKER" == "true" ]]; then
        build_docker
    fi
    
    print_success "🎉 所有构建任务完成！"
    print_info "构建产物位于: $dist_dir"
}

# 执行主函数
main
