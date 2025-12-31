use crate::models::{MemoryCursor, MemoryRecord, MemoryRetrieval, MemoryRole};
use crate::store::{build_retrieval, LongTermMemory, MemoryError, MemoryStore, SessionMemory};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DEFAULT_LIMIT: usize = 32;

pub struct MemoryCoordinator {
    root: PathBuf,
    long_term: Arc<LongTermMemory>,
    sessions: RwLock<HashMap<String, Arc<SessionMemory>>>,
    default_limit: usize,
}

impl MemoryCoordinator {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;
        let long_term = Arc::new(LongTermMemory::open(&root)?);
        Ok(Self {
            root,
            long_term,
            sessions: RwLock::new(HashMap::new()),
            default_limit: DEFAULT_LIMIT,
        })
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        if limit > 0 {
            self.default_limit = limit;
        }
        self
    }

    pub fn long_term(&self) -> Arc<LongTermMemory> {
        Arc::clone(&self.long_term)
    }

    fn session_store(&self, session_id: &str) -> Result<Arc<SessionMemory>, MemoryError> {
        if let Some(store) = self.sessions.read().get(session_id) {
            return Ok(Arc::clone(store));
        }

        let mut sessions = self.sessions.write();
        if let Some(store) = sessions.get(session_id) {
            return Ok(Arc::clone(store));
        }

        let store = Arc::new(SessionMemory::open(&self.root, session_id.to_string())?);
        sessions.insert(session_id.to_string(), Arc::clone(&store));
        Ok(store)
    }

    pub fn record_interaction(
        &self,
        session_id: Option<&str>,
        agent: &str,
        role: MemoryRole,
        content: &str,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError> {
        let record =
            self.long_term
                .append(agent, role.clone(), content, metadata.clone(), tags.clone())?;

        if let Some(session_id) = session_id {
            let session = self.session_store(session_id)?;
            let _ = session.append(agent, role, content, metadata, tags)?;
        }

        Ok(record)
    }

    pub fn record_session_only(
        &self,
        session_id: &str,
        agent: &str,
        role: MemoryRole,
        content: &str,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError> {
        let session = self.session_store(session_id)?;
        session.append(agent, role, content, metadata, tags)
    }

    pub fn incremental_context(
        &self,
        session_id: Option<&str>,
        cursor: MemoryCursor,
        limit: Option<usize>,
    ) -> Result<MemoryRetrieval, MemoryError> {
        let limit = limit.unwrap_or(self.default_limit);
        if limit == 0 {
            return Ok(MemoryRetrieval::empty(cursor));
        }

        match session_id {
            Some(id) => {
                let session = self.session_store(id)?;
                build_retrieval(&*self.long_term, &*session, cursor, limit)
            }
            None => build_retrieval(&*self.long_term, &EmptyMemory, cursor, limit),
        }
    }
}

struct EmptyMemory;

impl MemoryStore for EmptyMemory {
    fn append(
        &self,
        _agent: &str,
        _role: MemoryRole,
        _content: &str,
        _metadata: HashMap<String, String>,
        _tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError> {
        Err(MemoryError::StoreMissing("session".into()))
    }

    fn incremental(
        &self,
        cursor: u64,
        _limit: usize,
    ) -> Result<(Vec<MemoryRecord>, u64), MemoryError> {
        Ok((Vec::new(), cursor))
    }
}
