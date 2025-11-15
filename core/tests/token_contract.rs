use noa_core::config::manifest::{KernelManifest, SCOPE_HOST_ENVIRONMENT_TAKEOVER};
use noa_core::token::{self, service as token_service, TokenError, TokenIssuanceRequest};

fn setup_manifest() {
    token_service().reset();
    let manifest = KernelManifest::default();
    token::configure_from_manifest(&manifest);
}

#[test]
fn token_issuance_and_validation_flow() {
    setup_manifest();
    let request = TokenIssuanceRequest::new("controller", [SCOPE_HOST_ENVIRONMENT_TAKEOVER]);
    let token = token_service()
        .issue_token(request)
        .expect("token issuance should succeed");

    let validated = token_service()
        .validate(&token.token, SCOPE_HOST_ENVIRONMENT_TAKEOVER)
        .expect("token should validate");
    assert_eq!(validated.issued_to, "controller");
    assert!(validated.grants_scope(SCOPE_HOST_ENVIRONMENT_TAKEOVER));
}

#[test]
fn token_revocation_is_enforced() {
    setup_manifest();
    let request = TokenIssuanceRequest::new("tester", [SCOPE_HOST_ENVIRONMENT_TAKEOVER]);
    let token = token_service()
        .issue_token(request)
        .expect("token issuance should succeed");
    token_service()
        .revoke(&token.token)
        .expect("revocation should succeed");

    let result = token_service().validate(&token.token, SCOPE_HOST_ENVIRONMENT_TAKEOVER);
    match result {
        Err(TokenError::Revoked(_)) => {}
        other => panic!("expected revoked token error, got {:?}", other),
    }
}

#[test]
fn rejecting_unknown_scopes() {
    setup_manifest();
    let request = TokenIssuanceRequest::new("tester", ["unknown.scope"]);
    let result = token_service().issue_token(request);
    assert!(matches!(result, Err(TokenError::UnknownScope(scope)) if scope == "unknown.scope"));
}
