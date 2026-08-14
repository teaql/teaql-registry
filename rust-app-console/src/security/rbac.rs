use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RbacChecker {
    pub privileges: HashSet<String>,
}

impl RbacChecker {
    pub fn new(privileges: HashSet<String>) -> Self {
        Self { privileges }
    }

    pub fn has_privilege(&self, required: &str) -> bool {
        if self.privileges.contains("nx-all") || self.privileges.contains("*") {
            return true;
        }

        if self.privileges.contains(required) {
            return true;
        }

        for priv_pattern in &self.privileges {
            if match_wildcard(priv_pattern, required) {
                return true;
            }
        }

        false
    }

    pub fn check_repository_permission(
        &self,
        format: &str,
        repository_name: &str,
        action: &str,
    ) -> bool {
        let required = format!("nx-repository-view-{}-{}-{}", format, repository_name, action);
        self.has_privilege(&required)
    }
}

pub fn match_wildcard(pattern: &str, target: &str) -> bool {
    if pattern == target || pattern == "*" {
        return true;
    }

    let regex_pattern = format!(
        "^{}$",
        regex::escape(pattern).replace(r"\*", ".*")
    );

    if let Ok(re) = regex::Regex::new(&regex_pattern) {
        re.is_match(target)
    } else {
        false
    }
}
