struct Card {
    title: String,
    description: String,
    image: String,
    image_alt: String,
    link: Option<String>,
}

struct Section {
    title: String,
    cards: Vec<Card>,
}

fn get_content() {
    return [
        Section {
            title: "My Passion".to_string(),
            cards: vec![
                Card {
                    title: "Responsible Computing".to_string(),
                    description: "We are in the midst of a data revolution and much like the industrial revolution, software engineers have a responsibility for how it affects our society.".to_string(),
                    image: "images/responsible-computing.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Health and Wellness".to_string(),
                    description: "Health is a fundamental human right and health data should help individuals not increase the profits of large corporations.".to_string(),
                    image: "images/health-and-wellness.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Climate Impact".to_string(),
                    description: "Climate change is the existential crisis of our generation.".to_string(),
                    image: "images/environmental-impact.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Education Reform".to_string(),
                    description: "Education is not personalized and technology can increase the availability and accessibility of learning to everyone.".to_string(),
                    image: "images/educational-reform.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Financial Freedom".to_string(),
                    description: "Not everyone is an entrepreneur. Creators and builders should be unencumbered by money to enhance their impact on the world!".to_string(),
                    image: "images/financial-freedom.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                }
        ]},
        Section {
            title: "My Projects".to_string(),
            cards: vec![
                Card {
                    title: "Luggage".to_string(),
                    description: "Data does not belong to corporations. It belongs to us. This project hopes to provide an open-source interface for making your data safe, secure and portable.".to_string(),
                    image: "images/luggage.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Gumby".to_string(),
                    description: "Data-driven health and fitness for everyone.".to_string(),
                    image: "images/gumby.svg".to_string(),
                    image_alt: "".to_string(),
                    link: Some("https://www.6umby.com".to_string())
                },
                Card {
                    title: "Passion Fruit".to_string(),
                    description: "When encumbered by too many dreams and goals, a platform of tools to accelerate many ideas at once is needed.".to_string(),
                    image: "images/passion-fruit.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Blue Eel".to_string(),
                    description: "Literacy is the single most important outcome in a child's life. Let's not forget about those who struggle.".to_string(),
                    image: "images/blue-eel.webp".to_string(),
                    image_alt: "".to_string(),
                    link: Some("https://blue.eel.education".to_string())
                },
                Card {
                    title: "Fire".to_string(),
                    description: "Financial planning tools for everyone so they can better understand the consequences of their decisions.".to_string(),
                    image: "images/fire.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                }
        ]},
        Section {
            title: "My Hobbies".to_string(),
            cards: vec![
                Card {
                    title: "Space Exploration".to_string(),
                    description: "I have always been interested in space and I believe that becoming multi-planetary is important for humanity. I just don't know yet how I can help or if it is our most pressing concern.".to_string(),
                    image: "images/space-exploration.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Fitness".to_string(),
                    description: "A sound body is key to a sound mind. Through running, swimming, and climbing I find my peace in a healthy life.".to_string(),
                    image: "images/fitness.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Natural Beauty".to_string(),
                    description: "I long for the beautiful vistas and natural places in this world. I want to preserve and sustain them for all generations!".to_string(),
                    image: "images/natural-beauty.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "Physics and Math".to_string(),
                    description: "Physics and math have always intrigued me. I would love to just spend all day working on fundamental unsolved problems in this space, but I am neither smart enough or focused enough. Instead, I am working on empowered those who are with financial freedom!".to_string(),
                    image: "images/physics-and-math.webp".to_string(),
                    image_alt: "".to_string(),
                    link: None
                },
                Card {
                    title: "FIRST Robotics".to_string(),
                    description: "Mentoring the next generation into being well rounded, thoughtful engineers brings me endless joy. ".to_string(),
                    image: "images/robotics-and-automation.webp".to_string(),
                    image_alt: "".to_string(),
                    link: Some("http://wildraccoons8891.org/".to_string())
                }
        ]}
    ];
}
