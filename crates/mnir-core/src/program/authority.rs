use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::{
    AllocationCounterState, AllocationNamespaceId, IdentifierCategory, ProgramId, RevisionId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AuthorityClaimError {
    AlreadyActive,
    Stale,
    IdentityCollision(IdentifierCategory),
}

#[derive(Clone, Copy, Debug)]
struct AuthorityRecord {
    namespace: AllocationNamespaceId,
    revision: RevisionId,
    counter: AllocationCounterState,
    active_claim: Option<u64>,
    next_claim: u64,
}

#[derive(Debug)]
pub(super) struct AuthorityLease {
    program_id: ProgramId,
    claim: u64,
}

impl AuthorityLease {
    pub(super) fn claim_fresh(
        program_id: ProgramId,
        namespace: AllocationNamespaceId,
        revision: RevisionId,
        counter: AllocationCounterState,
    ) -> Result<Self, AuthorityClaimError> {
        let mut registry = lock_registry();
        if registry.contains_key(&program_id) {
            return Err(AuthorityClaimError::IdentityCollision(
                IdentifierCategory::Program,
            ));
        }
        if registry
            .values()
            .any(|record| record.namespace == namespace)
        {
            return Err(AuthorityClaimError::IdentityCollision(
                IdentifierCategory::AllocationNamespace,
            ));
        }
        registry.insert(
            program_id,
            AuthorityRecord {
                namespace,
                revision,
                counter,
                active_claim: Some(1),
                next_claim: 2,
            },
        );
        Ok(Self {
            program_id,
            claim: 1,
        })
    }

    pub(super) fn claim_restored(
        program_id: ProgramId,
        namespace: AllocationNamespaceId,
        revision: RevisionId,
        counter: AllocationCounterState,
    ) -> Result<Self, AuthorityClaimError> {
        let mut registry = lock_registry();
        if !registry.contains_key(&program_id)
            && registry
                .values()
                .any(|record| record.namespace == namespace)
        {
            return Err(AuthorityClaimError::IdentityCollision(
                IdentifierCategory::AllocationNamespace,
            ));
        }

        let record = registry.entry(program_id).or_insert(AuthorityRecord {
            namespace,
            revision,
            counter,
            active_claim: None,
            next_claim: 1,
        });
        if record.namespace != namespace {
            return Err(AuthorityClaimError::IdentityCollision(
                IdentifierCategory::AllocationNamespace,
            ));
        }
        if record.active_claim.is_some() {
            return Err(AuthorityClaimError::AlreadyActive);
        }
        if revision.persistent_value() < record.revision.persistent_value()
            || counter_is_older(counter, record.counter)
        {
            return Err(AuthorityClaimError::Stale);
        }

        record.revision = revision;
        if counter_is_older(record.counter, counter) {
            record.counter = counter;
        }
        let claim = record.next_claim;
        record.next_claim = record.next_claim.wrapping_add(1).max(1);
        record.active_claim = Some(claim);
        Ok(Self { program_id, claim })
    }

    pub(super) fn observe(&self, revision: RevisionId, counter: AllocationCounterState) {
        let mut registry = lock_registry();
        let record = registry
            .get_mut(&self.program_id)
            .expect("live authority lease has a registry record");
        debug_assert_eq!(record.active_claim, Some(self.claim));
        if revision.persistent_value() > record.revision.persistent_value() {
            record.revision = revision;
        }
        if counter_is_older(record.counter, counter) {
            record.counter = counter;
        }
    }
}

impl Drop for AuthorityLease {
    fn drop(&mut self) {
        let mut registry = lock_registry();
        if let Some(record) = registry.get_mut(&self.program_id)
            && record.active_claim == Some(self.claim)
        {
            record.active_claim = None;
        }
    }
}

pub(super) fn known_program_ids() -> HashSet<[u8; 16]> {
    lock_registry()
        .keys()
        .map(|program_id| program_id.0)
        .collect()
}

pub(super) fn known_namespaces() -> HashSet<[u8; 16]> {
    lock_registry()
        .values()
        .map(|record| record.namespace.0)
        .collect()
}

const fn counter_is_older(
    candidate: AllocationCounterState,
    latest: AllocationCounterState,
) -> bool {
    match (candidate, latest) {
        (AllocationCounterState::Available(left), AllocationCounterState::Available(right)) => {
            left < right
        }
        (AllocationCounterState::Available(_), AllocationCounterState::Exhausted) => true,
        (AllocationCounterState::Exhausted, _) => false,
    }
}

fn registry() -> &'static Mutex<HashMap<ProgramId, AuthorityRecord>> {
    static REGISTRY: OnceLock<Mutex<HashMap<ProgramId, AuthorityRecord>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_registry() -> std::sync::MutexGuard<'static, HashMap<ProgramId, AuthorityRecord>> {
    registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
