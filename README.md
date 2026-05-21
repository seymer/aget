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
```

Decrypts to a temporary directory. Press Enter when done — the plaintext is securely deleted.

### destroy — Securely delete files

```sh
aget destroy file1.txt file2.txt
```

Prompts for confirmation, then overwrites each file with 3 passes of random data + 1 pass of zeros before removing.

### status — Show encryption status

```sh
aget status .
```

Lists files in a directory, showing which are encrypted (`.age`) and which are plaintext.

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
- **aget.yazi/init.lua** — Plugin for seal/open actions
- **keymap.toml** — `es` to seal, `eo` to open

### Install yazi integration

```sh
# Copy plugin
cp -r yazi/aget.yazi ~/.config/yazi/plugins/

# Append theme and keymap configs
cat yazi/theme.toml >> ~/.config/yazi/theme.toml
cat yazi/keymap.toml >> ~/.config/yazi/keymap.toml
```

## License

MIT
