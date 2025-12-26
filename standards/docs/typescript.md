# TypeScript Standards

All TypeScript code in this repository must adhere to strict type-checking and follow these standards.

## Compiler Options

All `tsconfig.json` files MUST include the following compiler options:

### Core Strict Options
- `"strict": true` - Enable all strict type-checking options
- `"target": "ES2022"` - Target modern JavaScript features
- `"module": "commonjs"` or `"module": "Node16"` - Use appropriate module system
- `"moduleResolution": "node"` - Use Node.js module resolution

### Additional Required Options
- `"esModuleInterop": true` - Enable interoperability between CommonJS and ES Modules
- `"skipLibCheck": true` - Skip type checking of declaration files for faster compilation
- `"forceConsistentCasingInFileNames": true` - Ensure consistent file name casing
- `"resolveJsonModule": true` - Allow importing JSON modules

### Library Support
- `"lib": ["ES2022"]` - Include ES2022 standard library definitions

## Project Structure

### Required Files
Every TypeScript project MUST include:
- `tsconfig.json` - TypeScript compiler configuration
- `package.json` - Node.js package manifest

### Configuration Sections
A properly configured `tsconfig.json` MUST include:
- `"compilerOptions"` - Compiler options as specified above
- `"include"` - Array of file patterns to include (e.g., `["tests/**/*.ts", "src/**/*.ts"]`)
- `"exclude"` - Array of patterns to exclude (e.g., `["node_modules"]`)

## Standard Template

### For Playwright Test Projects

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "moduleResolution": "node",
    "lib": ["ES2022"],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "types": ["node", "@playwright/test"]
  },
  "include": [
    "tests/**/*.ts",
    "playwright.config.ts"
  ],
  "exclude": [
    "node_modules"
  ]
}
```

### For VS Code Extensions

```json
{
  "compilerOptions": {
    "module": "Node16",
    "target": "ES2022",
    "outDir": "out",
    "lib": ["ES2022"],
    "sourceMap": true,
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*.ts"],
  "exclude": ["node_modules"]
}
```

## Exceptions

- TypeScript is only permitted for:
  - VS Code extensions
  - Playwright test files
  - Configuration files that require TypeScript

All other code MUST be written in Rust as per the general language standards.
