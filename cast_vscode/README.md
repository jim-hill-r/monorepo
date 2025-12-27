# Cast VS Code Extension

A Visual Studio Code extension for the Cast CLI tool - highly opinionated tooling for Rust monorepos.

## Features

### Session Timer Status Bar

The extension displays a running timer in the VS Code status bar that tracks your current Cast session. The timer shows elapsed time in `HH:MM:SS` format based on session logs stored in `.cast/sessions/`.

## Requirements

- Visual Studio Code 1.100.0 or higher
- Cast CLI tool (generates session logs in `.cast/sessions/`)

## Installation

### From VSIX File

1. Download the `cast.vsix` file from the repository
2. In VS Code, open the Command Palette (Ctrl+Shift+P / Cmd+Shift+P)
3. Run "Extensions: Install from VSIX..."
4. Select the downloaded `cast.vsix` file

### From Source

1. Clone this repository
2. Run `npm install` to install dependencies
3. Run `npm run compile` to compile the TypeScript code
4. Run `npm run package` to create the VSIX file
5. Install the generated `cast.vsix` file

## Usage

The extension activates automatically when VS Code starts. If a Cast session is active (session logs exist in `.cast/sessions/`), the status bar will display the elapsed time since the session started.

The timer updates every second and displays:
- Hours (2 digits)
- Minutes (2 digits)
- Seconds (2 digits)

## Development

### Building

```bash
npm run compile
```

### Watching for Changes

```bash
npm run watch
```

### Running Tests

```bash
npm test
```

### Packaging

```bash
npm run package
```

This creates a `cast.vsix` file that can be installed in VS Code.

## How It Works

The extension monitors the `.cast/sessions/` directory in your workspace for session log files. Each session log contains timestamps and events. The extension:

1. Reads the most recent session log file (lexicographically sorted, last file is selected)
2. Parses the log to find the session start time (first line with "Start" event)
3. Calculates elapsed time since the start
4. Updates the status bar every second

Session log format:
```
2024-01-01T12:00:00Z,Start
2024-01-01T12:30:00Z,Event
```

## Contributing

This extension is part of the Cast monorepo tooling. When making changes:

1. Update tests in the `src/test/` directory
2. Run linting with `npm run lint`
3. Update this README if adding new features
4. Update `CHANGELOG.md` with your changes

## Release Notes

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## License

See the repository root LICENSE.md file for license information.
