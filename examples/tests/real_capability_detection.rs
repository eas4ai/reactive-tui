use reactive_tui::core::terminal_capabilities::TerminalCapabilities;
use std::time::Instant;

fn main() {
    println!("🧪 FRAMEWORK VALIDATION TEST");
    println!("============================");
    println!("Testing if the terminal capability detection framework works correctly");
    println!();

    let mut tests_passed = 0;
    let mut tests_failed = 0;

    // Test 1: API Functionality
    print!("🔧 Test 1: API Functionality... ");
    let start = Instant::now();
    match TerminalCapabilities::detect() {
        Ok(caps) => {
            println!("✅ PASS ({:.1}ms)", start.elapsed().as_millis());
            tests_passed += 1;

            // Run additional validation tests
            run_capability_validation_tests(&caps, &mut tests_passed, &mut tests_failed);
        }
        Err(e) => {
            println!("❌ FAIL - Detection failed: {}", e);
            tests_failed += 1;
        }
    }

    // Test 2: Performance Test
    print!("⚡ Test 2: Performance (multiple detections)... ");
    let start = Instant::now();
    let mut detection_times = Vec::new();

    for _ in 0..5 {
        let detect_start = Instant::now();
        match TerminalCapabilities::detect() {
            Ok(_) => detection_times.push(detect_start.elapsed().as_millis()),
            Err(_) => {
                println!("❌ FAIL - Detection failed during performance test");
                tests_failed += 1;
                break;
            }
        }
    }

    if detection_times.len() == 5 {
        let avg_time = detection_times.iter().sum::<u128>() / 5;
        let total_time = start.elapsed().as_millis();

        if avg_time < 100 && total_time < 1000 {
            println!(
                "✅ PASS (avg: {:.1}ms, total: {:.1}ms)",
                avg_time, total_time
            );
            tests_passed += 1;
        } else {
            println!(
                "❌ FAIL - Too slow (avg: {:.1}ms, total: {:.1}ms)",
                avg_time, total_time
            );
            tests_failed += 1;
        }
    }

    // Final Results
    println!();
    println!("📊 FRAMEWORK TEST RESULTS");
    println!("=========================");
    println!("✅ Tests Passed: {}", tests_passed);
    println!("❌ Tests Failed: {}", tests_failed);
    println!(
        "📈 Success Rate: {:.1}%",
        (tests_passed as f64 / (tests_passed + tests_failed) as f64) * 100.0
    );

    if tests_failed == 0 {
        println!();
        println!("🎉 FRAMEWORK VALIDATION: SUCCESS!");
        println!("   The terminal capability detection framework is working correctly.");
        println!("   Ready for production use.");
    } else {
        println!();
        println!("⚠️  FRAMEWORK VALIDATION: ISSUES DETECTED");
        println!("   Some tests failed. Framework needs fixes before production use.");
        std::process::exit(1);
    }
}

fn run_capability_validation_tests(
    caps: &TerminalCapabilities,
    passed: &mut i32,
    failed: &mut i32,
) {
    // Test 3: Data Integrity
    print!("🔍 Test 3: Data Integrity... ");
    let mut integrity_issues = 0;

    // Check that basic data makes sense
    if caps.terminal_info.size.0 == 0 || caps.terminal_info.size.1 == 0 {
        integrity_issues += 1;
    }

    // Check that capability score is reasonable
    let score = caps.capability_score();
    if score > 100 {
        integrity_issues += 1;
    }

    // Check that performance values are reasonable
    if caps.performance.input_latency_ms < 0.0 || caps.performance.input_latency_ms > 1000.0 {
        integrity_issues += 1;
    }

    if integrity_issues == 0 {
        println!("✅ PASS");
        *passed += 1;
    } else {
        println!("❌ FAIL - {} integrity issues", integrity_issues);
        *failed += 1;
    }

    // Test 4: API Consistency
    print!("🔄 Test 4: API Consistency... ");
    let settings1 = caps.recommended_settings();
    let settings2 = caps.recommended_settings();
    let score1 = caps.capability_score();
    let score2 = caps.capability_score();

    if settings1.max_fps == settings2.max_fps && score1 == score2 {
        println!("✅ PASS");
        *passed += 1;
    } else {
        println!("❌ FAIL - Inconsistent results");
        *failed += 1;
    }

    // Test 5: Feature Logic
    print!("🧠 Test 5: Feature Logic... ");
    let mut logic_issues = 0;

    // If true color is supported, color depth should not be monochrome
    if caps.graphics.truecolor
        && matches!(
            caps.graphics.color_depth,
            reactive_tui::core::terminal_capabilities::ColorDepth::Monochrome
        )
    {
        logic_issues += 1;
    }

    // If mouse wheel works, basic mouse should work
    if caps.input.mouse.wheel && !caps.input.mouse.basic {
        logic_issues += 1;
    }

    if logic_issues == 0 {
        println!("✅ PASS");
        *passed += 1;
    } else {
        println!("❌ FAIL - {} logic issues", logic_issues);
        *failed += 1;
    }
}
