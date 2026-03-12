use serde::{Deserialize, Serialize};

/// Represents a mock user for development authentication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockUser {
    pub id: &'static str,
    pub display_name: &'static str,
    pub avatar_url: &'static str,
}

pub const MOCK_USERS: [MockUser; 3] = [
    MockUser {
        id: "mock-alice-001",
        display_name: "Alice",
        avatar_url: "https://api.dicebear.com/9.x/avataaars/svg?seed=Alice",
    },
    MockUser {
        id: "mock-bob-002",
        display_name: "Bob",
        avatar_url: "https://api.dicebear.com/9.x/avataaars/svg?seed=Bob",
    },
    MockUser {
        id: "mock-charlie-003",
        display_name: "Charlie",
        avatar_url: "https://api.dicebear.com/9.x/avataaars/svg?seed=Charlie",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_users_have_unique_ids() {
        let ids: Vec<&str> = MOCK_USERS.iter().map(|u| u.id).collect();
        assert_eq!(ids.len(), 3);
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
    }

    #[test]
    fn mock_users_have_unique_names() {
        let names: Vec<&str> = MOCK_USERS.iter().map(|u| u.display_name).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn mock_users_have_unique_avatar_urls() {
        let urls: Vec<&str> = MOCK_USERS.iter().map(|u| u.avatar_url).collect();
        assert_ne!(urls[0], urls[1]);
        assert_ne!(urls[1], urls[2]);
        assert_ne!(urls[0], urls[2]);
    }
}
