use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, BufRead};
use nucleo_matcher::{Matcher, Config, Utf32Str};

#[derive(Debug, Clone)]
pub struct Project {
    folder: String,
    path: String,
}

#[derive(Debug, Clone)]
struct SearchResult {
    project: Project,
    score: f64,
}

#[derive(Serialize, Deserialize)]
struct AlfredIcon {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    icon_type: Option<String>,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct AlfredMod {
    arg: String,
    subtitle: String,
    icon: AlfredIcon,
}

#[derive(Serialize, Deserialize)]
struct AlfredItem {
    title: String,
    subtitle: String,
    arg: String,
    icon: AlfredIcon,
    mods: std::collections::HashMap<String, AlfredMod>,
}

#[derive(Serialize, Deserialize)]
struct AlfredResponse {
    items: Vec<AlfredItem>,
}

struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    fn new() -> Self {
        let config = Config::DEFAULT;
        Self {
            matcher: Matcher::new(config),
        }
    }

    fn search(&mut self, query: &str, projects: &[Project]) -> Vec<SearchResult> {
        let query = query.trim();
        
        if query.is_empty() {
            let mut results: Vec<SearchResult> = projects
                .iter()
                .map(|p| SearchResult {
                    project: p.clone(),
                    score: 1.0,
                })
                .collect();
            
            results.sort_by(|a, b| a.project.folder.to_lowercase().cmp(&b.project.folder.to_lowercase()));
            return results;
        }

        // Create buffers for UTF-32 conversion
        let mut query_chars = Vec::new();
        let query_utf32 = Utf32Str::new(query, &mut query_chars);

        let mut results: Vec<SearchResult> = projects
            .iter()
            .filter_map(|project| {
                // Create buffer for each project folder
                let mut folder_chars = Vec::new();
                let folder_utf32 = Utf32Str::new(&project.folder, &mut folder_chars);
                
                // Use nucleo-matcher to score the match
                if let Some(score) = self.matcher.fuzzy_match(folder_utf32, query_utf32) {
                    // Convert nucleo score (u16) to f64 and normalize
                    let normalized_score = score as f64 / 1000.0; // nucleo scores are typically 0-1000+
                    Some(SearchResult {
                        project: project.clone(),
                        score: normalized_score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

pub fn expand_home_path(path: &str, home_path: &str) -> String {
    if path.starts_with("~/") {
        format!("{}/{}", home_path, &path[2..])
    } else {
        path.to_string()
    }
}

pub fn find_projects(search_paths: &str, ignore_patterns_string: &str, home_path: &str) -> Vec<Project> {
    let ignore_patterns: Vec<&str> = if ignore_patterns_string.is_empty() {
        vec![".", "..", ".DS_Store"]
    } else {
        ignore_patterns_string.split(',').map(|s| s.trim()).collect()
    };

    let paths: Vec<String> = search_paths
        .split(',')
        .map(|path| expand_home_path(path.trim(), home_path))
        .collect();

    let mut projects = Vec::new();

    for base_path in paths {
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(folder_name) = entry.file_name().to_str() {
                            if !ignore_patterns.contains(&folder_name) {
                                projects.push(Project {
                                    folder: folder_name.to_string(),
                                    path: entry.path().to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    projects
}

fn create_alfred_output(results: &[SearchResult]) -> String {
    let items: Vec<AlfredItem> = results
        .iter()
        .map(|result| {
            let mut mods = std::collections::HashMap::new();

            mods.insert(
                "cmd".to_string(),
                AlfredMod {
                    arg: result.project.path.clone(),
                    subtitle: "Open in PhpStorm".to_string(),
                    icon: AlfredIcon {
                        icon_type: Some("fileicon".to_string()),
                        path: "/Applications/PhpStorm.app".to_string(),
                    },
                },
            );

            mods.insert(
                "shift".to_string(),
                AlfredMod {
                    arg: result.project.path.clone(),
                    subtitle: "Open in iTerm".to_string(),
                    icon: AlfredIcon {
                        icon_type: Some("fileicon".to_string()),
                        path: "/Applications/iTerm.app".to_string(),
                    },
                },
            );

            mods.insert(
                "ctrl".to_string(),
                AlfredMod {
                    arg: result.project.path.clone(),
                    subtitle: "Reveal in Finder".to_string(),
                    icon: AlfredIcon {
                        icon_type: Some("fileicon".to_string()),
                        path: "/System/Library/CoreServices/Finder.app".to_string(),
                    },
                },
            );

            AlfredItem {
                title: result.project.folder.clone(),
                subtitle: result.project.path.clone(),
                arg: result.project.path.clone(),
                icon: AlfredIcon {
                    icon_type: Some("fileicon".to_string()),
                    path: "/Applications/Visual Studio Code.app".to_string(),
                },
                mods,
            }
        })
        .collect();

    let response = AlfredResponse { items };

    serde_json::to_string(&response).unwrap_or_else(|_| r#"{"items":[]}"#.to_string())
}

pub fn process_search_request(query: &str, search_paths: &str, ignore_patterns: &str, home_path: &str) -> String {
    if search_paths.is_empty() {
        let warning_response = AlfredResponse {
            items: vec![AlfredItem {
                title: "Search paths not configured".to_string(),
                subtitle: "Add directories for the Quick Open Project workflow".to_string(),
                arg: "alfred://workflow/com.mattstein.quick-open-project".to_string(),
                icon: AlfredIcon {
                    icon_type: None,
                    path: "./warning.png".to_string(),
                },
                mods: std::collections::HashMap::new(),
            }],
        };
        return serde_json::to_string(&warning_response).unwrap_or_else(|_| r#"{"items":[]}"#.to_string());
    }

    let projects = find_projects(search_paths, ignore_patterns, home_path);
    let mut matcher = FuzzyMatcher::new();
    let results = matcher.search(query, &projects);
    create_alfred_output(&results)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut query = String::new();

    // Try command line arguments first
    if args.len() > 1 {
        query = args[1].clone();
    } else {
        // If no command line args, try reading from STDIN
        if let Some(Ok(line)) = io::stdin().lock().lines().next() {
            query = line.trim().to_string();
        }
    }

    let query = query.trim();
    let search_paths_env = env::var("SEARCH_PATHS").unwrap_or_default();
    let ignore_patterns_env = env::var("IGNORE_PATTERNS").unwrap_or_else(|_| ".,..,.DS_Store".to_string());
    let home_path_env = env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

    let output = process_search_request(query, &search_paths_env, &ignore_patterns_env, &home_path_env);
    println!("{}", output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_projects(base_dir: &std::path::Path) -> Vec<&'static str> {
        let project_names = vec!["project-one", "another-project", "test-app", "my-website"];
        for name in &project_names {
            fs::create_dir_all(base_dir.join(name)).unwrap();
        }
        project_names
    }

    #[test]
    fn test_empty_search_paths_warning() {
        let result = process_search_request("test", "", ".,..,.DS_Store", "/home/user");
        
        // Parse JSON to verify structure
        let parsed: AlfredResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.items.len(), 1);
        
        let item = &parsed.items[0];
        assert!(item.title.contains("Search paths not configured"));
        assert!(item.subtitle.contains("directories for the Quick Open Project workflow"));
        assert!(item.arg.contains("alfred://workflow"));
        assert_eq!(item.icon.icon_type, None);
        assert_eq!(item.icon.path, "./warning.png");
    }

    #[test]
    fn test_home_directory_expansion() {
        // Test basic expansion
        let result = expand_home_path("~/Documents/Projects", "/Users/testuser");
        assert_eq!(result, "/Users/testuser/Documents/Projects");

        // Test path that doesn't start with ~/
        let result = expand_home_path("/absolute/path", "/Users/testuser");
        assert_eq!(result, "/absolute/path");

        // Test just ~/
        let result = expand_home_path("~/", "/Users/testuser");
        assert_eq!(result, "/Users/testuser/");
    }

    #[test]
    fn test_find_projects_with_temp_directories() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test project directories
        let project_names = create_test_projects(temp_path);
        
        // Test finding projects
        let projects = find_projects(
            &temp_path.to_string_lossy(),
            ".,..,.DS_Store",
            "/tmp"
        );

        // Should find all created projects
        assert_eq!(projects.len(), project_names.len());
        
        // Verify project names are found
        let found_names: Vec<&str> = projects.iter().map(|p| p.folder.as_str()).collect();
        for name in project_names {
            assert!(found_names.contains(&name));
        }
    }

    #[test]
    fn test_find_projects_with_home_expansion() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let fake_home = temp_path.to_string_lossy().to_string();
        
        // Create a subdirectory to simulate ~/Projects
        let projects_dir = temp_path.join("Projects");
        fs::create_dir_all(&projects_dir).unwrap();
        create_test_projects(&projects_dir);
        
        // Test with ~ expansion
        let projects = find_projects("~/Projects", ".,..,.DS_Store", &fake_home);
        assert_eq!(projects.len(), 4);
    }

    #[test]
    fn test_find_projects_ignores_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create normal projects
        create_test_projects(temp_path);
        
        // Create directories that should be ignored
        fs::create_dir_all(temp_path.join(".git")).unwrap();
        fs::create_dir_all(temp_path.join("node_modules")).unwrap();
        
        // Test with custom ignore patterns
        let projects = find_projects(
            &temp_path.to_string_lossy(),
            ".git,node_modules,.,..,.DS_Store",
            "/tmp"
        );

        // Should only find the 4 real projects, not the ignored ones
        assert_eq!(projects.len(), 4);
        
        // Verify ignored directories are not included
        let found_names: Vec<&str> = projects.iter().map(|p| p.folder.as_str()).collect();
        assert!(!found_names.contains(&".git"));
        assert!(!found_names.contains(&"node_modules"));
    }

    #[test]
    fn test_search_with_real_projects() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let fake_home = temp_path.to_string_lossy().to_string();
        
        create_test_projects(temp_path);
        
        // Test search with query
        let result = process_search_request(
            "test",
            &temp_path.to_string_lossy(),
            ".,..,.DS_Store",
            &fake_home
        );
        
        // Parse and verify results
        let parsed: AlfredResponse = serde_json::from_str(&result).unwrap();
        assert!(!parsed.items.is_empty());
        
        // Should find test-app project
        let found_test_app = parsed.items.iter().any(|item| item.title == "test-app");
        assert!(found_test_app);
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let fake_home = temp_path.to_string_lossy().to_string();
        
        create_test_projects(temp_path);
        
        // Test search with empty query
        let result = process_search_request(
            "",
            &temp_path.to_string_lossy(),
            ".,..,.DS_Store",
            &fake_home
        );
        
        // Parse and verify results
        let parsed: AlfredResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.items.len(), 4); // Should return all projects
    }

    #[test]
    fn test_search_fuzzy_matching() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let fake_home = temp_path.to_string_lossy().to_string();
        
        create_test_projects(temp_path);
        
        // Test fuzzy search with "proj" should match "project-one" and "another-project"
        let result = process_search_request(
            "proj",
            &temp_path.to_string_lossy(),
            ".,..,.DS_Store",
            &fake_home
        );
        
        let parsed: AlfredResponse = serde_json::from_str(&result).unwrap();
        assert!(!parsed.items.is_empty());
        
        // Should find projects containing "proj"
        let project_titles: Vec<&str> = parsed.items.iter().map(|item| item.title.as_str()).collect();
        assert!(project_titles.contains(&"project-one") || project_titles.contains(&"another-project"));
    }

    #[test]
    fn test_alfred_output_structure() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let fake_home = temp_path.to_string_lossy().to_string();
        
        create_test_projects(temp_path);
        
        let result = process_search_request(
            "project",
            &temp_path.to_string_lossy(),
            ".,..,.DS_Store",
            &fake_home
        );
        
        let parsed: AlfredResponse = serde_json::from_str(&result).unwrap();
        assert!(!parsed.items.is_empty());
        
        // Verify Alfred item structure
        let item = &parsed.items[0];
        assert!(!item.title.is_empty());
        assert!(!item.subtitle.is_empty());
        assert!(!item.arg.is_empty());
        assert_eq!(item.icon.icon_type, Some("fileicon".to_string()));
        assert!(item.icon.path.contains("Visual Studio Code"));
        
        // Verify modifier keys
        assert!(item.mods.contains_key("cmd"));
        assert!(item.mods.contains_key("shift"));
        assert!(item.mods.contains_key("ctrl"));
        
        let cmd_mod = &item.mods["cmd"];
        assert!(cmd_mod.subtitle.contains("PhpStorm"));
    }
}