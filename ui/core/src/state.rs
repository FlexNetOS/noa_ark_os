// UI State Management
// Placeholder for UI state

pub struct AppState {
    pub connected: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
        }
    }
}
