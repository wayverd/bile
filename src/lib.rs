#![deny(rust_2018_idioms, unsafe_code)]
#![warn(
    absolute_paths_not_starting_with_crate,
    ambiguous_associated_items,
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    asm_sub_register,
    bad_asm_style,
    bindings_with_variant_name,
    break_with_label_and_loop,
    clashing_extern_declarations,
    coherence_leak_check,
    conflicting_repr_hints,
    confusable_idents,
    const_evaluatable_unchecked,
    const_item_mutation,
    dangling_pointers_from_temporaries,
    dead_code,
    deprecated_in_future,
    deprecated_where_clause_location,
    deprecated,
    deref_into_dyn_supertrait,
    deref_nullptr,
    drop_bounds,
    duplicate_macro_attributes,
    dyn_drop,
    ellipsis_inclusive_range_patterns,
    enum_intrinsics_non_enums,
    explicit_outlives_requirements,
    exported_private_dependencies,
    forbidden_lint_groups,
    function_item_references,
    future_incompatible,
    ill_formed_attribute_input,
    improper_ctypes_definitions,
    improper_ctypes,
    incomplete_features,
    incomplete_include,
    ineffective_unstable_trait_impl,
    inline_no_sanitize,
    invalid_atomic_ordering,
    invalid_doc_attributes,
    invalid_type_param_default,
    invalid_value,
    irrefutable_let_patterns,
    keyword_idents,
    large_assignments,
    late_bound_lifetime_arguments,
    legacy_derive_helpers,
    macro_expanded_macro_exports_accessed_by_absolute_paths,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    mixed_script_confusables,
    mutable_transmutes,
    named_arguments_used_positionally,
    named_asm_labels,
    no_mangle_const_items,
    no_mangle_generic_items,
    non_ascii_idents,
    non_camel_case_types,
    non_fmt_panics,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    nonstandard_style,
    noop_method_call,
    overflowing_literals,
    overlapping_range_endpoints,
    path_statements,
    patterns_in_fns_without_body,
    proc_macro_derive_resolution_fallback,
    pub_use_of_private_extern_crate,
    redundant_semicolons,
    repr_transparent_external_private_fields,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    semicolon_in_expressions_from_macros,
    soft_unstable,
    stable_features,
    text_direction_codepoint_in_comment,
    text_direction_codepoint_in_literal,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    tyvar_behind_raw_pointer,
    uncommon_codepoints,
    unconditional_panic,
    unconditional_recursion,
    unexpected_cfgs,
    uninhabited_static,
    unknown_crate_types,
    unnameable_test_items,
    unreachable_code,
    unreachable_patterns,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unstable_name_collisions,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_braces,
    unused_comparisons,
    unused_crate_dependencies,
    unused_doc_comments,
    unused_extern_crates,
    unused_features,
    unused_import_braces,
    unused_imports,
    unused_labels,
    unused_lifetimes,
    unused_macro_rules,
    unused_macros,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_qualifications,
    unused_unsafe,
    unused_variables,
    useless_deprecated,
    while_true
)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::cognitive_complexity,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_script_idents,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::float_cmp,
    clippy::float_equality_without_abs,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::iter_not_returning_iterator,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::missing_const_for_fn,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::nonstandard_macro_braces,
    clippy::nursery,
    clippy::option_if_let_else,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::pedantic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::suspicious_operation_groupings,
    clippy::todo,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::trivially_copy_pass_by_ref,
    clippy::unimplemented,
    clippy::unnecessary_wraps,
    clippy::unnested_or_patterns,
    clippy::unseparated_literal_suffix,
    clippy::unused_self,
    clippy::use_debug,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::wildcard_dependencies,
    clippy::wildcard_imports,
    clippy::zero_sized_map_values
)]
#![allow(clippy::missing_errors_doc)]

pub mod handlers;
pub mod utils;

pub mod config;

use std::{
    str,
    sync::{LazyLock, OnceLock},
    time::Duration,
};

