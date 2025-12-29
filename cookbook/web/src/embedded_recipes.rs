use cookbook_core::{Recipe, RecipeError, RecipeReader, RecipeResult};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Embedded recipe store that includes all recipe markdown files at compile time.
/// This is suitable for WASM/web targets where filesystem access is not available.
pub struct EmbeddedRecipeStore {
    recipes: HashMap<String, Recipe>,
}

static EMBEDDED_STORE: OnceLock<EmbeddedRecipeStore> = OnceLock::new();

impl EmbeddedRecipeStore {
    /// Get the global embedded recipe store instance
    pub fn global() -> &'static EmbeddedRecipeStore {
        EMBEDDED_STORE.get_or_init(|| {
            let mut store = EmbeddedRecipeStore {
                recipes: HashMap::new(),
            };
            store.load_embedded_recipes();
            store
        })
    }

    /// Load all embedded recipes at initialization
    fn load_embedded_recipes(&mut self) {
        // Include all recipe files at compile time
        let recipe_files: Vec<(u32, &str)> = vec![
            (1, include_str!("../../content/day-1.md")),
            (2, include_str!("../../content/day-2.md")),
            (3, include_str!("../../content/day-3.md")),
            (4, include_str!("../../content/day-4.md")),
            (5, include_str!("../../content/day-5.md")),
            (6, include_str!("../../content/day-6.md")),
            (7, include_str!("../../content/day-7.md")),
            (8, include_str!("../../content/day-8.md")),
            (9, include_str!("../../content/day-9.md")),
            (10, include_str!("../../content/day-10.md")),
            (11, include_str!("../../content/day-11.md")),
            (12, include_str!("../../content/day-12.md")),
            (13, include_str!("../../content/day-13.md")),
            (14, include_str!("../../content/day-14.md")),
            (15, include_str!("../../content/day-15.md")),
            (16, include_str!("../../content/day-16.md")),
            (17, include_str!("../../content/day-17.md")),
            (18, include_str!("../../content/day-18.md")),
            (19, include_str!("../../content/day-19.md")),
            (20, include_str!("../../content/day-20.md")),
            (21, include_str!("../../content/day-21.md")),
            (22, include_str!("../../content/day-22.md")),
            (23, include_str!("../../content/day-23.md")),
            (24, include_str!("../../content/day-24.md")),
            (25, include_str!("../../content/day-25.md")),
            (26, include_str!("../../content/day-26.md")),
            (27, include_str!("../../content/day-27.md")),
            (28, include_str!("../../content/day-28.md")),
            (29, include_str!("../../content/day-29.md")),
            (30, include_str!("../../content/day-30.md")),
            (31, include_str!("../../content/day-31.md")),
            (32, include_str!("../../content/day-32.md")),
            (33, include_str!("../../content/day-33.md")),
            (34, include_str!("../../content/day-34.md")),
            (35, include_str!("../../content/day-35.md")),
            (36, include_str!("../../content/day-36.md")),
            (37, include_str!("../../content/day-37.md")),
            (38, include_str!("../../content/day-38.md")),
            (39, include_str!("../../content/day-39.md")),
            (40, include_str!("../../content/day-40.md")),
            (41, include_str!("../../content/day-41.md")),
            (42, include_str!("../../content/day-42.md")),
            (43, include_str!("../../content/day-43.md")),
            (44, include_str!("../../content/day-44.md")),
            (45, include_str!("../../content/day-45.md")),
            (46, include_str!("../../content/day-46.md")),
            (47, include_str!("../../content/day-47.md")),
            (48, include_str!("../../content/day-48.md")),
            (49, include_str!("../../content/day-49.md")),
            (50, include_str!("../../content/day-50.md")),
            (51, include_str!("../../content/day-51.md")),
            (52, include_str!("../../content/day-52.md")),
            (53, include_str!("../../content/day-53.md")),
            (54, include_str!("../../content/day-54.md")),
            (55, include_str!("../../content/day-55.md")),
            (56, include_str!("../../content/day-56.md")),
            (57, include_str!("../../content/day-57.md")),
            (58, include_str!("../../content/day-58.md")),
            (59, include_str!("../../content/day-59.md")),
            (60, include_str!("../../content/day-60.md")),
            (61, include_str!("../../content/day-61.md")),
            (62, include_str!("../../content/day-62.md")),
            (63, include_str!("../../content/day-63.md")),
            (64, include_str!("../../content/day-64.md")),
            (65, include_str!("../../content/day-65.md")),
            (66, include_str!("../../content/day-66.md")),
            (67, include_str!("../../content/day-67.md")),
            (68, include_str!("../../content/day-68.md")),
            (69, include_str!("../../content/day-69.md")),
            (70, include_str!("../../content/day-70.md")),
            (71, include_str!("../../content/day-71.md")),
            (72, include_str!("../../content/day-72.md")),
            (73, include_str!("../../content/day-73.md")),
            (74, include_str!("../../content/day-74.md")),
            (75, include_str!("../../content/day-75.md")),
            (76, include_str!("../../content/day-76.md")),
            (77, include_str!("../../content/day-77.md")),
            (78, include_str!("../../content/day-78.md")),
            (79, include_str!("../../content/day-79.md")),
            (80, include_str!("../../content/day-80.md")),
            (81, include_str!("../../content/day-81.md")),
            (82, include_str!("../../content/day-82.md")),
            (83, include_str!("../../content/day-83.md")),
            (84, include_str!("../../content/day-84.md")),
            (85, include_str!("../../content/day-85.md")),
            (86, include_str!("../../content/day-86.md")),
            (87, include_str!("../../content/day-87.md")),
            (88, include_str!("../../content/day-88.md")),
            (89, include_str!("../../content/day-89.md")),
            (90, include_str!("../../content/day-90.md")),
            (91, include_str!("../../content/day-91.md")),
            (92, include_str!("../../content/day-92.md")),
            (93, include_str!("../../content/day-93.md")),
            (94, include_str!("../../content/day-94.md")),
            (95, include_str!("../../content/day-95.md")),
            (96, include_str!("../../content/day-96.md")),
            (97, include_str!("../../content/day-97.md")),
            (98, include_str!("../../content/day-98.md")),
            (99, include_str!("../../content/day-99.md")),
            (100, include_str!("../../content/day-100.md")),
            (101, include_str!("../../content/day-101.md")),
            (102, include_str!("../../content/day-102.md")),
            (103, include_str!("../../content/day-103.md")),
            (104, include_str!("../../content/day-104.md")),
            (105, include_str!("../../content/day-105.md")),
            (106, include_str!("../../content/day-106.md")),
            (107, include_str!("../../content/day-107.md")),
            (108, include_str!("../../content/day-108.md")),
            (109, include_str!("../../content/day-109.md")),
            (110, include_str!("../../content/day-110.md")),
            (111, include_str!("../../content/day-111.md")),
            (112, include_str!("../../content/day-112.md")),
            (113, include_str!("../../content/day-113.md")),
            (114, include_str!("../../content/day-114.md")),
            (115, include_str!("../../content/day-115.md")),
            (116, include_str!("../../content/day-116.md")),
            (117, include_str!("../../content/day-117.md")),
            (118, include_str!("../../content/day-118.md")),
            (119, include_str!("../../content/day-119.md")),
            (120, include_str!("../../content/day-120.md")),
            (121, include_str!("../../content/day-121.md")),
            (122, include_str!("../../content/day-122.md")),
            (123, include_str!("../../content/day-123.md")),
            (124, include_str!("../../content/day-124.md")),
            (125, include_str!("../../content/day-125.md")),
            (126, include_str!("../../content/day-126.md")),
            (127, include_str!("../../content/day-127.md")),
            (128, include_str!("../../content/day-128.md")),
            (129, include_str!("../../content/day-129.md")),
            (130, include_str!("../../content/day-130.md")),
            (131, include_str!("../../content/day-131.md")),
            (132, include_str!("../../content/day-132.md")),
            (133, include_str!("../../content/day-133.md")),
            (134, include_str!("../../content/day-134.md")),
            (135, include_str!("../../content/day-135.md")),
            (136, include_str!("../../content/day-136.md")),
            (137, include_str!("../../content/day-137.md")),
            (138, include_str!("../../content/day-138.md")),
            (139, include_str!("../../content/day-139.md")),
            (140, include_str!("../../content/day-140.md")),
            (141, include_str!("../../content/day-141.md")),
            (142, include_str!("../../content/day-142.md")),
            (143, include_str!("../../content/day-143.md")),
            (144, include_str!("../../content/day-144.md")),
            (145, include_str!("../../content/day-145.md")),
            (146, include_str!("../../content/day-146.md")),
            (147, include_str!("../../content/day-147.md")),
            (148, include_str!("../../content/day-148.md")),
            (149, include_str!("../../content/day-149.md")),
            (150, include_str!("../../content/day-150.md")),
            (151, include_str!("../../content/day-151.md")),
            (152, include_str!("../../content/day-152.md")),
            (153, include_str!("../../content/day-153.md")),
            (154, include_str!("../../content/day-154.md")),
            (155, include_str!("../../content/day-155.md")),
            (156, include_str!("../../content/day-156.md")),
            (157, include_str!("../../content/day-157.md")),
            (158, include_str!("../../content/day-158.md")),
            (159, include_str!("../../content/day-159.md")),
            (160, include_str!("../../content/day-160.md")),
            (161, include_str!("../../content/day-161.md")),
            (162, include_str!("../../content/day-162.md")),
            (163, include_str!("../../content/day-163.md")),
            (164, include_str!("../../content/day-164.md")),
            (165, include_str!("../../content/day-165.md")),
            (166, include_str!("../../content/day-166.md")),
            (167, include_str!("../../content/day-167.md")),
            (168, include_str!("../../content/day-168.md")),
            (169, include_str!("../../content/day-169.md")),
            (170, include_str!("../../content/day-170.md")),
            (171, include_str!("../../content/day-171.md")),
            (172, include_str!("../../content/day-172.md")),
            (173, include_str!("../../content/day-173.md")),
            (174, include_str!("../../content/day-174.md")),
            (175, include_str!("../../content/day-175.md")),
            (176, include_str!("../../content/day-176.md")),
            (177, include_str!("../../content/day-177.md")),
            (178, include_str!("../../content/day-178.md")),
            (179, include_str!("../../content/day-179.md")),
            (180, include_str!("../../content/day-180.md")),
            (181, include_str!("../../content/day-181.md")),
            (182, include_str!("../../content/day-182.md")),
            (183, include_str!("../../content/day-183.md")),
            (184, include_str!("../../content/day-184.md")),
            (185, include_str!("../../content/day-185.md")),
            (186, include_str!("../../content/day-186.md")),
            (187, include_str!("../../content/day-187.md")),
            (188, include_str!("../../content/day-188.md")),
            (189, include_str!("../../content/day-189.md")),
            (190, include_str!("../../content/day-190.md")),
            (191, include_str!("../../content/day-191.md")),
            (192, include_str!("../../content/day-192.md")),
            (193, include_str!("../../content/day-193.md")),
            (194, include_str!("../../content/day-194.md")),
            (195, include_str!("../../content/day-195.md")),
            (196, include_str!("../../content/day-196.md")),
            (197, include_str!("../../content/day-197.md")),
            (198, include_str!("../../content/day-198.md")),
            (199, include_str!("../../content/day-199.md")),
            (200, include_str!("../../content/day-200.md")),
            (201, include_str!("../../content/day-201.md")),
            (202, include_str!("../../content/day-202.md")),
            (203, include_str!("../../content/day-203.md")),
            (204, include_str!("../../content/day-204.md")),
            (205, include_str!("../../content/day-205.md")),
            (206, include_str!("../../content/day-206.md")),
            (207, include_str!("../../content/day-207.md")),
            (208, include_str!("../../content/day-208.md")),
            (209, include_str!("../../content/day-209.md")),
            (210, include_str!("../../content/day-210.md")),
            (211, include_str!("../../content/day-211.md")),
            (212, include_str!("../../content/day-212.md")),
            (213, include_str!("../../content/day-213.md")),
            (214, include_str!("../../content/day-214.md")),
            (215, include_str!("../../content/day-215.md")),
            (216, include_str!("../../content/day-216.md")),
            (217, include_str!("../../content/day-217.md")),
            (218, include_str!("../../content/day-218.md")),
            (219, include_str!("../../content/day-219.md")),
            (220, include_str!("../../content/day-220.md")),
            (221, include_str!("../../content/day-221.md")),
            (222, include_str!("../../content/day-222.md")),
            (223, include_str!("../../content/day-223.md")),
            (224, include_str!("../../content/day-224.md")),
            (225, include_str!("../../content/day-225.md")),
            (226, include_str!("../../content/day-226.md")),
            (227, include_str!("../../content/day-227.md")),
            (228, include_str!("../../content/day-228.md")),
            (229, include_str!("../../content/day-229.md")),
            (230, include_str!("../../content/day-230.md")),
            (231, include_str!("../../content/day-231.md")),
            (232, include_str!("../../content/day-232.md")),
            (233, include_str!("../../content/day-233.md")),
            (234, include_str!("../../content/day-234.md")),
            (235, include_str!("../../content/day-235.md")),
            (236, include_str!("../../content/day-236.md")),
            (237, include_str!("../../content/day-237.md")),
            (238, include_str!("../../content/day-238.md")),
            (239, include_str!("../../content/day-239.md")),
            (240, include_str!("../../content/day-240.md")),
            (241, include_str!("../../content/day-241.md")),
            (242, include_str!("../../content/day-242.md")),
            (243, include_str!("../../content/day-243.md")),
            (244, include_str!("../../content/day-244.md")),
            (245, include_str!("../../content/day-245.md")),
            (246, include_str!("../../content/day-246.md")),
            (247, include_str!("../../content/day-247.md")),
            (248, include_str!("../../content/day-248.md")),
            (249, include_str!("../../content/day-249.md")),
            (250, include_str!("../../content/day-250.md")),
            (251, include_str!("../../content/day-251.md")),
            (252, include_str!("../../content/day-252.md")),
            (253, include_str!("../../content/day-253.md")),
            (254, include_str!("../../content/day-254.md")),
            (255, include_str!("../../content/day-255.md")),
            (256, include_str!("../../content/day-256.md")),
            (257, include_str!("../../content/day-257.md")),
            (258, include_str!("../../content/day-258.md")),
            (259, include_str!("../../content/day-259.md")),
            (260, include_str!("../../content/day-260.md")),
            (261, include_str!("../../content/day-261.md")),
            (262, include_str!("../../content/day-262.md")),
            (263, include_str!("../../content/day-263.md")),
            (264, include_str!("../../content/day-264.md")),
            (265, include_str!("../../content/day-265.md")),
            (266, include_str!("../../content/day-266.md")),
            (267, include_str!("../../content/day-267.md")),
            (268, include_str!("../../content/day-268.md")),
            (269, include_str!("../../content/day-269.md")),
            (270, include_str!("../../content/day-270.md")),
            (271, include_str!("../../content/day-271.md")),
            (272, include_str!("../../content/day-272.md")),
            (273, include_str!("../../content/day-273.md")),
            (274, include_str!("../../content/day-274.md")),
            (275, include_str!("../../content/day-275.md")),
            (276, include_str!("../../content/day-276.md")),
            (277, include_str!("../../content/day-277.md")),
            (278, include_str!("../../content/day-278.md")),
            (279, include_str!("../../content/day-279.md")),
            (280, include_str!("../../content/day-280.md")),
            (281, include_str!("../../content/day-281.md")),
            (282, include_str!("../../content/day-282.md")),
            (283, include_str!("../../content/day-283.md")),
            (284, include_str!("../../content/day-284.md")),
            (285, include_str!("../../content/day-285.md")),
            (286, include_str!("../../content/day-286.md")),
            (287, include_str!("../../content/day-287.md")),
            (288, include_str!("../../content/day-288.md")),
            (289, include_str!("../../content/day-289.md")),
            (290, include_str!("../../content/day-290.md")),
            (291, include_str!("../../content/day-291.md")),
            (292, include_str!("../../content/day-292.md")),
            (293, include_str!("../../content/day-293.md")),
            (294, include_str!("../../content/day-294.md")),
            (295, include_str!("../../content/day-295.md")),
            (296, include_str!("../../content/day-296.md")),
            (297, include_str!("../../content/day-297.md")),
            (298, include_str!("../../content/day-298.md")),
            (299, include_str!("../../content/day-299.md")),
            (300, include_str!("../../content/day-300.md")),
            (301, include_str!("../../content/day-301.md")),
            (302, include_str!("../../content/day-302.md")),
            (303, include_str!("../../content/day-303.md")),
            (304, include_str!("../../content/day-304.md")),
            (305, include_str!("../../content/day-305.md")),
            (306, include_str!("../../content/day-306.md")),
            (307, include_str!("../../content/day-307.md")),
            (308, include_str!("../../content/day-308.md")),
            (309, include_str!("../../content/day-309.md")),
            (310, include_str!("../../content/day-310.md")),
            (311, include_str!("../../content/day-311.md")),
            (312, include_str!("../../content/day-312.md")),
            (313, include_str!("../../content/day-313.md")),
            (314, include_str!("../../content/day-314.md")),
            (315, include_str!("../../content/day-315.md")),
            (316, include_str!("../../content/day-316.md")),
            (317, include_str!("../../content/day-317.md")),
            (318, include_str!("../../content/day-318.md")),
            (319, include_str!("../../content/day-319.md")),
            (320, include_str!("../../content/day-320.md")),
            (321, include_str!("../../content/day-321.md")),
            (322, include_str!("../../content/day-322.md")),
            (323, include_str!("../../content/day-323.md")),
            (324, include_str!("../../content/day-324.md")),
            (325, include_str!("../../content/day-325.md")),
            (326, include_str!("../../content/day-326.md")),
            (327, include_str!("../../content/day-327.md")),
            (328, include_str!("../../content/day-328.md")),
            (329, include_str!("../../content/day-329.md")),
            (330, include_str!("../../content/day-330.md")),
            (331, include_str!("../../content/day-331.md")),
            (332, include_str!("../../content/day-332.md")),
            (333, include_str!("../../content/day-333.md")),
            (334, include_str!("../../content/day-334.md")),
            (335, include_str!("../../content/day-335.md")),
            (336, include_str!("../../content/day-336.md")),
            (337, include_str!("../../content/day-337.md")),
            (338, include_str!("../../content/day-338.md")),
            (339, include_str!("../../content/day-339.md")),
            (340, include_str!("../../content/day-340.md")),
            (341, include_str!("../../content/day-341.md")),
            (342, include_str!("../../content/day-342.md")),
            (343, include_str!("../../content/day-343.md")),
            (344, include_str!("../../content/day-344.md")),
            (345, include_str!("../../content/day-345.md")),
            (346, include_str!("../../content/day-346.md")),
            (347, include_str!("../../content/day-347.md")),
            (348, include_str!("../../content/day-348.md")),
            (349, include_str!("../../content/day-349.md")),
            (350, include_str!("../../content/day-350.md")),
            (351, include_str!("../../content/day-351.md")),
            (352, include_str!("../../content/day-352.md")),
            (353, include_str!("../../content/day-353.md")),
            (354, include_str!("../../content/day-354.md")),
            (355, include_str!("../../content/day-355.md")),
            (356, include_str!("../../content/day-356.md")),
            (357, include_str!("../../content/day-357.md")),
            (358, include_str!("../../content/day-358.md")),
            (359, include_str!("../../content/day-359.md")),
            (360, include_str!("../../content/day-360.md")),
            (361, include_str!("../../content/day-361.md")),
            (362, include_str!("../../content/day-362.md")),
            (363, include_str!("../../content/day-363.md")),
            (364, include_str!("../../content/day-364.md")),
            (365, include_str!("../../content/day-365.md")),
        ];

        for (day, content) in recipe_files {
            if let Ok(recipe) = Self::parse_recipe_markdown(content, day) {
                self.recipes.insert(recipe.id.clone(), recipe);
            }
        }
    }

    /// Parses recipe data from markdown content
    fn parse_recipe_markdown(content: &str, day: u32) -> RecipeResult<Recipe> {
        let id = format!("day-{}", day);

        // Parse title from first heading or use day as title
        let title = Self::extract_title(content).unwrap_or_else(|| format!("Day {} Recipe", day));

        let mut recipe = Recipe::new(id, title);

        // Parse optional fields from markdown
        recipe.description = Self::extract_description(content);
        recipe.ingredients = Self::extract_ingredients(content);
        recipe.instructions = Self::extract_instructions(content);
        recipe.prep_time_minutes = Self::extract_prep_time(content);
        recipe.cook_time_minutes = Self::extract_cook_time(content);
        recipe.servings = Self::extract_servings(content);
        recipe.tags = Self::extract_tags(content);

        Ok(recipe)
    }

    /// Extracts the title from markdown (first # heading)
    fn extract_title(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
        }
        None
    }

    /// Extracts description (text before first section)
    fn extract_description(content: &str) -> Option<String> {
        let mut desc_lines = Vec::new();
        let mut in_description = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip title
            if trimmed.starts_with("# ") {
                in_description = true;
                continue;
            }

            // Stop at first section heading
            if trimmed.starts_with("## ") {
                break;
            }

            if in_description && !trimmed.is_empty() {
                desc_lines.push(trimmed.to_string());
            }
        }

        if desc_lines.is_empty() {
            None
        } else {
            Some(desc_lines.join(" "))
        }
    }

    /// Extracts ingredients from ## Ingredients section
    fn extract_ingredients(content: &str) -> Vec<String> {
        Self::extract_list_section(content, "## Ingredients")
    }

    /// Extracts instructions from ## Instructions section
    fn extract_instructions(content: &str) -> Vec<String> {
        Self::extract_list_section(content, "## Instructions")
    }

    /// Extracts a list from a markdown section
    fn extract_list_section(content: &str, section_header: &str) -> Vec<String> {
        let mut items = Vec::new();
        let mut in_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == section_header {
                in_section = true;
                continue;
            }

            // Stop at next section
            if in_section && trimmed.starts_with("## ") {
                break;
            }

            if in_section {
                // Handle both - and numbered lists
                if let Some(item) = trimmed.strip_prefix("- ") {
                    items.push(item.trim().to_string());
                } else if let Some(pos) = trimmed.find(". ") {
                    // Check if it's a numbered list (e.g., "1. ")
                    if trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                        items.push(trimmed[pos + 2..].trim().to_string());
                    }
                }
            }
        }

        items
    }

    /// Extracts prep time from metadata
    fn extract_prep_time(content: &str) -> Option<u32> {
        Self::extract_time_field(content, "Prep Time:")
    }

    /// Extracts cook time from metadata
    fn extract_cook_time(content: &str) -> Option<u32> {
        Self::extract_time_field(content, "Cook Time:")
    }

    /// Extracts a time field in minutes
    fn extract_time_field(content: &str, field: &str) -> Option<u32> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix(field) {
                let value = value.trim();
                // Parse "X minutes" or just "X"
                let num_str = value.split_whitespace().next()?;
                return num_str.parse::<u32>().ok();
            }
        }
        None
    }

    /// Extracts servings from metadata
    fn extract_servings(content: &str) -> Option<u32> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Servings:") {
                let value = value.trim();
                let num_str = value.split_whitespace().next()?;
                return num_str.parse::<u32>().ok();
            }
        }
        None
    }

    /// Extracts tags from metadata
    fn extract_tags(content: &str) -> Vec<String> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Tags:") {
                return value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }
}

