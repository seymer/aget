# aget

基于 [age](https://age-encryption.org/) 的安全文件加密与销毁工具。

原地加密文件（原文件安全删除）、解密到临时目录（自动清理）、或通过多轮覆写安全销毁文件。

## 安装

```sh
cargo install --path .
```

或从 [Releases](https://github.com/seymer/aget/releases) 下载预编译二进制。

> **macOS：** 如果下载的二进制弹出 Gatekeeper 警告，运行：
> `xattr -d com.apple.quarantine ./aget`

## 使用

### seal — 加密文件

```sh
# 使用密码
aget seal secret.txt --passphrase

# 使用接收方公钥
aget seal secret.txt --recipient age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
```

加密后原文件被安全删除。输出：`secret.txt.age`

### open — 解密查看

```sh
# 密码加密的文件
aget open secret.txt.age

# 密钥加密的文件
aget open secret.txt.age --identity ~/.age/key.txt

# 非交互模式：解密后输出路径（供脚本/插件使用）
aget open secret.txt.age --no-wait
```

解密到临时目录。查看完毕按 Enter，明文将被安全删除。

使用 `--no-wait` 时，输出解密文件路径后立即退出。之后用 `cleanup` 安全删除。

### cleanup — 安全删除解密的临时文件

```sh
aget cleanup /tmp/.tmpXXXXXX/secret.txt
```

安全删除由 `open --no-wait` 创建的临时文件。供 yazi 插件查看后清理使用。

### destroy — 安全销毁文件

```sh
aget destroy file1.txt file2.txt
```

确认后，对每个文件进行 3 轮随机数据覆写 + 1 轮零覆写，然后删除。

### status — 查看加密状态

```sh
aget status .
aget status . --recursive
```

列出目录中的文件，显示哪些已加密（`.age`）、哪些是明文。使用 `-r` 递归扫描子目录。

## 安全删除机制

文件覆写流程：
1. 3 轮加密随机数据覆写
2. 1 轮全零覆写
3. 每轮后 `fsync` 刷盘
4. 删除文件

> **注意：** 在有磨损均衡的 SSD 上，覆写删除只能尽力而为。加密本身（age）才是核心保护。

## Yazi 集成

aget 附带 [yazi](https://yazi-rs.github.io/) 文件管理器插件。见 [`yazi/`](./yazi/) 目录：

- **theme.toml** — `.age` 文件显示 `󰈡` 图标（与 `.lock` 文件区分）
- **aget.yazi/main.lua** — 加密/解密插件（在 yazi 内弹窗输入密码）
- **keymap.toml** — `cs` 加密，`co` 解密

### 安装 yazi 集成

```sh
# 复制插件
cp -r yazi/aget.yazi ~/.config/yazi/plugins/

# 追加主题和快捷键配置
cat yazi/theme.toml >> ~/.config/yazi/theme.toml
cat yazi/keymap.toml >> ~/.config/yazi/keymap.toml
```

## 非交互使用

密码从 stdin 读取。脚本或插件集成时通过管道传入：

```sh
echo "mypassphrase" | aget seal secret.txt --passphrase
echo "mypassphrase" | aget open secret.txt.age --no-wait
```

这比环境变量更安全（环境变量可通过 `ps eww` 被其他进程看到）。

## 许可证

MIT
