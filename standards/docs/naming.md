- All projects MUST be snake_case.
- A project's directory name MUST either exactly match its Cargo package name, OR the package name MUST equal the parent directory name joined with the directory name using an underscore (e.g., a project at `cookbook/core` may use package name `cookbook_core`).
- All deployments MUST be kebab-case.
- All Proof of Concept projects must begin with poc (ie: poc_project_name)

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
| bear_down | `BEAR` |
| blueeel | `BEEL` |
| bogan | `BOGN` |
| cahokia | `CAHO` |
| controlled_chaos | `CHAOS` |
| cookbook | `COOK` |
| evals | `EVAL` |
| homer | `HOME` |
| luggage | `LUGG` |
| monorepo | `MONO` |
| my_lang | `LANG` |
| pane | `PANE` |
| profiles | `PROF` |
| psy_ents | `PSYE` |
| royalty | `ROYL` |
| sparkhill | `SPRK` |
| standards | `STND` |
| starcraft | `STAR` |

`EVL` is taken by the `evals check` finding codes (EVL-001 to EVL-004), so
it is not a project prefix. `MONO` covers repo-wide process requirements
such as change failure rate and lead time.
