use noa_core::config::manifest::{SCOPE_HOST_ENVIRONMENT_TAKEOVER, SCOPE_HOST_RESOURCE_ARBITRATE};
use noa_core::host_control::{service as host_control_service, ResourceArbitrationRequest};
use noa_core::kernel;
use noa_core::security::SecurityService;
use noa_core::token::TokenIssuanceRequest;

fn main() {
    if let Err(err) = kernel::init() {
        eprintln!("Host control harness failed to initialise kernel: {err}");
        return;
    }

    let security = SecurityService;
    let token = match security.issue_scope_token(
        TokenIssuanceRequest::new(
            "host-operator",
            [
                SCOPE_HOST_ENVIRONMENT_TAKEOVER,
                SCOPE_HOST_RESOURCE_ARBITRATE,
            ],
        )
        .with_metadata("session", "harness-demo"),
    ) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Failed to issue host control token: {err}");
            return;
        }
    };

    let host_control = host_control_service();
    match host_control.request_environment_takeover(&token.token, "lab-environment") {
        Ok(lease) => {
            println!(
                "Environment takeover granted for {} by {}",
                lease.environment, lease.issued_to
            );
        }
        Err(err) => {
            eprintln!("Failed to request environment takeover: {err}");
            return;
        }
    }

    match host_control.arbitrate_resources(
        &token.token,
        ResourceArbitrationRequest {
            environment: "lab-environment".to_string(),
            desired_cpu_share: 0.9,
            desired_memory_bytes: 2 * 1024 * 1024 * 1024,
        },
    ) {
        Ok(decision) => {
            println!(
                "Granted {:.0}% CPU and {} MiB to {}",
                decision.granted_cpu_share * 100.0,
                decision.granted_memory_bytes / (1024 * 1024),
                decision.environment
            );
        }
        Err(err) => eprintln!("Resource arbitration failed: {err}"),
    }

    if let Err(err) = host_control.release_environment(&token.token, "lab-environment") {
        eprintln!("Failed to release environment: {err}");
    } else {
        println!("Environment released successfully");
    }
}
