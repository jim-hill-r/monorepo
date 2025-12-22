# Cahokia Web Cloudflare Pages Deployment

This project handles the deployment of the `cahokia/web` Dioxus web application build artifacts to Cloudflare Pages.

## Prerequisites

Before deploying, you need:
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
- Cloudflare account with Pages access
- Built artifacts from the `cahokia/web` project

### Installing Wrangler

Install Wrangler globally using npm:

```bash
npm install -g wrangler
```

> **Note:** Wrangler must be installed via npm. The cargo-based installation method is no longer supported.

### Authenticating with Cloudflare

Log in to your Cloudflare account:

```bash
wrangler login
```

## Building Cahokia Web

Before deploying, you need to build the `cahokia/web` project:

```bash
cd ../cahokia/web
dx build --release
```

This will create build artifacts in `../cahokia/web/dist/` directory.

## Deploying to Cloudflare Pages

### Deploy with Wrangler

To deploy the cahokia web build artifacts:

```bash
wrangler pages deploy ../cahokia/web/dist --project-name=cahokia-web
```

### Configuration

The deployment is configured via `wrangler.toml`. Key settings include:
- Project name: `cahokia-web`
- Build output directory: `../cahokia/web/dist`
- Branch deployments and environment settings

### Automated Deployment

For CI/CD pipelines, you can use:

```bash
# Set your Cloudflare API token
export CLOUDFLARE_API_TOKEN=<your-token>

# Deploy
wrangler pages deploy ../cahokia/web/dist --project-name=cahokia-web
```

## Project Structure

```
cahokia_web_cloudflare/
├── wrangler.toml     # Cloudflare Pages configuration
├── Cast.toml         # Cast monorepo metadata
├── deploy.sh         # Deployment script
├── tests/
│   └── integration_tests.rs   # Validation test suite
└── README.md         # This file
```

## Testing

To validate the project configuration:

```bash
cargo test
```

This will verify:
- All required files exist
- Configuration files are valid
- Scripts have correct syntax
- The cahokia/web project is properly linked

## Troubleshooting

### Build artifacts not found

Make sure you've built the cahokia web project first:
```bash
cd ../cahokia/web && dx build --release
```

### Authentication issues

Ensure you're logged in to Cloudflare:
```bash
wrangler login
```

### Project doesn't exist

Create the project on Cloudflare Pages first, or let Wrangler create it automatically on first deploy.

## Links

- [Cloudflare Pages Documentation](https://developers.cloudflare.com/pages/)
- [Wrangler CLI Documentation](https://developers.cloudflare.com/workers/wrangler/)
- [Dioxus Documentation](https://dioxuslabs.com/)