use axum::{Router, http::StatusCode, routing::get};
use axum_response_cache::CacheLayer;
use syntect::parsing::SyntaxSet;
use tower_helmet::HelmetLayer;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::{
    config::Config,
    utils::response::{Css, Ico, Json, Png, Text},
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static CONFIG: OnceLock<Config> = OnceLock::new();
static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

static APPLE_TOUCH_ICON_PNG: &[u8] = include_bytes!("../assets/apple-touch-icon.png");
static FAVICON_ICO: &[u8] = include_bytes!("../assets/favicon.ico");
static ICON_192_MASKABLE: &[u8] = include_bytes!("../assets/icon-192-maskable.png");
static ICON_192: &[u8] = include_bytes!("../assets/icon-192.png");
static ICON_512_MASKABLE: &[u8] = include_bytes!("../assets/icon-512-maskable.png");
static ICON_512: &[u8] = include_bytes!("../assets/icon-512.png");
static MANIFEST_JSON: &str = include_str!("../assets/manifest.json");
static ROBOTS_TXT: &str = include_str!("../assets/robots.txt");
static STYLE_CSS: &str = include_str!("../assets/style.css");

static META_PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn config() -> &'static Config {
    CONFIG
        .get()
        .unwrap_or_else(|| unreachable!("failed to get global config, this should not happen"))
}

pub fn set_config(config: Config) {
    CONFIG.set(config).expect("config already set");
}

#[rustfmt::skip]
pub fn routes() -> Router {
    // let backend = MokaBackend::builder().max_entries(10_000).build();

    // let config = hitbox::Config::builder()
    //     .request_predicate(predicates::request::Method::new(http::Method::GET).unwrap())
    //     .response_predicate(hitbox::Neutral::new().status_code_class(predicates::response::StatusClass::Success))
    //     .extractor(extractors::Method::new())
    //     .policy(
    //         hitbox::policy::PolicyConfig::builder()
    //             .ttl(Duration::from_secs(60))
    //             .stale(Duration::from_secs(30))
    //             .build(),
    //     )
    //     .build();

    // let cache = hitbox_tower::Cache::builder()
    //     .backend(backend.clone())
    //     .config(config)
    //     .build();

    Router::new()
        .route("/", get(handlers::index::get))
        // assets
        .route("/apple-touch-icon.png", get(async || Png(APPLE_TOUCH_ICON_PNG)))
        .route("/favicon.ico", get(async || Ico(FAVICON_ICO)))
        .route("/icon-192-maskable.png", get(async || Png(ICON_192_MASKABLE)))
        .route("/icon-192.png", get(async || Png(ICON_192)))
        .route("/icon-512-maskable.png", get(async || Png(ICON_512_MASKABLE)))
        .route("/icon-512.png", get(async || Png(ICON_512)))
        .route("/manifest.json", get(async || Json(MANIFEST_JSON)))
        .route("/robots.txt", get(async || Text(ROBOTS_TXT)))
        .route("/style.css", get(async || Css(STYLE_CSS)))
        //
        .route("/{repo_name}", get(handlers::repo_home::get))
        .route("/{repo_name}/", get(handlers::repo_home::get))
        // git clone stuff
        .route("/{repo_name}/info/refs", get(handlers::git::get_1))
        .route("/{repo_name}/HEAD", get(handlers::git::get_1))
        .route("/{repo_name}/objects/{*obj}", get(handlers::git::get_2))
        // web pages
        .route("/{repo_name}/commit/{commit}", get(handlers::repo_commit::get))
        .route("/{repo_name}/refs", get(handlers::repo_refs::get))
        .route("/{repo_name}/refs/", get(handlers::repo_refs::get))
        .route("/{repo_name}/refs.xml", get(handlers::repo_refs_feed::get))
        .route("/{repo_name}/refs/{tag}", get(handlers::repo_tag::get))
        //
        .route("/{repo_name}/log", get(handlers::repo_log::get_1))
        .route("/{repo_name}/log/", get(handlers::repo_log::get_1))
        .route("/{repo_name}/log.xml", get(handlers::repo_log_feed::get_1))
        .route("/{repo_name}/log/{ref}", get(handlers::repo_log::get_2))
        .route("/{repo_name}/log/{ref}/", get(handlers::repo_log::get_2))
        .route("/{repo_name}/log/{ref}/feed.xml", get(handlers::repo_log_feed::get_2))
        .route("/{repo_name}/log/{ref}/{*object_name}", get(handlers::repo_log::get_3))
        //
        .route("/{repo_name}/tree", get(handlers::repo_file::get_1))
        .route("/{repo_name}/tree/", get(handlers::repo_file::get_1))
        .route("/{repo_name}/tree/{ref}", get(handlers::repo_file::get_2))
        .route("/{repo_name}/tree/{ref}/", get(handlers::repo_file::get_2))
        .route("/{repo_name}/tree/{ref}/item/{*object_name}", get(handlers::repo_file::get_3))
        .route("/{repo_name}/tree/{ref}/raw/{*object_name}", get(handlers::repo_file_raw::get))
        //
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
            CacheLayer::with_lifespan(Duration::from_secs(60)).use_stale_on_failure(),
            HelmetLayer::with_defaults(),
        ))
}
