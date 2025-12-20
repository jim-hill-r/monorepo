




TODO: Move the docs folder into projects
TODO: Move all the projects up one level and remove the projects folder
TODO: Add a feature to the standards project that checks every directory in the monorepo and ensures they are snake_case. If they are not, it should add a TODO to the ISSUES.md to rename it. It should also output a json file with details on what percentage of directories are not snake_case and details on which ones are needing updating.
TODO: Instead of having a separate Cast.toml, just extend Cargo.toml. So the cast project should look for either a Cast.toml or a Cargo.toml
TODO: Add some context to the repo to let github copilot know that github workflows should have as little logic as possible. They should only run cast cli commands. If logic is required, it should be added to cast cli.
TODO: Refactor cast ci github workflow such that the majority of the logic exists within the cast project and not hardcoded scripts in the workflow yamls.
TODO: Move .github/agent-prompts to a new directory called prompts inside the agent-copilot project
TODO: Create a project called macos which provides all of the instructions to install required global dependencies onto a brand new macos machine. The dependencies should be rust and npm.
TODO: In github workflow start-a-new-task, the check for running agent tasks is not working. Even with an open draft PR from another agent it proceeds to make another task.
TODO: Refactor github workflow cast ci so that all of the logic for finding changes files in contained within the cast cli. The workflow should end up being two steps. Run `cast projects --with-changes` in root which would return any child directories with cast configuration that have source code changes and then run `cast ci` on those projects. 