impl RecipeReader for EmbeddedRecipeStore {
    fn get_by_id(&self, id: &str) -> RecipeResult<Recipe> {
        self.recipes
            .get(id)
            .cloned()
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with id '{}' not found", id)))
    }

    fn get_by_day(&self, day: u32) -> RecipeResult<Recipe> {
        if !(1..=365).contains(&day) {
            return Err(RecipeError::InvalidData(format!(
                "Day must be between 1 and 365, got {}",
                day
            )));
        }

        // Look for recipe with ID "day-{day}"
        let id = format!("day-{}", day);
        self.get_by_id(&id)
    }

    fn get_all(&self) -> RecipeResult<Vec<Recipe>> {
        Ok(self.recipes.values().cloned().collect())
    }

    fn exists(&self, id: &str) -> bool {
        self.recipes.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_store_loads_recipes() {
        let store = EmbeddedRecipeStore::global();
        let recipes = store.get_all().unwrap();
        assert_eq!(recipes.len(), 365, "Should load all 365 recipes");
    }

    #[test]
    fn test_get_by_day() {
        let store = EmbeddedRecipeStore::global();

        // Test day 1
        let recipe = store.get_by_day(1).unwrap();
        assert_eq!(recipe.id, "day-1");
        assert!(!recipe.title.is_empty());

        // Test a middle day
        let recipe = store.get_by_day(100).unwrap();
        assert_eq!(recipe.id, "day-100");

        // Test last day
        let recipe = store.get_by_day(365).unwrap();
        assert_eq!(recipe.id, "day-365");
    }

    #[test]
    fn test_invalid_days() {
        let store = EmbeddedRecipeStore::global();

        // Day 0 should fail
        assert!(store.get_by_day(0).is_err());

        // Day 366 should fail
        assert!(store.get_by_day(366).is_err());
    }

    #[test]
    fn test_recipe_has_content() {
        let store = EmbeddedRecipeStore::global();
        let recipe = store.get_by_day(1).unwrap();

        // Verify recipe has parsed content
        assert!(!recipe.title.is_empty());
        assert!(recipe.description.is_some());
        assert!(!recipe.ingredients.is_empty());
        assert!(!recipe.instructions.is_empty());
        assert!(recipe.prep_time_minutes.is_some());
        assert!(recipe.cook_time_minutes.is_some());
        assert!(recipe.servings.is_some());
    }

    #[test]
    fn test_global_instance_is_singleton() {
        let store1 = EmbeddedRecipeStore::global();
        let store2 = EmbeddedRecipeStore::global();

        // Both should point to the same instance
        assert_eq!(
            store1 as *const _, store2 as *const _,
            "Global instance should be a singleton"
        );
    }
}
