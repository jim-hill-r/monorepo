# SSG Server Fixture

This fixture provides a reusable way to run Playwright tests against the static assets built by `dx bundle --platform web --ssg` instead of requiring a dev server.

## Features

- **Automatic Bundle Creation**: Builds the SSG bundle once per worker
- **Static HTTP Server**: Serves the static assets with proper content types
- **Worker-Scoped Setup**: Efficient for running multiple tests (builds bundle once, reuses across tests)
- **Automatic Cleanup**: Stops server and cleans up resources after tests complete
- **Port Configuration**: Supports custom port configuration
- **Path Security**: Includes directory traversal protection

## Usage

### Basic Usage

```typescript
import { expect, createSSGWorkerFixture } from './fixtures/ssg-server';

// Create a test instance with SSG server fixture
const test = createSSGWorkerFixture({
  port: 8091, // Optional: Choose an available port (default: 8090 + workerIndex)
  bundleTimeout: 600000, // Optional: Timeout for bundle creation (default: 10 minutes)
  skipBundle: false, // Optional: Skip bundle creation if bundle already exists (default: false)
});

test.describe('My SSG Tests', () => {
  // Override the base URL to use the SSG server
  test.use({ baseURL: 'http://localhost:8091' });

  test('should load the home page', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/.+/);
  });

  test('should navigate to about page', async ({ page }) => {
    await page.goto('/');
    await page.locator('a:has-text("About")').click();
    expect(page.url()).toContain('/about');
  });
});
```

### Converting Existing Tests

To convert an existing test file to use the SSG fixture:

1. Change the import from `@playwright/test` to the fixture:
   ```typescript
   // Before:
   import { test, expect } from '@playwright/test';
   
   // After:
   import { expect, createSSGWorkerFixture } from './fixtures/ssg-server';
   const test = createSSGWorkerFixture({ port: 8091 });
   ```

2. Add `test.use({ baseURL: 'http://localhost:PORT' })` to override the base URL:
   ```typescript
   test.describe('My Tests', () => {
     test.use({ baseURL: 'http://localhost:8091' });
     // ... tests
   });
   ```

3. Run your tests - the fixture will handle bundle creation and server setup!

## Configuration Options

### SSGServerConfig

- **port** (number, optional): Base port for the test server. Each parallel worker will use `port + workerIndex` to avoid conflicts (e.g., if port is 9000, worker 0 uses 9000, worker 1 uses 9001, etc.). Default: `8090`
- **bundleTimeout** (number, optional): Timeout for bundle creation in milliseconds. Default: `600000` (10 minutes)
- **skipBundle** (boolean, optional): Whether to skip bundle creation and use existing bundle. Default: `false`

### Example with Custom Configuration

```typescript
const test = createSSGWorkerFixture({
  port: 9000, // Worker 0 will use 9000, worker 1 will use 9001, etc.
  bundleTimeout: 300000, // 5 minutes
  skipBundle: false,
});
```

## How It Works

1. **Bundle Creation** (once per worker):
   - Runs `dx bundle --platform web --ssg` to generate static site
   - Verifies the bundle output directory and index.html exist
   - Stores the bundle directory path for the worker

2. **Server Setup** (once per worker):
   - Creates a static HTTP server that serves files from the bundle directory
   - Implements security features (path traversal protection)
   - Handles different content types (HTML, JS, CSS, images, WASM, etc.)
   - Supports clean URL routing (e.g., `/about` → `/about.html`)

3. **Test Execution** (per test):
   - Each test receives the server port and base URL
   - Tests run normally using `page.goto()` etc.
   - Multiple tests share the same server instance

4. **Cleanup** (once per worker):
   - Server is stopped after all tests in the worker complete
   - Resources are cleaned up automatically

## Parallel Execution

The fixture supports parallel test execution:

```bash
# Run tests in parallel (Playwright default)
npm test -- example-ssg.spec.ts

# Each worker gets its own server on a different port (port + workerIndex)
# With default port (8090):
#   Worker 0: port 8090
#   Worker 1: port 8091
#   Worker 2: port 8092
# With custom port (e.g., 9000):
#   Worker 0: port 9000
#   Worker 1: port 9001
#   Worker 2: port 9002
```

## Prerequisites

- **Dioxus CLI (`dx`)**: Required for building the SSG bundle
  ```bash
  cargo install dioxus-cli
  ```
- **Node.js 18+**: Required for running Playwright tests
- **Playwright**: Install with `npm install`

## Examples

See the following test files for examples:

- `example-ssg.spec.ts` - Complete example showing how to convert standard tests to use SSG fixture
- `ssg-bundle.spec.ts` - Original SSG test (can be refactored to use this fixture)

## Tips

- **Bundle Creation**: The first run takes longer (several minutes) because it builds the SSG bundle. Subsequent runs are faster if you don't change the code.
- **Port Conflicts**: If you get port conflicts, specify a different port in the fixture configuration
- **Debugging**: Use `npm run test:headed` to see the browser while tests run
- **Skip Bundle**: If you've already built the bundle, use `skipBundle: true` to save time during development

## Troubleshooting

### "dx CLI not installed"
Install the Dioxus CLI: `cargo install dioxus-cli`

### "Bundle output directory does not exist"
The bundle creation failed. Check the console output for errors from `dx bundle`.

### "Port already in use"
Change the port in the fixture configuration or stop the process using that port.

### Tests timeout or fail
- Ensure the bundle built successfully
- Check that the static server started on the expected port
- Verify your tests are using the correct base URL
