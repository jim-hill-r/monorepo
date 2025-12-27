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
cd ../web
dx build --release
```

This will create build artifacts in `../web/dist/` directory.

## Deploying to Cloudflare Pages

### Deploy with Cast

The recommended way to deploy is using the Cast CLI:

```bash
cast deploy
```

This command will:
1. Verify the project is properly configured as an IAC project
2. Check that wrangler is installed
3. Load environment variables from `.env` file if present
4. Deploy using `wrangler pages deploy`

The Cast deploy command reads configuration from `wrangler.toml` and automatically uses the correct build output directory.

### Deploy with Wrangler Directly

Alternatively, you can deploy directly with wrangler:

```bash
wrangler pages deploy ../web/dist --project-name=cookbook
```

### Automated Deployment

For CI/CD pipelines, you can use Cast deploy with environment variables:

```bash
# Set your Cloudflare API token in .env file or environment
export CLOUDFLARE_API_TOKEN=<your-token>

# Deploy using Cast
cast deploy
```

## Project Structure

```
cookbook/cloudflare/
├── wrangler.toml     # Cloudflare Pages configuration
├── Cast.toml         # Cast monorepo metadata
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
cd ../web && dx build --release
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
