
# dfl: A Git-Powered Dotfile Manager

[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/dfl.svg)](https://crates.io/crates/dfl)
[![Repo](https://img.shields.io/badge/github-aashish--thapa%2Fdfl-lightgrey.svg)](https://github.com/aashish-thapa/dfl)
[![docs.rs](https://img.shields.io/badge/docs-docs.rs-informational.svg)](https://docs.rs/dfl)

`dfl` is a simple, fast, and feature-rich command-line tool for managing your dotfiles with **Git**.
It automates moving, symlinking, and syncing your configuration files across multiple machines.

Whether you're setting up a fresh dev box or keeping your fleet consistent, `dfl` provides a seamless workflow.

---

## ✨ Features

- **Automated Workflow**: Uses Git under the hood to automatically add and commit files.
- **Centralized Repository**: Keeps all your dotfiles in one place (`~/.dfl`).
- **Easy Deployment**: A single command links all your configs on a new machine.
- **Built for Speed**: Written in Rust for a fast, reliable experience.

---

## 🚀 Installation

### 📦 Install from crates.io (recommended)

If you have Rust installed:

```bash
cargo install dfl
````

This installs the binary to `~/.cargo/bin`.
Be sure that directory is in your `PATH`.

---
## Screenshot
<img width="948" height="333" alt="image" src="https://github.com/user-attachments/assets/b525e7e9-b5a5-4110-af7a-7effc3ae31ef" />

### 🔧 Build from source

1. **Clone the repository**:

   ```bash
   git clone https://github.com/aashish-thapa/dfl.git
   cd dfl
   ```

2. **Build and install the executable**:

   ```bash
   cargo build --release
   sudo mv target/release/dfl /usr/local/bin/
   ```

---

## 💡 Usage

### 1) Initialize Your Repository

Create `~/.dfl` and initialize it as a Git repo:

```bash
dfl init
```

### 2) Add and Commit a Dotfile

Automatically move a file into the repo, create a symlink back, and commit:

```bash
dfl add ~/.bashrc
```

> Tip: Repeat `dfl add` for other files like `~/.zshrc`, `~/.gitconfig`, `~/.config/nvim/init.lua`, etc.

### 3) Deploy on a New Machine

Link everything from your `~/.dfl` repo into the home directory:

```bash
dfl link
```

(If your tool uses a different subcommand name like `deploy`, replace accordingly.)

---

## 🤝 Contributing

We welcome contributions!

1. Fork the repo
2. Create a feature branch: `git checkout -b feature/AmazingFeature`
3. Commit: `git commit -m 'feat: Add amazing feature'`
4. Push: `git push origin feature/AmazingFeature`
5. Open a Pull Request

---

## 📄 License

Licensed under the **MIT License**.
See the [LICENSE](LICENSE) file for details.

---

## 🙏 Credits

Created by [**aashish-thapa**](https://github.com/aashish-thapa).

```
```
