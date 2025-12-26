# Cookbook Cloudflare Pages Deployment

This project handles the deployment of the `cookbook/web` Dioxus web application build artifacts to Cloudflare Pages.

## Prerequisites

Before deploying, you need:
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
- Cloudflare account with Pages access
- Built artifacts from the `cookbook/web` project

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

## Building Cookbook Web

Before deploying, you need to build the `cookbook/web` project:

```bash
cd ../cookbook/web
dx build --release
```

This will create build artifacts in `../cookbook/web/dist/` directory.

## Deploying to Cloudflare Pages

### Deploy with Wrangler

To deploy the cookbook/web build artifacts:

```bash
wrangler pages deploy ../cookbook/web/dist --project-name=cookbook
```

### Using the Deploy Script

For convenience, you can use the provided deployment script:

```bash
./deploy.sh
```

The script will:
1. Check for required tools (wrangler, dx)
2. Build the cookbook/web project if needed
3. Deploy to Cloudflare Pages

### Configuration

The deployment is configured via `wrangler.toml`. Key settings include:
- Project name: `cookbook`
- Build output directory: `../cookbook/web/dist`
- Branch deployments and environment settings

### Automated Deployment

For CI/CD pipelines, you can use:

```bash
# Set your Cloudflare API token
export CLOUDFLARE_API_TOKEN=<your-token>

# Deploy
wrangler pages deploy ../cookbook/web/dist --project-name=cookbook
```

## Project Structure

```
cookbook-cloudflare/
├── wrangler.toml     # Cloudflare Pages configuration
├── Cast.toml         # Cast monorepo metadata
├── deploy.sh         # Deployment script
├── tests/
│   └── validate.sh   # Validation test suite
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
- The cookbook/web project is properly linked

## Troubleshooting

### Build artifacts not found

Make sure you've built the cookbook/web project first:
```bash
cd ../cookbook/web && dx build --release
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
