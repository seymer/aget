# aget

Secure file encryption and destruction tool powered by [age](https://age-encryption.org/).

Encrypt files in-place (original securely deleted), decrypt to a temporary location that auto-cleans, or securely destroy files with multi-pass overwrite.

## Install

```sh
cargo install --path .
```

## Usage

### seal — Encrypt a file

```sh
# With passphrase
aget seal secret.txt --passphrase

# With recipient public key
aget seal secret.txt --recipient age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
```

The original file is securely deleted after encryption. Output: `secret.txt.age`

### open — Decrypt and view

```sh
# Passphrase-encrypted file
aget open secret.txt.age

# Key-encrypted file
aget open secret.txt.age --identity ~/.age/key.txt

# Non-interactive: decrypt and print path (for scripts/plugins)
aget open secret.txt.age --no-wait
```

Decrypts to a temporary directory. Press Enter when done — the plaintext is securely deleted.

With `--no-wait`, prints the decrypted file path to stdout and exits immediately. Use `cleanup` to securely delete afterwards.

### cleanup — Securely delete a decrypted temp file

```sh
aget cleanup /tmp/.tmpXXXXXX/secret.txt
```

Securely deletes a file previously created by `open --no-wait`. Used by the yazi plugin to clean up after viewing.

### destroy — Securely delete files

```sh
aget destroy file1.txt file2.txt
```

Prompts for confirmation, then overwrites each file with 3 passes of random data + 1 pass of zeros before removing.

### status — Show encryption status

```sh
aget status .
aget status . --recursive
```

Lists files in a directory, showing which are encrypted (`.age`) and which are plaintext. Use `-r` to scan subdirectories.

## Secure Deletion

Files are overwritten with:
1. 3 passes of cryptographically random data
2. 1 pass of zeros
3. `fsync` after each pass
4. File removed

> **Note:** On SSDs with wear-leveling, overwrite-based deletion is best-effort. The encryption itself (age) is the primary protection.

## Yazi Integration

aget ships with a [yazi](https://yazi-rs.github.io/) plugin for file-manager integration. See [`yazi/`](./yazi/) for:

- **theme.toml** — `󰈡` icon for `.age` files (distinct from `.lock` files)
- **aget.yazi/main.lua** — Plugin for seal/open actions (passphrase input via yazi popup)
- **keymap.toml** — `cs` to seal, `co` to open

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
echo "mypassphrase" | aget open secret.txt.age --no-wait
```

This is more secure than environment variables (which are visible to other processes via `ps eww`).

## License

MIT
