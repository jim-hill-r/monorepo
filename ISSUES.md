TODO: Move a project from the projects directory up one level into the root directory. Don't remove this TODO until there is nothing left in the projects directory.
TODO: Add some context to the repo to let github copilot know that github workflows should have as little logic as possible. They should only run cast cli commands. If logic is required, it should be added to cast cli.
TODO: Refactor cast ci github workflow such that the majority of the logic exists within the cast project and not hardcoded scripts in the workflow yamls.
TODO: In github workflow start-a-new-task, the check for running agent tasks is not working. Even with an open draft PR from another agent it proceeds to make another task.
