# Priority Issues

- TODO: Add a `core` project to this workspace as a rust library. This project will be for storing core business logic and data models.
- TODO: Add a recipe struct to the `core` project that contains recipe information.
- TODO: Add rust traits to the `core` project for reading and writing recipe information.
- TODO: Add a `data_md` project to this workspace that implements the recipe read/write traits and can read and write from markdown files stored in the content folder. The data is made available at build time.
- TODO: Move the login button into the top header.
- TODO: Make the top header shorter.
- TODO: Make the sidebar pop in and out based on clicking a "hamburger" icon located in the header bar.
- TODO: Ensure the sorting of sidebar is sorted by numeric value rather than string. (ie Day 1, day 11, ... , day 2 is wrong) (it should be day 1, day 2, ... , day 11)
- TODO: Only show 10 entries in sidebar for recipes
- TODO: Only show 4 entries in sidebar for plans
- TODO: Start showing recipes based on today's date. The first recipe should be for the day of the year it is today and then going up from there.
- TODO: Start showing plans in the sidebar based on today's date. The first plan should be for the week of the year it is today then going up from there.