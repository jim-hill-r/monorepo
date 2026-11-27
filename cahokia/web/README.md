# web

The Cahokia genealogy app in the browser, built with Dioxus and served from
Cloudflare Pages by the [cloudflare](../cloudflare/README.md) crate. It is
named for the platform it targets: each Cahokia platform crate (`web`,
`desktop`, `mobile`) is named after where it runs. Pages every platform
shows come from the shared `ui` crate; this crate adds the web routes, the
sidebar and Auth0 sign-in through `auth_sdk`. See
[CONTRIBUTING.md](CONTRIBUTING.md) to build and test it.

## Pages

| Route | What it shows |
| --- | --- |
| `/` | Home page. |
| `/about` | About Cahokia. |
| `/history` | History of Cahokia. |
| `/explore` | Things to explore. |
| `/family-tree` | `ui::FamilyTreePage`: the sample family from `core::sample`, drawn with `ui::FamilyTreeView`. Linked from the sidebar. The desktop and mobile apps route the same path to the same page. |

## Notes

- Authentication only with callback urls of `local.app.cahokia.com:8080`. Set hostfile for local host to ensure this is the callback url.