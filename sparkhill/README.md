# sparkhill

Sparkhill groups the Sparkhill website and the Blue Eel letter tracing app,
each with the project that deploys it to Cloudflare Pages. The folder itself
holds no code, only the shared decisions and issues.

| Project | Does |
| --- | --- |
| [`website/`](website/README.md) | The Sparkhill website, a static Perseus export. |
| [`cloudflare/`](cloudflare/README.md) | Deploys the website to <https://jimhillr-sparkhill.pages.dev>. |
| [`webapp/`](webapp/README.md) | Blue Eel, a Dioxus app for letter and word tracing practice. |
| [`webapp_cloudflare/`](webapp_cloudflare/README.md) | Deploys Blue Eel to <https://jimhillr-sparkhill-webapp.pages.dev>. |

Decisions are in [docs/decisions](docs/decisions/README.md) and open work
in [docs/issues/](docs/issues/). How to build and test each project is in
[CONTRIBUTING.md](CONTRIBUTING.md).
