use noa_core::config::manifest::{
    KernelManifest, SCOPE_HOST_ENVIRONMENT_TAKEOVER, SCOPE_HOST_RESOURCE_ARBITRATE,
};
use noa_core::host_control::{self, HostControlError, ResourceArbitrationRequest};
use noa_core::token::{self, service as token_service, TokenIssuanceRequest};

fn setup_services() {
    token_service().reset();
    host_control::service().reset();
    let manifest = KernelManifest::default();
    token::configure_from_manifest(&manifest);
}

#[test]
fn environment_takeover_requires_scope() {
    setup_services();
    let request = TokenIssuanceRequest::new("controller", [SCOPE_HOST_RESOURCE_ARBITRATE]);
    let token = token_service()
        .issue_token(request)
        .expect("token issuance succeeds");
    let result = host_control::service().request_environment_takeover(&token.token, "lab");
    assert!(matches!(result, Err(HostControlError::Token(_))));
}

#[test]
fn arbitration_enforces_isolation() {
    setup_services();
    let takeover_request = TokenIssuanceRequest::new(
        "controller",
        [
            SCOPE_HOST_ENVIRONMENT_TAKEOVER,
            SCOPE_HOST_RESOURCE_ARBITRATE,
        ],
    );
    let controller_token = token_service()
        .issue_token(takeover_request)
        .expect("token issuance succeeds");

    host_control::service()
        .request_environment_takeover(&controller_token.token, "lab")
        .expect("controller should obtain lease");

    let rogue_request = TokenIssuanceRequest::new("rogue", [SCOPE_HOST_RESOURCE_ARBITRATE]);
    let rogue_token = token_service()
        .issue_token(rogue_request)
        .expect("rogue token issuance succeeds");
    let decision = host_control::service().arbitrate_resources(
        &rogue_token.token,
        ResourceArbitrationRequest {
            environment: "lab".to_string(),
            desired_cpu_share: 0.5,
            desired_memory_bytes: 512 * 1024 * 1024,
        },
    );
    assert!(matches!(
        decision,
        Err(HostControlError::EnvironmentIsolationViolation(env)) if env == "lab"
    ));

    let granted = host_control::service()
        .arbitrate_resources(
            &controller_token.token,
            ResourceArbitrationRequest {
                environment: "lab".to_string(),
                desired_cpu_share: 0.9,
                desired_memory_bytes: 2 * 1024 * 1024 * 1024,
            },
        )
        .expect("controller arbitration should succeed");
    assert!(granted.granted_cpu_share <= 0.75);
    assert!(granted.isolation_enforced);
}
