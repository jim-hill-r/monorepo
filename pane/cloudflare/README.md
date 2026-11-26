# cloudflare

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

> **Note:** Wrangler must be installed via npm. The cargo-based installation method is no longer supported.

### Authenticating with Cloudflare

Log in to your Cloudflare account:

```bash
wrangler login
```

## Building Pane

Before deploying, you need to build the `pane` project:

```bash
cd ../web
dx bundle --release
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
wrangler pages deploy ../web/dist --project-name=pane
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
pane-cloudflare/
├── wrangler.toml     # Cloudflare Pages configuration
├── mise.toml         # Tools and standard tasks (deploy builds ../web first)
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
- The pane project is properly linked

## Troubleshooting

### Build artifacts not found

Make sure you've built the pane project first:
```bash
cd ../web && dx bundle --release
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
