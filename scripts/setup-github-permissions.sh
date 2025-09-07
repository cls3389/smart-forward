#!/bin/bash
# GitHub 权限配置脚本 (Linux/macOS)
# 用于快速检查和配置 GitHub 仓库权限

REPOSITORY=${1:-"cls3389/smart-forward"}
TOKEN=${2:-""}

echo "🔧 GitHub 权限配置工具"
echo "========================="

# 检查 GitHub CLI 是否安装
echo ""
echo "📋 环境检查:"
if command -v gh &> /dev/null; then
    echo "✅ GitHub CLI 已安装: $(gh --version | head -n1)"
else
    echo "❌ GitHub CLI 未安装"
    echo "请安装: https://cli.github.com/"
    exit 1
fi

# 检查认证状态
echo ""
echo "🔐 认证检查:"
if gh auth status &> /dev/null; then
    echo "✅ GitHub 认证成功"
else
    echo "❌ GitHub 未认证"
    echo "请运行: gh auth login"
    exit 1
fi

# 检查仓库权限
echo ""
echo "🏠 仓库权限检查:"
if gh repo view "$REPOSITORY" --json permissions &> /dev/null; then
    PERMISSIONS=$(gh repo view "$REPOSITORY" --json permissions -q '.permissions')
    echo "仓库: $REPOSITORY"
    echo "管理员权限: $(echo $PERMISSIONS | jq -r '.admin')"
    echo "推送权限: $(echo $PERMISSIONS | jq -r '.push')"
    echo "拉取权限: $(echo $PERMISSIONS | jq -r '.pull')"
else
    echo "❌ 无法访问仓库: $REPOSITORY"
    echo "请检查仓库名称和权限"
fi

# 检查 Secrets
echo ""
echo "🔑 Secrets 检查:"
if gh secret list --repo "$REPOSITORY" &> /dev/null; then
    SECRETS=$(gh secret list --repo "$REPOSITORY")
    if [ -n "$SECRETS" ]; then
        echo "✅ 找到 Secrets:"
        echo "$SECRETS" | while read -r secret; do
            echo "  - $secret"
        done
    else
        echo "⚠️  没有找到 Secrets"
    fi
else
    echo "❌ 无法访问 Secrets"
fi

# 检查包权限
echo ""
echo "📦 包权限检查:"
if gh api "repos/$REPOSITORY/packages" &> /dev/null; then
    PACKAGES=$(gh api "repos/$REPOSITORY/packages" -q '.[].name' 2>/dev/null)
    if [ -n "$PACKAGES" ]; then
        echo "✅ 找到包:"
        echo "$PACKAGES" | while read -r package; do
            echo "  - $package"
        done
    else
        echo "⚠️  没有找到包"
    fi
else
    echo "❌ 无法访问包信息"
fi

# 提供配置建议
echo ""
echo "💡 配置建议:"
echo "1. 检查仓库 Settings → Actions → General"
echo "2. 确认 Workflow permissions 设置为 'Read and write permissions'"
echo "3. 添加必要的 Secrets (GHCR_TOKEN 等)"
echo "4. 检查包权限设置"
echo "5. 配置分支保护规则"

echo ""
echo "🔗 相关链接:"
echo "仓库设置: https://github.com/$REPOSITORY/settings"
echo "Actions: https://github.com/$REPOSITORY/actions"
echo "包管理: https://github.com/$REPOSITORY/packages"
echo "详细文档: docs/github-permissions-guide.md"
