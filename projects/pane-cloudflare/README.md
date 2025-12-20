# Pane Cloudflare Pages Deployment

This project handles the deployment of the `pane` Dioxus web application build artifacts to Cloudflare Pages.

## Prerequisites

Before deploying, you need:
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
- Cloudflare account with Pages access
- Built artifacts from the `pane` project

### Installing Wrangler

Install Wrangler globally using npm:

```bash
npm install -g wrangler
```

Or using cargo:

```bash
cargo install wrangler
```

### Authenticating with Cloudflare

Log in to your Cloudflare account:

```bash
wrangler login
```

## Building Pane

Before deploying, you need to build the `pane` project:

```bash
cd ../pane
dx build --release
```

This will create build artifacts in `../pane/dist/` directory.

## Deploying to Cloudflare Pages

### Deploy with Wrangler

To deploy the pane build artifacts:

```bash
wrangler pages deploy ../pane/dist --project-name=pane
```

### Configuration

The deployment is configured via `wrangler.toml`. Key settings include:
- Project name: `pane`
- Build output directory: `../pane/dist`
- Branch deployments and environment settings

### Automated Deployment

For CI/CD pipelines, you can use:

```bash
# Set your Cloudflare API token
export CLOUDFLARE_API_TOKEN=<your-token>

# Deploy
wrangler pages deploy ../pane/dist --project-name=pane
```

## Project Structure

```
pane-cloudflare/
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
./tests/validate.sh
```

This will verify:
- All required files exist
- Configuration files are valid
- Scripts have correct syntax
- The pane project is properly linked

## Troubleshooting

### Build artifacts not found

Make sure you've built the pane project first:
```bash
cd ../pane && dx build --release
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
