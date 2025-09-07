# 🔧 构建问题排查指南

## 🎯 **常见构建错误及解决方案**

### 🐳 **Docker构建错误**

#### 错误1: LTO编译冲突
```
error: options `-C embed-bitcode=no` and `-C lto` are incompatible
```

**原因**: Rust编译参数冲突
**解决方案**: 
1. 检查 `Dockerfile` 中的 `RUSTFLAGS` 设置
2. 确保不要同时设置冲突的编译选项
3. 使用 `Cargo.toml` 中的 `[profile.release]` 配置LTO

#### 错误2: 权限被拒绝
```
ERROR: failed to push: denied: permission_denied
```

**原因**: GitHub Container Registry权限不足
**解决方案**:
1. 检查仓库设置 → Actions → General → Workflow permissions
2. 选择 "Read and write permissions"
3. 确保仓库名称是小写

#### 错误3: 构建超时
```
The job running on runner has exceeded the maximum execution time
```

**原因**: 构建时间过长
**解决方案**:
1. 检查网络连接
2. 使用缓存优化构建时间
3. 增加 `timeout-minutes` 设置

---

### 🔨 **多平台构建错误**

#### 错误1: 交叉编译失败
```
error: linker `aarch64-linux-gnu-gcc` not found
```

**原因**: 缺少交叉编译工具链
**解决方案**:
```yaml
- name: 安装交叉编译工具 (Linux ARM64)
  if: matrix.platform == 'linux-aarch64'
  run: |
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu
```

#### 错误2: 目标平台不支持
```
error: target 'aarch64-apple-darwin' not found
```

**原因**: Rust目标平台未安装
**解决方案**:
```yaml
- name: 设置 Rust
  uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.target }}
```

---

### 🔍 **CI检查错误**

#### 错误1: 格式检查失败
```
error: rustfmt failed
```

**解决方案**:
```bash
# 本地修复格式
cargo fmt

# 检查格式
cargo fmt -- --check
```

#### 错误2: Clippy警告
```
error: this could be written as `let...else`
```

**解决方案**:
```bash
# 修复Clippy警告
cargo clippy --fix

# 检查警告
cargo clippy -- -D warnings
```

#### 错误3: 测试失败
```
test result: FAILED. 0 passed; 1 failed
```

**解决方案**:
```bash
# 运行测试查看详细错误
cargo test --verbose

# 运行特定测试
cargo test test_name
```

---

### 📦 **发布错误**

#### 错误1: Release创建失败
```
Error: Resource not accessible by integration
```

**原因**: 权限不足
**解决方案**:
1. 检查工作流权限设置
2. 确保有 `contents: write` 权限

#### 错误2: 产物上传失败
```
Error: Artifact not found
```

**原因**: 构建产物路径错误
**解决方案**:
1. 检查 `upload-artifact` 的路径设置
2. 确保构建步骤成功完成

---

## 🔍 **调试步骤**

### 1. 查看详细日志
1. 进入GitHub仓库
2. 点击 **Actions** 标签
3. 点击失败的工作流
4. 展开失败的步骤查看详细错误

### 2. 本地测试
```bash
# 测试Rust编译
cargo check
cargo test
cargo build --release

# 测试Docker构建
docker build -t test .

# 测试格式和Clippy
cargo fmt -- --check
cargo clippy -- -D warnings
```

### 3. 检查配置文件
- `Cargo.toml` - Rust项目配置
- `Dockerfile` - Docker构建配置
- `.github/workflows/` - GitHub Actions配置

---

## 🚨 **紧急修复指南**

### 如果构建完全失败：

1. **回滚到上一个工作版本**:
   ```bash
   git revert HEAD
   git push
   ```

2. **禁用失败的工作流**:
   - 进入 `.github/workflows/`
   - 重命名文件添加 `.disabled` 后缀

3. **简化配置**:
   - 移除复杂的优化选项
   - 使用基础的构建配置
   - 逐步添加功能

---

## 📋 **预防措施**

### 1. 本地测试
在推送前始终本地测试：
```bash
# 完整测试流程
cargo fmt
cargo clippy
cargo test
cargo build --release
docker build -t test .
```

### 2. 分步提交
- 不要一次性修改太多配置
- 每次只改一个功能
- 确保每次提交都能构建成功

### 3. 使用分支
```bash
# 创建功能分支
git checkout -b fix-build
# 修改和测试
git add .
git commit -m "fix: 修复构建问题"
git push origin fix-build
# 创建PR测试
```

---

## 🎯 **成功构建检查清单**

构建成功的标志：

- [ ] ✅ CI工作流通过 (绿色✓)
- [ ] ✅ 所有平台二进制文件生成
- [ ] ✅ Docker镜像推送成功
- [ ] ✅ Release页面有下载链接
- [ ] ✅ 没有权限错误
- [ ] ✅ 构建时间合理 (<30分钟)

---

## 📞 **获取帮助**

如果问题仍然存在：

1. **复制完整错误信息**
2. **检查相关配置文件**
3. **查看GitHub Actions文档**
4. **在社区寻求帮助**

记住：大多数构建问题都是配置问题，仔细检查配置通常能解决90%的问题！🎯
