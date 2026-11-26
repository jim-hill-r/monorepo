# web

The Cahokia web app, built with Dioxus.

## Pages

| Route | What it shows |
| --- | --- |
| `/` | Home page. |
| `/about` | About Cahokia. |
| `/history` | History of Cahokia. |
| `/explore` | Things to explore. |
| `/family-tree` | The sample family from `core::sample`, drawn with `ui::FamilyTreeView`. Linked from the sidebar. |

## Notes

- Authentication only with callback urls of `local.app.cahokia.com:8080`. Set hostfile for local host to ensure this is the callback url.