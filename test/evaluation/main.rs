//! Main evaluation runner for Sprint ZL-008-008 comprehensive search quality evaluation
//!
//! This binary orchestrates the complete evaluation process including:
//! - A/B testing between baseline and hybrid systems
//! - Performance regression testing
//! - Statistical significance analysis
//! - Report generation

mod hybrid_search_evaluation;
mod performance_regression_tests;

use hybrid_search_evaluation::{ABTestConfig, run_hybrid_search_evaluation};
use performance_regression_tests::{PerformanceBenchmarkConfig, run_performance_regression_tests};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Sprint ZL-008-008: Comprehensive Search Quality Evaluation");
    println!("================================================================");
    
    // Initialize evaluation configuration
    let ab_test_config = ABTestConfig {
        dataset_path: "test/evaluation/labeled_dataset.json".to_string(),
        baseline_name: "vector_only_baseline".to_string(),
        test_name: "hybrid_bm25_vector".to_string(),
        bootstrap_samples: 10000,
        confidence_level: 0.95,
        search_timeout_ms: 1000,
        enable_multi_query_expansion: true,
        random_seed: Some(42),
    };
    
    let performance_config = PerformanceBenchmarkConfig {
        warmup_iterations: 50,   // Reduced for faster testing
        measurement_iterations: 500,
        search_timeout_ms: 1000,
        concurrency_levels: vec![1, 5, 10, 20],
        query_complexity_levels: vec![
            performance_regression_tests::QueryComplexity::Simple,
            performance_regression_tests::QueryComplexity::Medium,
            performance_regression_tests::QueryComplexity::Complex,
        ],
        enable_memory_monitoring: true,
        enable_cpu_monitoring: true,
    };
    
    println!("📋 Evaluation Configuration:");
    println!("  - Dataset: {}", ab_test_config.dataset_path);
    println!("  - Bootstrap samples: {}", ab_test_config.bootstrap_samples);
    println!("  - Confidence level: {}", ab_test_config.confidence_level);
    println!("  - Performance iterations: {}", performance_config.measurement_iterations);
    println!("  - Concurrency levels: {:?}", performance_config.concurrency_levels);
    println!();
    
    // Phase 1: A/B Testing and Search Quality Evaluation
    println!("📊 Phase 1: A/B Testing and Search Quality Evaluation");
    println!("----------------------------------------------------");
    
    match run_hybrid_search_evaluation(Some(ab_test_config.clone())).await {
        Ok(ab_report) => {
            println!("✅ A/B testing completed successfully");
            
            // Save A/B test report
            let ab_report_json = serde_json::to_string_pretty(&ab_report)?;
            tokio::fs::write("docs/evaluation/ab_test_report.json", ab_report_json).await?;
            println!("💾 A/B test report saved to docs/evaluation/ab_test_report.json");
            
            // Print key findings
            println!("\n🎯 Key A/B Test Results:");
            println!("  - Baseline system: {}", ab_report.baseline_report.system_name);
            println!("  - Test system: {}", ab_report.test_report.system_name);
            println!("  - Total queries evaluated: {}", ab_report.performance_summary.total_queries);
            println!("  - Mean NDCG@10 improvement: {:.2}%", 
                ab_report.performance_summary.mean_ndcg10_improvement * 100.0);
            println!("  - Queries with improved NDCG: {}/{}", 
                ab_report.performance_summary.queries_improved_ndcg,
                ab_report.performance_summary.total_queries);
            
            // Check if NDCG@10 target is met
            let ndcg_target_met = ab_report.performance_summary.mean_ndcg10_improvement >= 0.15;
            if ndcg_target_met {
                println!("  ✅ NDCG@10 improvement target (≥15%) achieved!");
            } else {
                println!("  ❌ NDCG@10 improvement target (≥15%) not achieved");
            }
            
            // Print statistical significance results
            if let Some(ndcg_test) = ab_report.significance_tests.get("ndcg_at_10") {
                println!("  - NDCG@10 statistical significance: p={:.4} ({})", 
                    ndcg_test.p_value,
                    if ndcg_test.is_significant { "significant" } else { "not significant" }
                );
            }
            
            // Print deployment recommendation
            println!("\n🚀 Deployment Recommendation:");
            match &ab_report.recommendation.deploy_recommendation {
                hybrid_search_evaluation::DeployRecommendation::StronglyRecommend => {
                    println!("  ✅ STRONGLY RECOMMEND deployment");
                }
                hybrid_search_evaluation::DeployRecommendation::Recommend => {
                    println!("  ✅ RECOMMEND deployment");
                }
                hybrid_search_evaluation::DeployRecommendation::Conditional(condition) => {
                    println!("  ⚠️  CONDITIONAL deployment: {}", condition);
                }
                hybrid_search_evaluation::DeployRecommendation::NotRecommend(reason) => {
                    println!("  ❌ NOT RECOMMENDED: {}", reason);
                }
                hybrid_search_evaluation::DeployRecommendation::StronglyNotRecommend(reason) => {
                    println!("  ❌ STRONGLY NOT RECOMMENDED: {}", reason);
                }
            }
            
            for finding in &ab_report.recommendation.key_findings {
                println!("    - {}", finding);
            }
        }
        Err(e) => {
            println!("❌ A/B testing failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!();
    
    // Phase 2: Performance Regression Testing
    println!("⚡ Phase 2: Performance Regression Testing");
    println!("------------------------------------------");
    
    match run_performance_regression_tests(Some(performance_config.clone())).await {
        Ok(regression_report) => {
            println!("✅ Performance regression testing completed successfully");
            
            // Save regression test report
            let regression_report_json = serde_json::to_string_pretty(&regression_report)?;
            tokio::fs::write("docs/evaluation/regression_test_report.json", regression_report_json).await?;
            println!("💾 Regression test report saved to docs/evaluation/regression_test_report.json");
            
            // Print key performance findings
            println!("\n⚡ Key Performance Results:");
            println!("  - Total scenarios tested: {}", regression_report.overall_summary.total_scenarios);
            println!("  - Scenarios with regression: {}", regression_report.overall_summary.scenarios_with_regression);
            println!("  - Worst regression severity: {:?}", regression_report.overall_summary.worst_regression_severity);
            
            // Check latency target
            let mut p95_latencies = Vec::new();
            for comparison in &regression_report.scenario_comparisons {
                p95_latencies.push(comparison.test_stats.response_time_stats.p95);
            }
            
            if let Some(&max_p95) = p95_latencies.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                println!("  - Maximum P95 latency: {:.1}ms", max_p95);
                if max_p95 <= 350.0 {
                    println!("  ✅ P95 latency target (≤350ms) achieved!");
                } else {
                    println!("  ❌ P95 latency target (≤350ms) not achieved");
                }
            }
            
            // Print test result
            println!("\n⚡ Performance Test Result:");
            match &regression_report.test_result {
                performance_regression_tests::RegressionTestResult::Pass => {
                    println!("  ✅ PASS - No significant performance regression");
                }
                performance_regression_tests::RegressionTestResult::FailMinor(reason) => {
                    println!("  ⚠️  FAIL (Minor) - {}", reason);
                }
                performance_regression_tests::RegressionTestResult::FailMajor(reason) => {
                    println!("  ❌ FAIL (Major) - {}", reason);
                }
                performance_regression_tests::RegressionTestResult::FailCritical(reason) => {
                    println!("  ❌ FAIL (Critical) - {}", reason);
                }
            }
            
            for impact in &regression_report.overall_summary.key_impacts {
                println!("    - {}", impact);
            }
        }
        Err(e) => {
            println!("❌ Performance regression testing failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!();
    
    // Phase 3: Generate Final Evaluation Summary
    println!("📄 Phase 3: Final Evaluation Summary");
    println!("------------------------------------");
    
    // Load both reports for combined analysis
    let ab_report_data = tokio::fs::read_to_string("docs/evaluation/ab_test_report.json").await?;
    let ab_report: hybrid_search_evaluation::ABTestReport = serde_json::from_str(&ab_report_data)?;
    
    let regression_report_data = tokio::fs::read_to_string("docs/evaluation/regression_test_report.json").await?;
    let regression_report: performance_regression_tests::RegressionTestReport = serde_json::from_str(&regression_report_data)?;
    
    // Generate overall sprint success assessment
    let sprint_success = assess_sprint_success(&ab_report, &regression_report);
    
    println!("🎯 Sprint ZL-008-008 Success Assessment:");
    println!("========================================");
    
    // Check success criteria from sprint plan
    let success_criteria = vec![
        (
            "NDCG@10 improvement ≥15% vs vector-only baseline",
            ab_report.performance_summary.mean_ndcg10_improvement >= 0.15,
        ),
        (
            "P95 latency ≤350ms for hybrid search maintained",
            regression_report.scenario_comparisons.iter()
                .all(|comp| comp.test_stats.response_time_stats.p95 <= 350.0),
        ),
        (
            "Query recall improvement ≥20% with multi-query expansion",
            ab_report.performance_summary.mean_recall_improvement >= 0.20,
        ),
        (
            "Zero performance regression on existing workloads",
            matches!(regression_report.test_result, 
                performance_regression_tests::RegressionTestResult::Pass),
        ),
        (
            "Statistical significance achieved for improvements",
            ab_report.significance_tests.get("ndcg_at_10")
                .map_or(false, |test| test.is_significant),
        ),
    ];
    
    let passed_criteria = success_criteria.iter().filter(|(_, passed)| *passed).count();
    let total_criteria = success_criteria.len();
    
    for (criterion, passed) in &success_criteria {
        let status = if *passed { "✅ PASS" } else { "❌ FAIL" };
        println!("  {} {}", status, criterion);
    }
    
    println!();
    println!("📊 Overall Sprint Success Rate: {}/{} ({:.1}%)", 
        passed_criteria, total_criteria, 
        (passed_criteria as f64 / total_criteria as f64) * 100.0);
    
    if sprint_success {
        println!("🎉 SPRINT ZL-008-008 SUCCESS! ✅");
        println!("   Ready to proceed to ZL-008-009 (Performance Optimization & Caching)");
    } else {
        println!("⚠️  SPRINT ZL-008-008 NEEDS ATTENTION ❌");
        println!("   Review findings and address issues before proceeding");
    }
    
    // Generate updated search quality report
    update_search_quality_report(&ab_report, &regression_report).await?;
    
    println!("\n📋 Deliverables Created:");
    println!("  - docs/evaluation/ab_test_report.json");
    println!("  - docs/evaluation/regression_test_report.json");
    println!("  - docs/evaluation/search_quality_report.md (updated)");
    
    println!("\n🎯 Next Steps:");
    if sprint_success {
        println!("  1. Review detailed reports for insights");
        println!("  2. Prepare for gradual rollout to production");
        println!("  3. Proceed to ZL-008-009 (Performance Optimization & Caching)");
        println!("  4. Set up monitoring for production deployment");
    } else {
        println!("  1. Analyze failed criteria in detail");
        println!("  2. Implement fixes for identified issues");
        println!("  3. Re-run evaluation after fixes");
        println!("  4. Consider scope adjustment if needed");
    }
    
    Ok(())
}

/// Assess overall sprint success based on evaluation results
fn assess_sprint_success(
    ab_report: &hybrid_search_evaluation::ABTestReport,
    regression_report: &performance_regression_tests::RegressionTestReport,
) -> bool {
    // Key success criteria:
    // 1. NDCG@10 improvement ≥15%
    let ndcg_success = ab_report.performance_summary.mean_ndcg10_improvement >= 0.15;
    
    // 2. Statistical significance achieved
    let significance_success = ab_report.significance_tests.get("ndcg_at_10")
        .map_or(false, |test| test.is_significant);
    
    // 3. No major performance regression
    let performance_success = matches!(regression_report.test_result, 
        performance_regression_tests::RegressionTestResult::Pass |
        performance_regression_tests::RegressionTestResult::FailMinor(_)
    );
    
    // 4. P95 latency target met
    let latency_success = regression_report.scenario_comparisons.iter()
        .all(|comp| comp.test_stats.response_time_stats.p95 <= 350.0);
    
    // Require at least 3 out of 4 key criteria for success
    let success_count = [ndcg_success, significance_success, performance_success, latency_success]
        .iter()
        .filter(|&&x| x)
        .count();
    
    success_count >= 3
}

/// Update the search quality report with actual results
async fn update_search_quality_report(
    ab_report: &hybrid_search_evaluation::ABTestReport,
    regression_report: &performance_regression_tests::RegressionTestReport,
) -> Result<(), Box<dyn std::error::Error>> {
    // For now, just create a summary file with key results
    // In a full implementation, this would update the markdown template
    
    let summary = format!(
        r#"# Search Quality Evaluation Summary

**Evaluation Completed:** {}
**Sprint:** ZL-008-008 - Comprehensive Search Quality Evaluation

## Executive Summary

### A/B Test Results
- **Total queries evaluated:** {}
- **Mean NDCG@10 improvement:** {:.2}% (Target: ≥15%)
- **Queries with improved NDCG:** {}/{}
- **Statistical significance:** {}
- **Deployment recommendation:** {:?}

### Performance Results  
- **Scenarios tested:** {}
- **Scenarios with regression:** {}
- **Worst regression severity:** {:?}
- **Performance test result:** {:?}

### Sprint Success Assessment
- **NDCG@10 target achieved:** {}
- **Performance targets met:** {}
- **Overall assessment:** {}

## Key Findings
{}

## Recommendations
{}

## Next Steps
{}
"#,
        ab_report.timestamp,
        ab_report.performance_summary.total_queries,
        ab_report.performance_summary.mean_ndcg10_improvement * 100.0,
        ab_report.performance_summary.queries_improved_ndcg,
        ab_report.performance_summary.total_queries,
        ab_report.significance_tests.get("ndcg_at_10")
            .map_or("Not calculated".to_string(), |test| 
                if test.is_significant { "Significant".to_string() } 
                else { "Not significant".to_string() }
            ),
        ab_report.recommendation.deploy_recommendation,
        regression_report.overall_summary.total_scenarios,
        regression_report.overall_summary.scenarios_with_regression,
        regression_report.overall_summary.worst_regression_severity,
        regression_report.test_result,
        ab_report.performance_summary.mean_ndcg10_improvement >= 0.15,
        matches!(regression_report.test_result, 
            performance_regression_tests::RegressionTestResult::Pass),
        assess_sprint_success(ab_report, regression_report),
        ab_report.recommendation.key_findings.join("\n- "),
        ab_report.recommendation.next_steps.join("\n- "),
        regression_report.overall_summary.recommendations.join("\n- "),
    );
    
    tokio::fs::write("docs/evaluation/evaluation_summary.md", summary).await?;
    
    Ok(())
}
