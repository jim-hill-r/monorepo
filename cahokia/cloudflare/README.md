# cloudflare

This project handles the deployment of the `cahokia/web` Dioxus web application build artifacts to Cloudflare Pages.

This project is located at `cahokia/cloudflare` within the cahokia workspace.

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
cd ../web
dx build --release
```

This will create build artifacts in `../web/dist/` directory.

## Deploying to Cloudflare Pages

### Deploy with mise

```bash
mise run deploy
```

This runs `mise run release` in `../web` (a `dx bundle` into `../web/dist`)
and then `wrangler pages deploy`, which reads `wrangler.toml` for the project
name and output directory. Pushes to `main` that touch this project or its
web app deploy automatically; see `.github/workflows/push-main.yml`.

### Deploy with Wrangler Directly

Alternatively, you can deploy directly with wrangler:

```bash
wrangler pages deploy ../web/dist --project-name=jimhillr-cahokia
```

### Automated Deployment

CI deploys with the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`
repository secrets. To deploy from your machine without `wrangler login`, set
the same variables:

```bash
export CLOUDFLARE_API_TOKEN=<your-token>
export CLOUDFLARE_ACCOUNT_ID=<your-account-id>
mise run deploy
```

## Project Structure

```
cahokia/cloudflare/
├── wrangler.toml     # Cloudflare Pages configuration
├── mise.toml         # Tools and standard tasks (deploy builds ../web first)
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
