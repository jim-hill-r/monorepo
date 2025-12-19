   
TODO: Add documentation on how to use the `cross` rust package to compile the agent-copilot project for Linux x86_64 (comparible with ubuntu latest)
TODO: Improve readme of the pane project with directions on how to run the code locally in dev mode.
TODO: Create a new project called pane-cloudflare. Put it in the projects folder. This project is a deployment project for deployment the build artifacts from project pane onto cloudflare pages.   
TODO: Implement the project blueeel. Go to blue.eel.education and replicate the functionality there using Dioxus. For the initial implementation, just mock out any api requests by hardcoding the values in the project.   
TODO: In the cast project, add an optional configuration called exemplar which is true or false.
TODO: In the cast project, add an optional configuration called proof_of_concept which is true or false.
TODO: Move all proof_of_concept projects out of the proof_of_concepts directory and label them using a Cast.toml as proof_of_concept = true.
TODO: Refactor cast to find "exemplar" projects instead of using the templates.
TODO: Remove the templates directory
TODO: Move the docs folder into projects
TODO: Move all the projects up one level and remove the projects folder
TODO: Add a feature to the standards project that checks every directory in the monorepo and ensures they are snake_case. If they are not, it should add a TODO to the ISSUES.md to rename it. It should also output a json file with details on what percentage of directories are not snake_case and details on which ones are needing updating.
