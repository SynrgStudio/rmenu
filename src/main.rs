mod app_state;
mod fuzzy;
mod launcher;
mod modules;
mod ranking;
mod settings;
mod sources;
mod ui_win32;

use app_state::{AppState, LauncherItem, LauncherSource};
use atty;
use ranking::{rank_items, source_name};
use settings::{parse_args, CmdOptions, RmenuConfig};
use sources::{index_cache_size_bytes, load_launcher_items};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    time::Instant,
};
use ui_win32::{measure_ui_latencies, UiLatencyMetrics};

fn p95_duration_ms(samples: &mut [u128]) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    let idx = ((samples.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(samples.len() - 1);
    samples[idx]
}

fn estimated_dataset_bytes(items: &[LauncherItem]) -> usize {
    items
        .iter()
        .map(|item| item.label.len() + item.target.len())
        .sum()
}

fn print_metrics(
    app_state: &AppState,
    case_sensitive: bool,
    startup_ms: u128,
    ui_metrics: Option<UiLatencyMetrics>,
) {
    let mut queries: Vec<String> = vec![
        "pow".to_string(),
        "not".to_string(),
        "calc".to_string(),
        "code".to_string(),
        "expl".to_string(),
    ];

    for label in app_state
        .all_items
        .iter()
        .take(15)
        .map(|item| item.label.as_str())
    {
        let q: String = label.chars().take(3).collect::<String>().to_lowercase();
        if q.len() >= 2 {
            queries.push(q);
        }
    }

    let mut search_samples_ms: Vec<u128> = Vec::new();
    for query in queries {
        let t0 = Instant::now();
        let _ = rank_items(app_state, &query, case_sensitive);
        search_samples_ms.push(t0.elapsed().as_micros());
    }

    let p95_us = p95_duration_ms(&mut search_samples_ms);
    let p95_ms = p95_us as f64 / 1000.0;
    let cache_size = index_cache_size_bytes().unwrap_or(0);

    println!("rmenu metrics");
    println!("- startup_prepare_ms: {}", startup_ms);
    if let Some(ui) = ui_metrics {
        println!(
            "- time_to_window_visible_ms: {}",
            ui.time_to_window_visible_ms
        );
        println!("- time_to_first_paint_ms: {}", ui.time_to_first_paint_ms);
        println!("- time_to_input_ready_ms: {}", ui.time_to_input_ready_ms);
    }
    println!("- search_p95_ms: {:.3}", p95_ms);
    println!("- dataset_items: {}", app_state.all_items.len());
    println!(
        "- dataset_estimated_bytes: {}",
        estimated_dataset_bytes(&app_state.all_items)
    );
    println!("- index_cache_bytes: {}", cache_size);
}

fn main() -> windows::core::Result<()> {
    let startup_t0 = Instant::now();

    let cmd_options: CmdOptions = parse_args();
    let silent_mode = cmd_options.silent;

    let config_path_from_cli = cmd_options.config_path.as_ref().map(Path::new);

    let mut app_config = match RmenuConfig::load(config_path_from_cli) {
        Ok(cfg) => cfg,
        Err(e) => {
            if !silent_mode {
                eprintln!("Error loading configuration: {}. Using default config.", e);
            }
            RmenuConfig::default()
        }
    };

    app_config.apply_cli_overrides(&cmd_options);
    let launcher_config = app_config.launcher.clone();

    let mut initial_items: Vec<LauncherItem> = Vec::new();
    let mut launcher_mode = false;

    if let Some(elements_str) = &cmd_options.elements_str {
        initial_items = elements_str
            .split(app_config.behavior.element_delimiter)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| LauncherItem::new(s.to_string(), s.to_string(), LauncherSource::Direct))
            .collect();
    } else if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    initial_items = buffer
                        .lines()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            LauncherItem::new(s.to_string(), s.to_string(), LauncherSource::Direct)
                        })
                        .collect();
                }
            }
            Err(e) => {
                if !silent_mode {
                    eprintln!("Error reading from stdin: {}", e);
                }
            }
        }
    } else if launcher_config.launcher_mode_default {
        launcher_mode = true;
        initial_items = load_launcher_items(&launcher_config, silent_mode, cmd_options.reindex);
    }

    let final_initial_items = initial_items;
    let initial_app_state = AppState {
        current_input: String::new(),
        selected_index: 0,
        scroll_offset: 0,
        matching_items: final_initial_items.clone(),
        all_items: final_initial_items,
        prompt: cmd_options.prompt.clone(),
        launcher_mode,
        silent_mode,
        history_max_items: launcher_config.history_max_items,
        source_boost_history: launcher_config.source_boost_history,
        source_boost_start_menu: launcher_config.source_boost_start_menu,
        source_boost_path: launcher_config.source_boost_path,
    };

    let case_sensitive = app_config.behavior.case_sensitive;
    let mut module_runtime = modules::ModuleRuntime::new();
    module_runtime.configure_policy(modules::ModuleRuntimePolicy {
        provider_total_budget_ms: app_config.modules.provider_total_budget_ms,
        provider_timeout_ms: app_config.modules.provider_timeout_ms,
        max_items_per_provider_host: app_config.modules.max_items_per_provider_host,
        dedupe_source_priority: match app_config.modules.dedupe_source_priority {
            settings::DedupeSourcePriority::CoreFirst => modules::DedupeSourcePriority::CoreFirst,
            settings::DedupeSourcePriority::ProviderFirst => {
                modules::DedupeSourcePriority::ProviderFirst
            }
        },
        host_restart_backoff_ms: app_config.modules.host_restart_backoff_ms,
        max_ipc_payload_bytes: app_config.modules.max_ipc_payload_bytes,
    });
    let _module_api_version = module_runtime.api_version();
    module_runtime.register_builtin_module(Box::new(modules::BuiltinLifecycleModule));
    module_runtime.register_builtin_module(Box::new(modules::BuiltinQueryProviderModule));
    module_runtime.load_external_descriptors(&PathBuf::from("modules"), silent_mode);

    if cmd_options.modules_debug {
        println!("{}", module_runtime.modules_debug_report());
        return Ok(());
    }

    if cmd_options.metrics {
        let startup_ms = startup_t0.elapsed().as_millis();
        let ui_metrics = measure_ui_latencies(
            &cmd_options,
            &app_config,
            initial_app_state.clone(),
            module_runtime,
        )
        .ok();
        print_metrics(&initial_app_state, case_sensitive, startup_ms, ui_metrics);
        return Ok(());
    }

    if let Some(debug_query) = &cmd_options.debug_ranking {
        let ranked = rank_items(&initial_app_state, debug_query, case_sensitive);

        println!("Debug ranking for query: '{}'", debug_query);
        println!(
            "Dataset size: {} | case_sensitive={} | launcher_mode={}",
            initial_app_state.all_items.len(),
            case_sensitive,
            initial_app_state.launcher_mode
        );

        for (i, entry) in ranked.iter().take(20).enumerate() {
            println!(
                "{:>2}. total={:<5} fuzzy={:<5} boost={:<5} source={:<10} label={} target={}",
                i + 1,
                entry.total_score,
                entry.fuzzy_score,
                entry.source_boost,
                source_name(entry.item.source),
                entry.item.label,
                entry.item.target
            );
        }

        return Ok(());
    }

    let exit_code = ui_win32::run_ui(&cmd_options, &app_config, initial_app_state, module_runtime)?;
    std::process::exit(exit_code);
}
