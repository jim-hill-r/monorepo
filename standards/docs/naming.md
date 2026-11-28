- All projects MUST be snake_case.
- A project's directory name MUST either exactly match its Cargo package name, OR the package name MUST equal the parent directory name joined with the directory name using an underscore (e.g., a project at `cookbook/core` may use package name `cookbook_core`).
- All deployments MUST be kebab-case.
- All Proof of Concept projects must begin with poc (ie: poc_project_name)

`mise run //:audit` fails with NAM-001 on a project directory that isn't
snake_case, NAM-002 on a Cargo or npm package name that matches neither
rule above, and NAM-004 on a project whose package name or path says `poc`
or `proof` without the `poc_` prefix (`standards/scripts/project-check.sh`).

## Cloudflare Pages projects

Every Cloudflare Pages project MUST be named `jimhillr-<project>` (Jim,
2026-11-26), where `<project>` is the top-level project directory in
kebab-case: `cahokia/cloudflare` deploys `jimhillr-cahokia` and
`controlled_chaos` deploys `jimhillr-controlled-chaos`. The name is the
`name` in the project's `wrangler.toml`, and the site is served at
`https://jimhillr-<project>.pages.dev`, which is the project's `DEPLOY_URL`.
When one top-level project deploys a second site, the second Pages project
adds its sub-project directory in kebab-case: `sparkhill/cloudflare` deploys
the website as `jimhillr-sparkhill`, and `sparkhill/webapp_cloudflare`
deploys the Blue Eel web app (`sparkhill/webapp`) as
`jimhillr-sparkhill-webapp`.

A `*.pages.dev` subdomain is global across all Cloudflare accounts. When
another account already has `<name>.pages.dev`, Cloudflare quietly gives a
new project a random suffix instead (our first `pane` project got
`pane-dbc.pages.dev`), so the deploy URL no longer follows from the name. The
prefix keeps our names out of everyone else's way.

`free-tiers.sh check` (part of `mise run //:audit`) fails with FT-006 on a
Pages project without the prefix, and `free-tiers.sh pages-project` refuses
to create one.

## Requirement prefixes

Each project's product requirements use IDs made of a short prefix and a
number, such as `COOK-001` (see [evals.md](./evals.md)). A prefix is two to
five uppercase letters, belongs to one top-level project (its sub-projects
share it), and is never reused, even after the project is removed. Add a
row here before a project's first `docs/requirements.md`.

| Project | Prefix |
| --- | --- |
| athenaeum | `ATHN` |
| auth_sdk | `AUTH` |
| backups | `BACK` |
| bear_down | `BEAR` |
| blueeel | `BEEL` |
| bogan | `BOGN` |
| cahokia | `CAHO` |
| controlled_chaos | `CHAOS` |
| cookbook | `COOK` |
| design_system | `DSGN` |
| evals | `EVAL` |
| homer | `HOME` |
| jimhillr | `JIMH` |
| luggage | `LUGG` |
| my_lang | `LANG` |
| pane | `PANE` |
| profiles | `PROF` |
| psy_ents | `PSYE` |
| royalty | `ROYL` |
| sparkhill | `SPRK` |
| standards | `STND` |
| starcraft | `STAR` |
| telemetry | `TELE` |
| tofu_state | `TOFU` |

`EVL` is taken by the `evals check` finding codes (EVL-001 to EVL-006), so
it is not a project prefix. `JIMH`, the `jimhillr` project's prefix, covers
repo-wide process requirements such as change failure rate and lead time.
It replaced `MONO` when the `monorepo` folder was renamed, before any
`MONO` requirement existed.
