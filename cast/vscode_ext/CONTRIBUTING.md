# Contributing to cast_vscode_ext

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd cast/vscode_ext
cast install
```

This will automatically install:
- Node.js and npm
- VSCode Extension toolchain (vsce)

### Build

To compile the TypeScript source:

```bash
npm run compile
```

To watch for changes and recompile automatically:

```bash
npm run watch
```

### Test

To run linting:

```bash
npm run lint
```

To run all tests:

```bash
npm test
```

To run all CI checks:

```bash
cast ci
```

### Package

To create a `.vsix` installable extension file:

```bash
npm run package
```

This creates `cast.vsix` in the project directory.
