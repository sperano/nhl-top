use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nhl::tui::reducers::standings_layout::build_standings_layout;
use nhl::tui::reducer::reduce;
use nhl::tui::state::AppState;
use nhl::tui::action::Action;
use nhl::commands::standings::GroupBy;
use nhl_api::Standing;
use std::sync::Arc;

/// Create sample standings data for benchmarking
fn create_sample_standings() -> Vec<Standing> {
    // Create 32 teams with realistic data
    vec![
        create_standing("Boston Bruins", "Atlantic", "Eastern", 1, 100),
        create_standing("Toronto Maple Leafs", "Atlantic", "Eastern", 2, 95),
        create_standing("Tampa Bay Lightning", "Atlantic", "Eastern", 3, 92),
        create_standing("Florida Panthers", "Atlantic", "Eastern", 4, 88),
        create_standing("Buffalo Sabres", "Atlantic", "Eastern", 5, 85),
        create_standing("Ottawa Senators", "Atlantic", "Eastern", 6, 80),
        create_standing("Detroit Red Wings", "Atlantic", "Eastern", 7, 75),
        create_standing("Montreal Canadiens", "Atlantic", "Eastern", 8, 70),

        create_standing("Carolina Hurricanes", "Metropolitan", "Eastern", 1, 98),
        create_standing("New Jersey Devils", "Metropolitan", "Eastern", 2, 94),
        create_standing("New York Rangers", "Metropolitan", "Eastern", 3, 90),
        create_standing("Pittsburgh Penguins", "Metropolitan", "Eastern", 4, 86),
        create_standing("New York Islanders", "Metropolitan", "Eastern", 5, 82),
        create_standing("Washington Capitals", "Metropolitan", "Eastern", 6, 78),
        create_standing("Columbus Blue Jackets", "Metropolitan", "Eastern", 7, 74),
        create_standing("Philadelphia Flyers", "Metropolitan", "Eastern", 8, 68),

        create_standing("Colorado Avalanche", "Central", "Western", 1, 102),
        create_standing("Dallas Stars", "Central", "Western", 2, 96),
        create_standing("Minnesota Wild", "Central", "Western", 3, 91),
        create_standing("Winnipeg Jets", "Central", "Western", 4, 87),
        create_standing("Nashville Predators", "Central", "Western", 5, 84),
        create_standing("St. Louis Blues", "Central", "Western", 6, 79),
        create_standing("Arizona Coyotes", "Central", "Western", 7, 73),
        create_standing("Chicago Blackhawks", "Central", "Western", 8, 66),

        create_standing("Vegas Golden Knights", "Pacific", "Western", 1, 99),
        create_standing("Edmonton Oilers", "Pacific", "Western", 2, 93),
        create_standing("Los Angeles Kings", "Pacific", "Western", 3, 89),
        create_standing("Seattle Kraken", "Pacific", "Western", 4, 85),
        create_standing("Calgary Flames", "Pacific", "Western", 5, 81),
        create_standing("Vancouver Canucks", "Pacific", "Western", 6, 76),
        create_standing("Anaheim Ducks", "Pacific", "Western", 7, 71),
        create_standing("San Jose Sharks", "Pacific", "Western", 8, 64),
    ]
}

fn create_standing(
    name: &str,
    division: &str,
    conference: &str,
    _div_rank: u32,
    points: i32,
) -> Standing {
    use nhl_api::LocalizedString;

    let wins = points / 2;
    let remaining = 82 - wins;
    let losses = remaining / 2;
    let ot_losses = remaining - losses;

    Standing {
        team_name: LocalizedString { default: name.to_string() },
        team_abbrev: LocalizedString {
            default: name.split_whitespace().last().unwrap_or("UNK").to_uppercase()
        },
        team_common_name: LocalizedString { default: name.to_string() },
        team_logo: format!("https://assets.nhle.com/logos/teams/{}.svg",
            name.split_whitespace().last().unwrap_or("UNK")),
        division_name: division.to_string(),
        division_abbrev: division.chars().take(3).collect(),
        conference_name: Some(conference.to_string()),
        conference_abbrev: Some(conference.chars().take(3).collect()),
        points,
        wins,
        losses,
        ot_losses,
    }
}

/// Benchmark standings layout computation for different views
fn bench_standings_layout(c: &mut Criterion) {
    let standings = create_sample_standings();

    let mut group = c.benchmark_group("standings_layout");

    group.bench_function("wildcard_view", |b| {
        b.iter(|| {
            build_standings_layout(
                black_box(&standings),
                black_box(GroupBy::Wildcard),
                black_box(false),
            )
        })
    });

    group.bench_function("division_view", |b| {
        b.iter(|| {
            build_standings_layout(
                black_box(&standings),
                black_box(GroupBy::Division),
                black_box(false),
            )
        })
    });

    group.bench_function("conference_view", |b| {
        b.iter(|| {
            build_standings_layout(
                black_box(&standings),
                black_box(GroupBy::Conference),
                black_box(false),
            )
        })
    });

    group.bench_function("league_view", |b| {
        b.iter(|| {
            build_standings_layout(
                black_box(&standings),
                black_box(GroupBy::League),
                black_box(false),
            )
        })
    });

    group.finish();
}

/// Benchmark reducer action dispatch
fn bench_reducer_dispatch(c: &mut Criterion) {
    let mut state = AppState::default();
    // Set up standings data
    let standings = Arc::new(Some(create_sample_standings()));
    state.data.standings = standings;

    let mut group = c.benchmark_group("reducer");

    group.bench_function("navigate_tab_right", |b| {
        b.iter(|| {
            let (new_state, _effect) = reduce(
                black_box(state.clone()),
                black_box(Action::NavigateTabRight),
            );
            new_state
        })
    });

    group.bench_function("standings_cycle_view", |b| {
        b.iter(|| {
            let (new_state, _effect) = reduce(
                black_box(state.clone()),
                black_box(Action::StandingsAction(
                    nhl::tui::action::StandingsAction::CycleViewRight,
                )),
            );
            new_state
        })
    });

    group.bench_function("enter_content_focus", |b| {
        b.iter(|| {
            let (new_state, _effect) = reduce(
                black_box(state.clone()),
                black_box(Action::EnterContentFocus),
            );
            new_state
        })
    });

    group.finish();
}

/// Benchmark state cloning (to measure overhead)
fn bench_state_operations(c: &mut Criterion) {
    let mut state = AppState::default();
    let standings = Arc::new(Some(create_sample_standings()));
    state.data.standings = standings;

    let mut group = c.benchmark_group("state_operations");

    group.bench_function("clone_full_state", |b| {
        b.iter(|| {
            let cloned = black_box(state.clone());
            cloned
        })
    });

    group.bench_function("clone_standings_arc", |b| {
        b.iter(|| {
            let cloned = black_box(state.data.standings.clone());
            cloned
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_standings_layout,
    bench_reducer_dispatch,
    bench_state_operations
);
criterion_main!(benches);
