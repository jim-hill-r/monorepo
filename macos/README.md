# macOS Setup Guide

This guide provides instructions for setting up a brand new macOS machine with the required global dependencies for this monorepo.

## Required Dependencies

This monorepo requires two primary global dependencies:
1. **Rust** - For building and running Rust projects
2. **npm** - For managing Node.js/JavaScript dependencies

## Installation Instructions

### 1. Install Rust

Rust is best installed using rustup, the official Rust toolchain installer.

#### Installation Steps:

1. Open Terminal
2. Run the rustup installation command:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. Follow the on-screen prompts (press Enter to proceed with default installation)
4. After installation, restart your terminal or run:
   ```bash
   source $HOME/.cargo/env
   ```
5. Verify the installation:
   ```bash
   rustc --version
   cargo --version
   ```

#### What Gets Installed:

- `rustc` - The Rust compiler
- `cargo` - Rust's package manager and build tool
- `rustup` - Rust toolchain manager
- Standard library and documentation

#### Additional Resources:

- Official documentation: https://www.rust-lang.org/tools/install
- Rustup book: https://rust-lang.github.io/rustup/

### 2. Install npm (via Node.js)

npm is the package manager for Node.js. The recommended way to install npm on macOS is to install Node.js, which includes npm.

#### Installation Steps:

There are several ways to install Node.js on macOS:

##### Option A: Using Homebrew (Recommended)

1. Install Homebrew if not already installed:
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
2. Install Node.js (which includes npm):
   ```bash
   brew install node
   ```
3. Verify the installation:
   ```bash
   node --version
   npm --version
   ```

##### Option B: Using Official Installer

1. Visit the Node.js download page: https://nodejs.org/
2. Download the macOS Installer (.pkg) for the LTS (Long Term Support) version
3. Run the installer and follow the prompts
4. Verify the installation:
   ```bash
   node --version
   npm --version
   ```

##### Option C: Using nvm (Node Version Manager)

For developers who need to manage multiple Node.js versions:

1. Install nvm:
   ```bash
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
   ```
2. Restart your terminal or run:
   ```bash
   source ~/.bashrc  # or ~/.zshrc for zsh
   ```
3. Install Node.js:
   ```bash
   nvm install --lts
   nvm use --lts
   ```
4. Verify the installation:
   ```bash
   node --version
   npm --version
   ```

#### Additional Resources:

- Node.js official site: https://nodejs.org/
- npm documentation: https://docs.npmjs.com/
- nvm repository: https://github.com/nvm-sh/nvm

## Verification

After installing both dependencies, verify that everything is set up correctly:

```bash
# Check Rust
rustc --version
cargo --version

# Check Node.js and npm
node --version
npm --version
```

You should see version numbers for all commands.

## Next Steps

After setting up these dependencies, you can:

1. Clone this repository
2. Follow the main README.md for repository-specific setup
3. Install the Cast CLI: `cargo install --path ./projects/cast_cli`
4. Start working on projects

## Additional Dependencies

Depending on which projects you work on, you may need additional tools:

- **SurrealDB** - For database projects
  - Install via: https://surrealdb.com/docs/surrealdb/installation/macos
- **cargo-binstall** - For faster cargo binary installations
  - Install via: https://github.com/cargo-bins/cargo-binstall
- **cross** - For cross-compilation
  - Install via: https://github.com/cross-rs/cross

These can be installed as needed for specific projects.

## Troubleshooting

### Rust Issues

- If `rustc` or `cargo` commands are not found after installation, ensure `~/.cargo/bin` is in your PATH
- Try restarting your terminal or running `source $HOME/.cargo/env`

### npm Issues

- If you encounter permission errors with npm, avoid using `sudo`. Instead, configure npm to use a different directory or use nvm
- For permission issues, see: https://docs.npmjs.com/resolving-eacces-permissions-errors-when-installing-packages-globally

### General Issues

- Ensure you have Xcode Command Line Tools installed: `xcode-select --install`
- Some installations may require accepting Xcode license: `sudo xcodebuild -license accept`

## Keeping Dependencies Updated

### Updating Rust

```bash
rustup update
```

### Updating npm/Node.js

With Homebrew:
```bash
brew upgrade node
```

With nvm:
```bash
nvm install --lts
nvm use --lts
```

## Support

If you encounter issues not covered in this guide:

1. Check the main repository README.md
2. Consult the official documentation for each tool
3. Search for or create an issue in the repository
