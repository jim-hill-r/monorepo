# Blue Eel (blueeel) user guide

## Who it's for

blueeel is the start of a reading education app for learners of all ages:
students building their reading skills, people with dyslexia or other
reading challenges, and the parents and teachers who help them.

What ships today is one welcome page. It says what Blue Eel is for, names
the four areas the lessons will cover, and links to outside reading
support. There are no lessons, accounts or progress tracking yet, and the
app isn't published anywhere: you run it on your own computer.

Looking for the Blue Eel letter and word tracing app? That one is live, and
it lives in [sparkhill/webapp](../../../sparkhill/webapp/README.md), served
at <https://jimhillr-sparkhill-webapp.pages.dev>.

## Getting started

You need a copy of the repository and the
[mise](https://mise.jdx.dev) task runner.

1. From the repository, go to the project and install its tools (Rust, the
   web target and the Dioxus CLI):

   ```bash
   cd blueeel
   mise install
   ```

2. Start the app:

   ```bash
   mise run dev
   ```

3. Open <http://localhost:8080> in your browser. The page reloads by
   itself when the code changes.

## How to find your way around the page

- **Blue Eel** at the top left is the app's name. The **About** and
  **Lessons** buttons at the top right jump down the page to those
  sections.
- **Everyone deserves to read** is the welcome banner.
- **What We Do** (the About section) describes what Blue Eel offers.
- **Our Approach** (the Lessons section) shows the four areas the lessons
  will cover: Phonological Awareness (the sounds in words), Reading
  Comprehension (understanding what you read), Fluency Building (reading
  smoothly and with expression) and Vocabulary Development (learning new
  words). The cards describe each area; they don't open a lesson yet.

## How to find reading support now

The links at the bottom of the page open in a new tab:

- **Dyslexia Resources** goes to the International Dyslexia Association,
  <https://dyslexiaida.org>.
- **CHADD.org** goes to CHADD, the national resource on ADHD,
  <https://chadd.org>.

## Getting help

Common problems:

- **`mise: command not found`**: install mise first, from
  <https://mise.jdx.dev/getting-started.html>.
- **`dx: command not found`**: run `mise install` in the `blueeel` folder;
  it installs the Dioxus CLI.
- **The page doesn't load at localhost:8080**: wait for `mise run dev` to
  finish its first build (it can take a few minutes), and check nothing
  else is using port 8080.
- **A lesson card does nothing when clicked**: that's expected; lessons
  aren't built yet.

blueeel has no feedback button. To report a problem or ask for a feature,
tell Jim what you did and what you saw, and it will be recorded as an issue
in `blueeel/issues/`.
