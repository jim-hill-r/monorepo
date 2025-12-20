
TODO: Update Start New AI Agent Task to check if agents are already running. The workflow should not start another one if 1 is already running.

TODO: Implement the basic scaffold for project blueeel. Implement in Dioxus the basics of an app that looks and feels like blue.eel.education. 

FIX: In the pane-cloudflare project, the README says to run `cargo install wrangler`, but it does not work with the error "error[E0599]: no function or associated item named `new` found for struct `tokio::runtime::Runtime` in the current scope"
TODO: In the cast project, add an optional configuration called proof_of_concept which is true or false.
TODO: Move all proof_of_concept projects out of the proof_of_concepts directory and label them using a Cast.toml as proof_of_concept = true.
TODO: Refactor cast to find "exemplar" projects instead of using the templates.
TODO: Remove the templates directory
TODO: Move the docs folder into projects
TODO: Move all the projects up one level and remove the projects folder
TODO: Add a feature to the standards project that checks every directory in the monorepo and ensures they are snake_case. If they are not, it should add a TODO to the ISSUES.md to rename it. It should also output a json file with details on what percentage of directories are not snake_case and details on which ones are needing updating.
TODO: Instead of having a separate Cast.toml, just extend Cargo.toml. So the cast project should look for either a Cast.toml or a Cargo.toml
TODO: Remove anything that is using Replit. I see at least one file .replit but there may be more.
TODO: Add some context to the repo to let github copilot know that github workflows should have as little logic as possible. They should only run cast cli commands. If logic is required, it should be added to cast cli.
TODO: Refactor cast ci github workflow such that the majority of the logic exists within the cast project and not hardcoded scripts in the workflow yamls.
