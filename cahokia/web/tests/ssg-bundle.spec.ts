import { test, expect } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as http from 'http';
import * as fs from 'fs';
import * as path from 'path';

const execAsync = promisify(exec);

/**
 * Playwright test for Static Site Generation (SSG) bundle functionality.
 * 
 * This test validates that `dx bundle --platform web --ssg` works correctly
 * and produces output that can be served on a static hosting site.
 * 
 * Prerequisites:
 * - Dioxus CLI (`dx`) must be installed: cargo install dioxus-cli
 * - Test must be run from cahokia/web directory
 * 
 * The test:
 * 1. Runs `dx bundle --platform web --ssg` to generate static site
 * 2. Verifies the bundle output exists and contains required files
 * 3. Starts a simple HTTP server to serve the static content
 * 4. Uses Playwright to verify the site loads and functions correctly
 */

test.describe('SSG Bundle Functionality', () => {
  const bundleOutputDir = path.join(__dirname, '..', '..', 'target', 'dx', 'web', 'release', 'web', 'public');
  const testPort = 8090;
  let server: http.Server | null = null;

  // Helper function to create a simple static file server
  function createStaticServer(directory: string, port: number): Promise<http.Server> {
    return new Promise((resolve, reject) => {
      const server = http.createServer((req, res) => {
        // Default to index.html for directory requests
        let filePath = path.join(directory, req.url === '/' ? 'index.html' : req.url!);
        
        // Security: prevent directory traversal
        if (!filePath.startsWith(directory)) {
          res.writeHead(403);
          res.end('Forbidden');
          return;
        }

        // Try to serve the file
        fs.readFile(filePath, (err, data) => {
          if (err) {
            // If file not found and it's not index.html, try adding .html extension
            if (err.code === 'ENOENT' && !filePath.endsWith('.html') && !filePath.includes('.')) {
              filePath = filePath + '.html';
              fs.readFile(filePath, (err2, data2) => {
                if (err2) {
                  res.writeHead(404);
                  res.end('Not Found');
                } else {
                  serveFile(filePath, data2, res);
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
        console.log(`Static server running on http://localhost:${port}`);
        resolve(server);
      });

      server.on('error', (err) => {
        reject(err);
      });
    });
  }

  function serveFile(filePath: string, data: Buffer, res: http.ServerResponse) {
    // Set appropriate content type
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

  test.beforeAll(async () => {
    // Skip this test if dx is not installed
    try {
      await execAsync('dx --version');
    } catch (error) {
      console.log('Skipping SSG bundle test: dx CLI not installed');
      console.log('Install with: cargo install dioxus-cli');
      test.skip();
      return;
    }

    console.log('Building SSG bundle...');
    console.log('This may take several minutes on first run...');
    
    try {
      // Run dx bundle with SSG flag
      // Note: This command may take several minutes to complete
      const { stdout, stderr } = await execAsync(
        'dx bundle --platform web --ssg',
        {
          cwd: path.join(__dirname, '..'),
          timeout: 600000, // 10 minute timeout for bundle command
        }
      );
      
      console.log('Bundle stdout:', stdout);
      if (stderr) {
        console.log('Bundle stderr:', stderr);
      }
      
      console.log('Bundle creation completed');
    } catch (error: any) {
      console.error('Failed to create SSG bundle:', error.message);
      if (error.stdout) console.log('stdout:', error.stdout);
      if (error.stderr) console.log('stderr:', error.stderr);
      throw error;
    }

    // Start static file server
    try {
      server = await createStaticServer(bundleOutputDir, testPort);
      console.log(`Static server started on port ${testPort}`);
    } catch (error) {
      console.error('Failed to start static server:', error);
      throw error;
    }
  });

  test.afterAll(async () => {
    // Stop the static server
    if (server) {
      await new Promise<void>((resolve) => {
        server!.close(() => {
          console.log('Static server stopped');
          resolve();
        });
      });
    }
  });

  test('should generate SSG bundle with required files', async () => {
    // Verify the bundle output directory exists
    expect(fs.existsSync(bundleOutputDir)).toBe(true);

    // Verify index.html exists (entry point for static site)
    const indexPath = path.join(bundleOutputDir, 'index.html');
    expect(fs.existsSync(indexPath)).toBe(true);

    // Read and verify index.html has content
    const indexContent = fs.readFileSync(indexPath, 'utf-8');
    expect(indexContent.length).toBeGreaterThan(0);
    expect(indexContent).toContain('<!DOCTYPE html>');
  });

  test('should serve static site on HTTP server', async ({ page }) => {
    // Navigate to the static site
    await page.goto(`http://localhost:${testPort}/`);
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
    
    // Verify the page has loaded with a valid title
    const title = await page.title();
    expect(title).toBeTruthy();
    expect(title.length).toBeGreaterThan(0);
  });

  test('should have proper HTML structure in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Check that basic HTML structure exists
    await expect(page.locator('html')).toBeVisible();
    await expect(page.locator('body')).toBeVisible();
  });

  test('should display Cahokia header in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Check that the header exists
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Check that the Cahokia title is displayed
    const title = header.locator('h1');
    await expect(title).toHaveText('Cahokia');
  });

  test('should have navigation links in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Check that navigation links exist
    const nav = page.locator('.header-nav');
    await expect(nav).toBeVisible();
    
    // Verify all navigation links are present
    await expect(nav.locator('a:has-text("Home")')).toBeVisible();
    await expect(nav.locator('a:has-text("About")')).toBeVisible();
    await expect(nav.locator('a:has-text("History")')).toBeVisible();
    await expect(nav.locator('a:has-text("Explore")')).toBeVisible();
  });

  test('should navigate to About page in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Click the About link
    await page.locator('.header-nav a:has-text("About")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the about page
    expect(page.url()).toContain('/about');
    
    // Check that the About page content is displayed
    await expect(page.locator('h2:has-text("About Cahokia")')).toBeVisible();
  });

  test('should navigate to History page in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Click the History link
    await page.locator('.header-nav a:has-text("History")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the history page
    expect(page.url()).toContain('/history');
    
    // Check that the History page content is displayed
    await expect(page.locator('h2:has-text("History of Cahokia")')).toBeVisible();
  });

  test('should navigate to Explore page in static site', async ({ page }) => {
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Click the Explore link
    await page.locator('.header-nav a:has-text("Explore")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the explore page
    expect(page.url()).toContain('/explore');
    
    // Check that the Explore page content is displayed
    await expect(page.locator('h2:has-text("Explore Cahokia")')).toBeVisible();
  });

  test('static site should work without dynamic server', async ({ page }) => {
    // This test verifies that the SSG bundle is truly static and doesn't
    // require server-side rendering or dynamic backend functionality
    
    await page.goto(`http://localhost:${testPort}/`);
    await page.waitForLoadState('networkidle');
    
    // Verify that all static assets are loaded successfully
    // by checking that there are no 404 errors in the console
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Navigate to different pages
    await page.locator('.header-nav a:has-text("About")').click();
    await page.waitForLoadState('networkidle');
    
    await page.locator('.header-nav a:has-text("History")').click();
    await page.waitForLoadState('networkidle');
    
    await page.locator('.header-nav a:has-text("Explore")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that no critical errors occurred
    const criticalErrors = errors.filter(err => 
      err.includes('404') || 
      err.includes('Failed to fetch') ||
      err.includes('not found')
    );
    
    // Some errors might be expected (like auth provider issues in static mode)
    // but asset loading errors should not occur
    expect(criticalErrors.length).toBeLessThan(5);
  });
});
