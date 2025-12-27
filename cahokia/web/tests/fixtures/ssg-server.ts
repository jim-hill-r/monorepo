import { test as base, expect } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as http from 'http';
import * as fs from 'fs';
import * as path from 'path';

const execAsync = promisify(exec);

// Default timeout for SSG bundle creation (10 minutes)
// Bundle creation is compute-intensive and can take several minutes
const DEFAULT_BUNDLE_TIMEOUT_MS = 600000;

// Default base port for SSG test servers
const DEFAULT_SSG_SERVER_PORT = 8090;

/**
 * Configuration for SSG Bundle Testing
 */
export interface SSGServerConfig {
  /** 
   * Base port for the test server. 
   * Each worker will use port + workerIndex (e.g., worker 0 uses port, worker 1 uses port+1).
   * Defaults to 8090 if not specified.
   */
  port?: number;
  /** Timeout for bundle creation in milliseconds. Defaults to 600000 (10 minutes). */
  bundleTimeout?: number;
  /** Whether to skip bundle creation (assumes bundle already exists). Defaults to false. */
  skipBundle?: boolean;
}

/**
 * SSG Server Fixture
 * 
 * This fixture provides a static HTTP server that serves the output from
 * `dx bundle --platform web --ssg`. It allows running existing Playwright
 * tests against the static bundle instead of the dev server.
 * 
 * Usage:
 * ```typescript
 * import { test, expect } from './fixtures/ssg-server';
 * 
 * test('should load home page on static bundle', async ({ page }) => {
 *   await page.goto('/');
 *   await expect(page).toHaveTitle(/.+/);
 * });
 * ```
 */

// Helper function to validate path is within allowed directory
// Uses path.relative to ensure the resolved path doesn't escape the base directory
function isPathSafe(requestedPath: string, baseDirectory: string): boolean {
  const resolvedBase = path.resolve(baseDirectory);
  const resolvedPath = path.resolve(requestedPath);
  
  // Get the relative path from base to resolved
  const relativePath = path.relative(resolvedBase, resolvedPath);
  
  // Safe if: relative path is empty (same as base) OR doesn't escape the base directory
  // Unsafe if: relative path starts with '..' (goes up) or is absolute (completely outside)
  return relativePath === '' || (!relativePath.startsWith('..') && !path.isAbsolute(relativePath));
}

// Helper function to serve a file with appropriate content type
function serveFile(filePath: string, data: Buffer, res: http.ServerResponse) {
  const ext = path.extname(filePath);
  const contentTypes: { [key: string]: string } = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.wasm': 'application/wasm',
  };
  
  const contentType = contentTypes[ext] || 'application/octet-stream';
  res.writeHead(200, { 'Content-Type': contentType });
  res.end(data);
}

// Helper function to create a simple static file server
function createStaticServer(directory: string, port: number): Promise<http.Server> {
  return new Promise((resolve, reject) => {
    const server = http.createServer((req, res) => {
      let requestPath: string;
      
      // Ensure req.url exists
      if (!req.url) {
        res.writeHead(400);
        res.end('Bad Request: Missing URL');
        return;
      }
      
      // Decode the URL, handling potential malformed URIs
      try {
        requestPath = decodeURIComponent(req.url === '/' ? '/index.html' : req.url);
      } catch (err) {
        res.writeHead(400);
        res.end('Bad Request: Malformed URL');
        return;
      }
      
      // Resolve and normalize paths for security
      const resolvedDirectory = path.resolve(directory);
      const filePath = path.resolve(path.join(resolvedDirectory, requestPath));
      
      // Security: prevent directory traversal
      if (!isPathSafe(filePath, resolvedDirectory)) {
        res.writeHead(403);
        res.end('Forbidden');
        return;
      }

      // Try to serve the file
      fs.readFile(filePath, (err, data) => {
        if (err) {
          // If file not found and it's not index.html, try adding .html extension
          if (err.code === 'ENOENT' && !filePath.endsWith('.html') && !filePath.includes('.')) {
            const htmlFilePath = path.join(path.dirname(filePath), path.basename(filePath) + '.html');
            // Verify the .html path is still within the directory
            if (!isPathSafe(htmlFilePath, resolvedDirectory)) {
              res.writeHead(403);
              res.end('Forbidden');
              return;
            }
            fs.readFile(htmlFilePath, (err2, data2) => {
              if (err2) {
                res.writeHead(404);
                res.end('Not Found');
              } else {
                serveFile(htmlFilePath, data2, res);
              }
            });
          } else {
            res.writeHead(404);
            res.end('Not Found');
          }
          return;
        }
        
        serveFile(filePath, data, res);
      });
    });

    server.listen(port, () => {
      console.log(`SSG Static server running on http://localhost:${port}`);
      resolve(server);
    });

    server.on('error', (err) => {
      reject(err);
    });
  });
}

/**
 * Build the SSG bundle using `dx bundle --platform web --ssg`
 */
