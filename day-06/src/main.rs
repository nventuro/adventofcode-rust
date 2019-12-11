use std::collections::HashMap;
use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    let orbit_descriptions = contents.split_whitespace()
        .filter(|line| !line.is_empty())
        .collect();

    let orbits = get_orbits(orbit_descriptions);
    let total_orbits = get_total_orbits(&orbits);

    println!("Total orbits: {}", total_orbits);


    let parent = orbits.keys()
        .find(|body| {
            let subtree = get_subtree_nodes(&orbits, body);

            if has_target(&subtree, "YOU") && has_target(&subtree, "SAN") {
                let children = orbits.get(*body).unwrap();
                if !has_target(&children, "YOU") || !has_target(&children, "SAN") {
                    return true
                } else {
                    return false
                }
            } else {
                false
            }
        }).unwrap();

    println!("{:?}", get_path_to_body(&orbits, "COM", "SAN"));
    println!("{:?}", get_path_to_body(&orbits, "COM", "YOU"));
}

fn has_target(collection: &Vec<&str>, target: &str) -> bool {
    collection.iter().find(|value| **value == target).is_some()
}

fn get_orbits(orbit_descriptions: Vec<&str>) -> HashMap<&str, Vec<&str>> {
    let mut orbits = HashMap::new();

    for description in orbit_descriptions {
        let delimiter_index = description.split(")").collect::<Vec<_>>();
        assert!(delimiter_index.len() == 2);

        let parent = delimiter_index[0];
        let child = delimiter_index[1];

        orbits.entry(parent).or_insert(Vec::<&str>::new())
            .push(child);
    }

    orbits
}

fn get_path_to_body<'a>(orbits: &HashMap<&str, Vec<&'a str>>, root: &'a str, body: &str) -> Option<Vec<&'a str>> {
    if root == body {
        return Some(vec![root]);
    }

    for child in orbits.get(root).unwrap_or(&Vec::new()) {
        if let Some(mut path) = get_path_to_body(orbits, child, body) {
            path.push(root);
            return Some(path);
        }
    }

    return None;
}

fn get_total_orbits(orbits: &HashMap<&str, Vec<&str>>) -> u32 {
    sum_node_depths(orbits, "COM", 0)
}

fn get_subtree_nodes<'a>(orbits: &HashMap<&str, Vec<&'a str>>, root: &'a str) -> Vec<&'a str> {
    let new_vec = Vec::<&str>::new();

    let mut children = orbits
        .get(root).unwrap_or(&new_vec)
        .iter()
        .flat_map(|child| get_subtree_nodes(orbits, child))
        .collect::<Vec<_>>();

    children.push(root);
    children
}

fn sum_node_depths(orbits: &HashMap<&str, Vec<&str>>, current: &str, depth: u32) -> u32 {
    let new_vec = Vec::<&str>::new();

    let subtree_depth = orbits
        .get(current).unwrap_or(&new_vec)
        .iter()
        .map(|child| sum_node_depths(orbits, child, depth + 1))
        .sum::<u32>();

    depth + subtree_depth
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let orbits = get_orbits(vec!["COM)B","B)C","C)D","D)E","E)F","B)G","G)H","D)I","E)J","J)K","K)L"]);
        assert_eq!(get_total_orbits(&orbits), 42);
    }
}

