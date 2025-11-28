use std::sync::Arc;

use crate::events::ShellEvent;
use crate::state::{GlobalStore, Notification, NotificationLevel, UserSession};

/// Cross-cutting services exposed to module applications.
#[derive(Clone)]
pub struct ShellServices {
    store: GlobalStore,
    event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
}

impl ShellServices {
    pub fn new(store: GlobalStore, event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>) -> Self {
        Self { store, event_sink }
    }

    pub fn session(&self) -> UserSession {
        self.store.read().session
    }

    pub fn update_session<F>(&self, updater: F)
    where
        F: FnOnce(&mut UserSession),
    {
        self.store.update(|state| updater(&mut state.session));
    }

    pub fn notify(&self, message: impl Into<String>, level: NotificationLevel) {
        self.store
            .push_notification(Notification::new(level, message.into()));
    }

    pub fn publish(&self, event: ShellEvent) {
        (self.event_sink)(event);
    }

    pub fn notifications(&self) -> Vec<Notification> {
        self.store.read().notifications
    }
}

pub fn use_shell_services(
    store: &GlobalStore,
    event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
) -> ShellServices {
    ShellServices::new(store.clone(), event_sink)
}

pub fn use_session(store: &GlobalStore) -> UserSession {
    store.read().session
}

pub fn use_notifications(store: &GlobalStore) -> Vec<Notification> {
    store.read().notifications
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ShellEvent;
    use crate::state::GlobalState;

    #[test]
    fn hooks_surface_session_and_notifications() {
        let store = GlobalStore::new(GlobalState::default());
        let captured = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let sink = {
            let captured = captured.clone();
            Arc::new(move |event: ShellEvent| captured.lock().unwrap().push(event))
        };

        let services = use_shell_services(&store, sink.clone());
        assert_eq!(services.session().user_id, "anonymous");

        services.update_session(|session| session.user_id = "tester".into());
        services.notify("hello", NotificationLevel::Info);
        services.publish(ShellEvent::ModuleRegistered {
            module_id: "test".into(),
        });

        let session = use_session(&store);
        assert_eq!(session.user_id, "tester");
        assert_eq!(use_notifications(&store).len(), 1);
        assert_eq!(captured.lock().unwrap().len(), 1);
    }
}
