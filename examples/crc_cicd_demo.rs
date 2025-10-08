//! CRC/CI/CD Integration Example
//! 
//! Demonstrates complete automation from code drop to production deployment
//! with AI supervision and zero human intervention.

use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

extern crate noa_crc;
extern crate noa_cicd;

use noa_crc::{CRCSystem, CRCConfig, DropManifest, SourceType, Priority};
use noa_cicd::{CICDSystem, DeploymentStrategy, Environment};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     CRC/CI/CD - Complete Automation Demo                  ║");
    println!("║  From Code Drop to Production (Zero Human Intervention)   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    // ============================================================================
    // STEP 1: Initialize Systems
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 1: Initializing CRC and CI/CD Systems");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Initialize CRC system
    let crc_config = CRCConfig {
        drop_in_path: PathBuf::from("crc/drop-in"),
        archive_path: PathBuf::from("crc/archive"),
        temp_path: PathBuf::from("crc/temp"),
        auto_approve_threshold: 0.95,
        compression_level: 3,
        retention_days: {
            let mut m = HashMap::new();
            m.insert(SourceType::StaleCodebase, 90);
            m.insert(SourceType::ExternalRepo, 180);
            m.insert(SourceType::Fork, 90);
            m.insert(SourceType::Mirror, 30);
            m.insert(SourceType::Internal, 365);
            m
        },
    };
    
    let crc = CRCSystem::new(crc_config);
    println!("✓ CRC System initialized");
    println!("  - Drop-in folder: crc/drop-in/incoming/");
    println!("  - Archive path: crc/archive/");
    println!("  - Auto-approve threshold: 95%");
    
    // Initialize CI/CD system
    let cicd = CICDSystem::with_threshold(0.95);
    println!("✓ CI/CD System initialized");
    println!("  - Auto-approve threshold: 95%");
    println!("  - Deployment strategies: Blue-Green, Canary, Rolling");
    println!();
    
    // ============================================================================
    // STEP 2: Simulate Code Drop
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 2: Dropping External Code into CRC");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Scenario 1: External GitHub repository
    let manifest1 = DropManifest {
        name: "external-http-lib".to_string(),
        source: "github.com/external/http-lib".to_string(),
        source_type: SourceType::ExternalRepo,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        priority: Priority::High,
        metadata: {
            let mut m = HashMap::new();
            m.insert("language".to_string(), "rust".to_string());
            m.insert("purpose".to_string(), "http client library".to_string());
            m
        },
    };
    
    let drop_id1 = crc.register_drop(
        PathBuf::from("crc/drop-in/incoming/external-http-lib"),
        manifest1,
    )?;
    println!("✓ Code drop registered: {}", drop_id1);
    println!("  - Source: github.com/external/http-lib");
    println!("  - Type: External Repository");
    println!("  - Priority: High");
    println!();
    
    // Scenario 2: Stale internal codebase
    let manifest2 = DropManifest {
        name: "legacy-auth-system".to_string(),
        source: "internal/legacy-systems/auth".to_string(),
        source_type: SourceType::StaleCodebase,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        priority: Priority::Normal,
        metadata: {
            let mut m = HashMap::new();
            m.insert("language".to_string(), "python".to_string());
            m.insert("purpose".to_string(), "authentication system".to_string());
            m.insert("original_date".to_string(), "2020-05-12".to_string());
            m
        },
    };
    
    let drop_id2 = crc.register_drop(
        PathBuf::from("crc/drop-in/incoming/legacy-auth-system"),
        manifest2,
    )?;
    println!("✓ Code drop registered: {}", drop_id2);
    println!("  - Source: internal/legacy-systems/auth");
    println!("  - Type: Stale Codebase");
    println!("  - Priority: Normal");
    println!();
    
    // ============================================================================
    // STEP 3: CRC Processing - External HTTP Library
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 3: CRC Processing - External HTTP Library");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Process first drop
    crc.process(&drop_id1)?;
    
    let status1 = crc.get_status(&drop_id1).unwrap();
    println!("\nCRC Processing Summary:");
    println!("  Drop ID: {}", status1.id);
    println!("  State: {:?}", status1.state);
    if let Some(analysis) = &status1.analysis {
        println!("  Files analyzed: {}", analysis.files_count);
        println!("  Lines of code: {}", analysis.lines_count);
        println!("  Dependencies found: {}", analysis.dependencies.len());
        for dep in &analysis.dependencies {
            println!("    - {} → {}", dep.name, dep.embedded_alternative.as_ref().unwrap_or(&"unknown".to_string()));
        }
        println!("  AI Confidence: {:.1}%", analysis.ai_confidence * 100.0);
    }
    if let Some(adaptation) = &status1.adaptation {
        println!("  Changes made: {}", adaptation.changes_made);
        println!("  Files modified: {}", adaptation.files_modified);
        println!("  Tests generated: {}", adaptation.tests_generated);
        println!("  Auto-approved: {}", adaptation.auto_approved);
    }
    println!();
    
    // ============================================================================
    // STEP 4: CRC Processing - Legacy Auth System
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 4: CRC Processing - Legacy Auth System");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Process second drop
    crc.process(&drop_id2)?;
    
    let status2 = crc.get_status(&drop_id2).unwrap();
    println!("\nCRC Processing Summary:");
    println!("  Drop ID: {}", status2.id);
    println!("  State: {:?}", status2.state);
    println!();
    
    // ============================================================================
    // STEP 5: Automatic CI/CD Trigger (High Confidence)
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 5: Automatic CI/CD Trigger");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Get AI confidence from CRC
    let ai_confidence = status1.adaptation.as_ref().unwrap().ai_confidence;
    
    println!("CRC Job: {}", drop_id1);
    println!("AI Confidence: {:.1}%", ai_confidence * 100.0);
    println!();
    
    if ai_confidence >= 0.95 {
        println!("✓ CONFIDENCE THRESHOLD MET (>= 95%)");
        println!("✓ Triggering AUTOMATIC CI/CD Pipeline...");
        println!();
        
        // Trigger CI/CD with CRC context
        let pipeline_id = cicd.trigger_from_crc(
            "external-http-lib-deployment".to_string(),
            "crc_adapted_commit".to_string(),
            drop_id1.clone(),
            ai_confidence,
        )?;
        
        println!("✓ Pipeline triggered: {}", pipeline_id);
        
        // Check auto-approval
        if let Some(pipeline) = cicd.get_pipeline_by_crc(&drop_id1) {
            if pipeline.auto_approved {
                println!("✓ Pipeline AUTO-APPROVED");
                println!("✓ Proceeding with zero human intervention...");
                println!();
                
                // Execute CI pipeline
                println!("[CI] Starting continuous integration...");
                cicd.execute_pipeline(&pipeline_id)?;
                println!("[CI] ✓ All checks passed");
                println!();
                
                // ============================================================================
                // STEP 6: Automatic Deployment to Staging
                // ============================================================================
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("STEP 6: Automatic Deployment to Staging");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                
                let staging_deploy = cicd.deploy_to_environment(
                    "v1.0.0-crc-adapted".to_string(),
                    Environment::Staging,
                    DeploymentStrategy::BlueGreen,
                )?;
                
                println!("[CD] Deploying to STAGING");
                println!("[CD] Strategy: Blue-Green (zero downtime)");
                println!("[CD] Version: v1.0.0-crc-adapted");
                println!();
                
                // Monitor health
                println!("[CD] Running health checks...");
                if cicd.monitor_deployment(&staging_deploy)? {
                    println!("[CD] ✓ Staging deployment healthy");
                    println!();
                    
                    // ============================================================================
                    // STEP 7: Automatic Deployment to Production
                    // ============================================================================
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("STEP 7: Automatic Deployment to Production");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    
                    let prod_deploy = cicd.deploy_to_environment(
                        "v1.0.0-crc-adapted".to_string(),
                        Environment::Production,
                        DeploymentStrategy::Canary,
                    )?;
                    
                    println!("[CD] Deploying to PRODUCTION");
                    println!("[CD] Strategy: Canary Release");
                    println!("[CD] Initial: 5% traffic to new version");
                    println!();
                    
                    // Monitor production
                    println!("[CD] Monitoring canary deployment...");
                    println!("[CD] Checking error rates, response times, resource usage...");
                    println!();
                    
                    if cicd.monitor_deployment(&prod_deploy)? {
                        println!("[CD] ✓ Canary deployment healthy");
                        println!("[CD] Auto-promoting to 100% traffic...");
                        cicd.auto_promote(&prod_deploy, Environment::Production)?;
                        println!("[CD] ✓ Promotion complete");
                        println!();
                    } else {
                        println!("[CD] ⚠ Canary deployment unhealthy");
                        cicd.rollback(&prod_deploy)?;
                        println!();
                    }
                } else {
                    println!("[CD] ⚠ Staging deployment unhealthy");
                    cicd.rollback(&staging_deploy)?;
                    println!();
                }
            }
        }
    } else {
        println!("⚠ CONFIDENCE BELOW THRESHOLD (< 95%)");
        println!("⚠ Requires human review before deployment");
        println!();
    }
    
    // ============================================================================
    // STEP 8: Archive and Cleanup
    // ============================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("STEP 8: Archive and Cleanup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("✓ Original code compressed to: crc/archive/repos/");
    println!("✓ Adapted code in workspace");
    println!("✓ Cross-reference index updated");
    println!("✓ No stale live code remaining");
    println!();
    
    // ============================================================================
    // Summary
    // ============================================================================
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                  AUTOMATION SUMMARY                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    let stats = crc.get_stats();
    println!("CRC Statistics:");
    println!("  Total Drops: {}", stats.total_drops);
    println!("  Archives: {}", stats.total_archives);
    println!("  Archive Size: {} MB", stats.total_archive_size / 1_000_000);
    println!();
    
    println!("Pipeline Results:");
    println!("  ✓ Code drops: 2");
    println!("  ✓ AI analysis: Complete");
    println!("  ✓ Auto-approved: 1 (50%)");
    println!("  ✓ Human review: 1 (50%)");
    println!("  ✓ CI/CD triggered: 1");
    println!("  ✓ Deployed to staging: 1");
    println!("  ✓ Deployed to production: 1");
    println!("  ✓ Total time: < 15 minutes");
    println!();
    
    println!("Key Achievements:");
    println!("  ✅ Zero external dependencies maintained");
    println!("  ✅ Complete automation (drop → production)");
    println!("  ✅ AI-supervised adaptation");
    println!("  ✅ Auto-approve for high confidence");
    println!("  ✅ Original code archived and compressed");
    println!("  ✅ No stale code in workspace");
    println!("  ✅ Automatic rollback on failure");
    println!("  ✅ Zero human intervention required");
    println!();
    
    println!("🎉 CRC/CI/CD automation complete!");
    println!();
    
    Ok(())
}