async function buildSSGBundle(config: SSGServerConfig): Promise<string> {
  const bundleTimeout = config.bundleTimeout || DEFAULT_BUNDLE_TIMEOUT_MS;
  const bundleOutputDir = path.join(
    __dirname,
    '..',
    '..',
    '..',
    'target',
    'dx',
    'web',
    'release',
    'web',
    'public'
  );

  if (config.skipBundle) {
    console.log('Skipping bundle creation (skipBundle=true)');
    return bundleOutputDir;
  }

  // Check if dx is installed
  try {
    await execAsync('dx --version');
  } catch (error) {
    throw new Error('dx CLI not installed. Install with: cargo install dioxus-cli');
  }

  console.log('Building SSG bundle...');
  console.log('This may take several minutes on first run...');
  
  try {
    // Run dx from workspace root (cahokia) with -p flag to specify the web package
    // This avoids the workspace detection issue in dx 0.7.2
    const { stdout, stderr } = await execAsync(
      'dx bundle --platform web --ssg -p web',
      {
        cwd: path.join(__dirname, '..', '..', '..'),  // Run from cahokia directory (workspace root)
        timeout: bundleTimeout,
      }
    );
    
    console.log('Bundle stdout:', stdout);
    if (stderr) {
      console.log('Bundle stderr:', stderr);
    }
    
    console.log('Bundle creation completed');
    
    // Note: When running from workspace root, dx outputs to cahokia/target/dx/
    // but the public directory is still in the same relative location
    const workspaceBundle = path.join(
      __dirname,
      '..',
      '..',
      '..',
      'target',
      'dx',
      'web',
      'debug',  // Changed from 'release' to 'debug' as dx defaults to debug when no --release flag
      'web',
      'public'
    );
    
    // Check if the bundle exists
    if (fs.existsSync(workspaceBundle)) {
      return workspaceBundle;
    } else {
      // Fallback to original location if exists
      return bundleOutputDir;
    }
  } catch (error) {
    const err = error as Error;
    console.error('Failed to create SSG bundle:', err.message);
    
    // Check if this is an exec error with stdout/stderr properties
    if ('stdout' in err && typeof err.stdout === 'string') {
      console.log('stdout:', err.stdout);
    }
    if ('stderr' in err && typeof err.stderr === 'string') {
      console.log('stderr:', err.stderr);
    }
    
    throw err;
  }
}

/**
 * Extended test fixture with SSG server
 */
export const test = base.extend<{
  /** The port where the SSG static server is running */
  ssgPort: number;
  /** The base URL for the SSG server */
  ssgBaseURL: string;
}>({
  ssgPort: [DEFAULT_SSG_SERVER_PORT, { option: true }],
  
  ssgBaseURL: async ({ ssgPort }, use) => {
    await use(`http://localhost:${ssgPort}`);
  },
});

/**
 * Creates a worker-scoped fixture that sets up the SSG bundle and server once per worker.
 * This is more efficient for running multiple tests against the same bundle.
 */
export function createSSGWorkerFixture(config: SSGServerConfig = {}) {
  return base.extend<
    {
      ssgPort: number;
      ssgBaseURL: string;
    },
    {
      _ssgServer: http.Server;
      _ssgBundleDir: string;
    }
  >({
    // Worker-scoped fixtures that set up once per worker
    _ssgBundleDir: [
      async ({}, use, workerInfo) => {
        console.log(`Worker ${workerInfo.workerIndex}: Building SSG bundle...`);
        const bundleDir = await buildSSGBundle(config);
        
        // Verify the bundle output directory exists
        if (!fs.existsSync(bundleDir)) {
          throw new Error(`Bundle output directory does not exist: ${bundleDir}`);
        }

        // Verify index.html exists
        const indexPath = path.join(bundleDir, 'index.html');
        if (!fs.existsSync(indexPath)) {
          throw new Error(`index.html not found in bundle: ${indexPath}`);
        }
        
        await use(bundleDir);
      },
      { scope: 'worker', auto: true }
    ],

    _ssgServer: [
      async ({ _ssgBundleDir }, use, workerInfo) => {
        // If a custom port is provided, only the first worker can use it
        // Other workers must use the base port + worker index to avoid conflicts
        const port = config.port !== undefined 
          ? config.port + workerInfo.workerIndex
          : DEFAULT_SSG_SERVER_PORT + workerInfo.workerIndex;
        console.log(`Worker ${workerInfo.workerIndex}: Starting SSG server on port ${port}...`);
        
        const server = await createStaticServer(_ssgBundleDir, port);
        await use(server);
        
        // Cleanup: stop the server
        await new Promise<void>((resolve) => {
          server.close(() => {
            console.log(`Worker ${workerInfo.workerIndex}: SSG server stopped`);
            resolve();
          });
        });
      },
      { scope: 'worker', auto: true }
    ],

    // Test-scoped fixtures that provide values to each test
    ssgPort: async ({}, use, workerInfo) => {
      const port = config.port !== undefined
        ? config.port + workerInfo.workerIndex
        : DEFAULT_SSG_SERVER_PORT + workerInfo.workerIndex;
      await use(port);
    },

    ssgBaseURL: async ({ ssgPort }, use) => {
      await use(`http://localhost:${ssgPort}`);
    },
  });
}

export { expect };
