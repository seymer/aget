# aget

[中文文档](./README_zh.md)

Secure file encryption and destruction tool powered by [age](https://age-encryption.org/).

Encrypt files in-place (original securely deleted), decrypt to a temporary location that auto-cleans, or securely destroy files with multi-pass overwrite.

## Install

```sh
cargo install --path .
```

Or download a prebuilt binary from [Releases](https://github.com/seymer/aget/releases).

> **macOS:** If you get a Gatekeeper warning on downloaded binaries, run:
> `xattr -d com.apple.quarantine ./aget`

## Usage

### seal — Encrypt a file

```sh
# With passphrase
aget seal secret.txt --passphrase

# With recipient public key
aget seal secret.txt --recipient age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p

# Keep original file (don't delete)
aget seal secret.txt --passphrase --keep

# Fast deletion (1 pass instead of default 3)
aget seal secret.txt --passphrase --passes 1
```

The original file is securely deleted after encryption (unless `--keep` is used). Output: `secret.txt.age`

### open — Decrypt a file

```sh
# Interactive: decrypt, view, then auto-cleanup
aget open secret.txt.age

# With identity file
aget open secret.txt.age --identity ~/.age/key.txt

# Decrypt to a specific directory (no auto-cleanup)
aget open secret.txt.age --output ./decrypted/
```

Without `--output`: decrypts to a temporary directory. Press Enter when done — the plaintext is securely deleted.

With `--output`: decrypts to the specified directory, prints the path to stdout, and exits. Use `destroy` to clean up later.

### destroy — Securely delete files

```sh
aget destroy file1.txt file2.txt

# Skip confirmation (for scripts/plugins)
aget destroy file.txt --no-confirm

# Control overwrite passes
aget destroy file.txt --no-confirm --passes 7
```

Prompts for confirmation (unless `--no-confirm`), then overwrites each file before removing.

### status — Show encryption status

```sh
aget status .
aget status . --recursive
```

Lists files in a directory, showing which are encrypted (`.age`) and which are plaintext. Use `-r` to scan subdirectories.

## Secure Deletion

Files are overwritten with:
1. N passes of cryptographically random data (default: 3, configurable via `--passes`)
2. 1 pass of zeros
3. `fsync` after each pass
4. File removed

> **Note:** On SSDs with wear-leveling, overwrite-based deletion is best-effort. The encryption itself (age) is the primary protection.

## Yazi Integration

aget ships with a [yazi](https://yazi-rs.github.io/) plugin for file-manager integration. See [`yazi/`](./yazi/) for:

- **theme.toml** — `󰈡` icon for `.age` files (distinct from `.lock` files)
- **aget.yazi/main.lua** — Plugin for seal/open actions (passphrase input via yazi popup)
- **keymap.toml** — `cs` to seal, `ck` to seal (keep), `co` to open, `cp` to peek, `cd` to destroy

### Install yazi integration

```sh
# Copy plugin
cp -r yazi/aget.yazi ~/.config/yazi/plugins/

# Append theme and keymap configs
cat yazi/theme.toml >> ~/.config/yazi/theme.toml
cat yazi/keymap.toml >> ~/.config/yazi/keymap.toml
```

## Non-interactive Usage

Passphrase is read from stdin. For scripting or plugin integration, pipe it in:

```sh
echo "mypassphrase" | aget seal secret.txt --passphrase
echo "mypassphrase" | aget open secret.txt.age --output ./out/
```

This is more secure than environment variables (which are visible to other processes via `ps eww`).

## License

MIT
