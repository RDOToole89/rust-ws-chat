use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// User structure for managing connected users
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct User {
    pub username: String,
    pub peer_address: String,
}

// In-memory user manager for managing connected users
#[derive(Clone)]
pub struct UserManager {
    users: Arc<Mutex<HashMap<String, User>>>,
}

// UserManager implementation
#[allow(dead_code)]
impl UserManager {
    pub fn new() -> Self {
        UserManager {
            // Initialize an empty hashmap for storing users
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Add a new user to the manager
    pub async fn add_user(&self, username: &str, peer_address: &str) {
        // Comparison: String vs. &str
        // String: Owns its data
        // &str: Borrows data
        // Memory Allocation: &str is a pointer to a string in memory
        // Memory Management: String is allocated on the heap, &str is borrowed from the heap
        // Mutability: String is mutable, &str is immutable
        // Data Passed: Full string content (owned)

        // By default, println! with {} will show the contents of a &str.
        // To see the memory address of a &str, use the {:p} format specifier.

        let user = User {
            username: username.to_string(),         // converts &str to String
            peer_address: peer_address.to_string(), // converts &str to String
        };
        self.users.lock().await.insert(username.to_string(), user);
    }

    // Remove a user from the manager
    pub async fn remove_user(&self, peer_address: &str) {
        self.users.lock().await.remove(peer_address); // Remove the user from the user manager
    }

    // List all connected users
    pub async fn list_users(&self) -> Vec<User> {
        // Lock the users map and collect all usernames
        self.users
            .lock() // Lock the users map
            .await // Wait for the lock to be acquired
            .values() // Get the values of the users map
            .cloned() // Clone the User values instead of just their usernames
            .collect() // Collect the usernames into a vector
    }
}
