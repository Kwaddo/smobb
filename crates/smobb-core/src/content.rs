use crate::scene::{Content, Location, Scene};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ContentPart {
    #[serde(default)]
    locations: std::collections::HashMap<String, Location>,
    #[serde(default)]
    scenes: std::collections::HashMap<String, Scene>,
}

pub fn from_str(parts: &[&str]) -> Content {
    let mut locations = std::collections::HashMap::new();
    let mut scenes = std::collections::HashMap::new();
    for p in parts {
        let part: ContentPart =
            ron::from_str(p).unwrap_or_else(|e| panic!("content parse error: {e}"));
        locations.extend(part.locations);
        scenes.extend(part.scenes);
    }
    Content {
        start: "hub".to_string(),
        locations,
        scenes,
    }
}

/// Embeds every content/*.ron at build time. New files MUST be added here.
pub fn load() -> Content {
    from_str(&[
        include_str!("../../../content/hub.ron"),
        include_str!("../../../content/ali.ron"),
        include_str!("../../../content/bader.ron"),
        include_str!("../../../content/bevil.ron"),
        include_str!("../../../content/hasan.ron"),
        include_str!("../../../content/josh.ron"),
        include_str!("../../../content/qassim.ron"),
        include_str!("../../../content/smobb.ron"),
        include_str!("../../../content/extra.ron"),
        include_str!("../../../content/ending.ron"),
        include_str!("../../../content/guide.ron"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merges_parts() {
        let a = r#"ContentPart(
            locations: { "hub": (commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ]) },
            scenes: {},
        )"#;
        let b = r#"ContentPart(
            locations: { "ali": (commands: [ Branch(keys:["end"], target: Steps([ Return ])) ]) },
            scenes: { "x": (steps: [ Return ]) },
        )"#;
        let c = from_str(&[a, b]);
        assert!(c.locations.contains_key("hub"));
        assert!(c.locations.contains_key("ali"));
        assert!(c.scenes.contains_key("x"));
        assert_eq!(c.start, "hub");
    }
}
