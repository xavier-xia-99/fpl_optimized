//! Terminal display formatting

use crate::solver::ProblemResult;
use std::fmt::Write;

/// Display problem result to terminal
pub fn display_result(result: &ProblemResult) -> String {
    let mut output = String::new();

    // Header
    writeln!(&mut output, "\n╔══════════════════════════════════════════════════════════════╗").unwrap();
    writeln!(&mut output, "║  {:^58}  ║", result.problem_name).unwrap();
    writeln!(&mut output, "╠══════════════════════════════════════════════════════════════╣").unwrap();

    if let Some(gw) = result.gameweek {
        writeln!(&mut output, "║  Gameweek: {:47}  ║", format!("GW{}", gw.raw())).unwrap();
    }
    writeln!(&mut output, "║  Total Cost: {:44}  ║", result.total_cost).unwrap();
    writeln!(&mut output, "║  Expected Points: {:39.2}  ║", result.objective_value).unwrap();
    writeln!(&mut output, "╠══════════════════════════════════════════════════════════════╣").unwrap();

    // Starting Lineup
    writeln!(&mut output, "║  STARTING LINEUP                                             ║").unwrap();
    writeln!(&mut output, "╟──────────────────────────────────────────────────────────────╢").unwrap();
    writeln!(&mut output, "║  {:12} {:15} {:5} {:8} {:7} {:6}  ║", "Position", "Name", "Team", "Cost", "xP", "Own%").unwrap();
    writeln!(&mut output, "╟──────────────────────────────────────────────────────────────╢").unwrap();

    for player in result.lineup() {
        let captain_mark = if player.is_captain { "(C)" } else { "" };
        writeln!(
            &mut output,
            "║  {:12} {:15} {:5} {:>8} {:>7.2} {:>5.1}%  ║",
            player.position.short_name(),
            format!("{}{}", player.web_name, captain_mark),
            player.team_code,
            player.cost,
            player.expected_points,
            player.ownership_percent
        ).unwrap();
    }

    // Bench
    writeln!(&mut output, "╟──────────────────────────────────────────────────────────────╢").unwrap();
    writeln!(&mut output, "║  BENCH                                                       ║").unwrap();
    writeln!(&mut output, "╟──────────────────────────────────────────────────────────────╢").unwrap();

    for player in result.bench() {
        writeln!(
            &mut output,
            "║  {:12} {:15} {:5} {:>8} {:>7.2} {:>5.1}%  ║",
            player.position.short_name(),
            player.web_name,
            player.team_code,
            player.cost,
            player.expected_points,
            player.ownership_percent
        ).unwrap();
    }

    writeln!(&mut output, "╚══════════════════════════════════════════════════════════════╝").unwrap();

    output
}

/// Display compact summary
pub fn display_summary(result: &ProblemResult) -> String {
    let mut output = String::new();

    writeln!(&mut output, "\n{}", result.problem_name).unwrap();
    writeln!(&mut output, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").unwrap();

    if let Some(captain) = result.captain() {
        writeln!(&mut output, "Captain: {} ({:.2} xP)", captain.web_name, captain.expected_points).unwrap();
    }

    writeln!(&mut output, "Lineup xP: {:.2}", result.lineup_expected_points()).unwrap();
    writeln!(&mut output, "Total Cost: {}", result.total_cost).unwrap();

    writeln!(&mut output, "\nLineup:").unwrap();
    for player in result.lineup() {
        let captain_mark = if player.is_captain { " (C)" } else { "" };
        writeln!(
            &mut output,
            "  {} {} - {:.2} xP{}",
            player.position.short_name(),
            player.web_name,
            player.expected_points,
            captain_mark
        ).unwrap();
    }

    output
}
