# 100 New Goals — Post-V1 (Goals 151–250)

> All 150 previous goals (V1 + Phase 9–14) are complete and archived in `CHANGELOG.md`.
> These 100 new goals build on top of the existing implementation — extending, deepening,
> and adding net-new capabilities that were not part of the original roadmap.

> **Status: 100/100 implemented — Goals 151–250 complete (Phases 15–23).**

---

## Phase 15: AI-Native Tooling & Agent Integration (Goals 151–165)

> rye is positioned as the first AI-optimized UI framework (see `docs/27-AI-OPTIMIZATION.md`).
> These goals make that positioning concrete with tooling, protocols, and agent-facing APIs.

151. ✅ **Implement `rpg explain` CLI command** — Programmatic error code lookup with text and JSON output. `rpg explain R003`, `rpg explain --list`, `rpg explain --search "signal"`, `rpg explain --category ai`. — `rye-cli/src/explain.rs`, `rye-core/src/error_codes.rs`

152. ✅ **Implement `rpg explain --json` output mode** — Machine-readable JSON output with `error_code`, `category`, `message`, `suggestion`, `correct_example`, `common_causes[]`, `related_errors[]`. — `rye-cli/src/explain.rs`

153. ✅ **Implement component discovery API** — `component_registry::register()`, `find()`, `list_all()`, `search()`, `list_by_category()`, `format_all_json()`. Runtime introspection with `ComponentMeta` and `PropInfo` structs. — `rye-core/src/component_registry.rs`

154. ✅ **Implement `rpg scaffold` with AI-friendly templates** — `rpg scaffold component Button --props label:String,disabled:bool --style --test`. Also: `scaffold page`, `scaffold store`, `scaffold action`. Generates typed props, style blocks, tests, mod.rs registration. — `rye-cli/src/scaffold.rs`

155. ✅ **Implement `rpg test --generate` AI test scaffolding** — `rpg test --generate src/components/button.rs`, `--generate --all`, `--generate --dir`. Parses source for components, props, island attributes. Generates render, prop, event, and island tests. — `rye-cli/src/test_gen.rs`

156. ✅ **Implement `rpg lint --ai` static analysis pass** — AI-aware linter checking for R801–R809 patterns: missing `move`, missing `.get()`, direct Signal assignment, use_effect for derived state, unnecessary clones, raw async. `--json` output, `--dir` mode. — `rye-cli/src/lint.rs`, `rye-core/src/ai/code_review.rs`

157. ✅ **Implement MCP server for rye** — `rye-mcp` crate with 16 MCP tools over JSON-RPC 2.0 stdio. Tools: `rye_explain_error`, `rye_list_error_codes`, `rye_search_error_codes`, `rye_get_recovery_plan`, `rye_list_components`, `rye_find_component`, `rye_search_components`, `rye_nl_search_components`, `rye_list_prompt_templates`, `rye_get_prompt_template`, `rye_review_code`, `rye_get_context`, `rye_get_focused_context`, `rye_scaffold_component`, `rye_scaffold_test`, `rye_component_usage_stats`. — `crates/rye-mcp/`

158. ✅ **Implement AI-specific error codes R800–R899** — 11 AI-specific error codes (R800–R810): wrong prop type, missing `move`, signal read without `.get()`, signal write without `.set()`, non-PascalCase name, missing `#[component]`, use_effect for derived state, unnecessary clone, prop drilling, raw async, template outside component. — `rye-core/src/error_codes.rs`

159. ✅ **Implement AI prompt templates for common patterns** — 10 templates: component, form, list, page, store, action, island, crud, modal, auth. Placeholder filling, category filtering, JSON export. — `rye-core/src/ai/prompt_templates.rs`

160. ✅ **Implement AI context window optimization** — `ContextBudget` with token tracking, `generate_context_package()`, `generate_focused_context()`. Compact and detailed component summaries. — `rye-core/src/ai/context_optimizer.rs`

161. ✅ **Implement `rpg doctor` project health check** — 10 checks: Cargo.toml, rye dependency, edition, src/ dir, entry point, components, tests, .gitignore, rye.toml, Cargo.lock. `[OK]`/`[WARN]`/`[FAIL]` output with fix suggestions. `--json` mode. — `rye-cli/src/doctor.rs`

162. ✅ **Implement AI-friendly error recovery suggestions** — Step-by-step `RecoveryPlan` for R800–R809. Each plan: ordered steps with code examples, verification, common mistakes, alternatives. Text and JSON output. — `rye-core/src/ai/error_recovery.rs`

163. ✅ **Implement component usage analytics** — Global tracker with `record_definition`/`record_usage`, `stats_for()`, `all_stats()`, `most_used()`, `unused_components()`, `scan_source()`. — `rye-core/src/ai/usage_analytics.rs`

164. ✅ **Implement AI code review integration** — `review_source()` checks for 8 error patterns + 3 praise patterns. Severity: Error, Warning, Info, Praise. 0–100 score. Text and JSON output. — `rye-core/src/ai/code_review.rs`

165. ✅ **Implement natural language component search** — `search_nl("a button that submits a form")` → ranked results. Scores across name, description, category, tags, props. Semantic synonyms (button↔click, modal↔popup). — `rye-core/src/ai/nl_search.rs`

---

## Phase 16: Advanced Reactivity & State (Goals 166–175)

> Extends the signal system (goals 26–29, 76) with advanced patterns for complex applications.

166. ✅ **Implement derived signal selectors** — `Selector<T>` with `new()`, `new_with_eq()`, `select()` helper. Only recomputes when the selected slice changes. Structural sharing via custom equality function. — `rye-signals/src/selector.rs`

167. ✅ **Implement signal graph pruning** — `prune()` scans for signals with no subscribers and detaches them. `pin()`/`unpin()` to protect critical signals. `reattach()` on next read. Global enable/disable. — `rye-signals/src/prune.rs`

168. ✅ **Implement time-travel debugging with snapshot export** — `snapshot::register()`, `export()`, `import()`, `checkpoint()`, `restore()`, `export_json()`. History with configurable max size. Labeled snapshots for debugging. — `rye-signals/src/snapshot.rs`

169. ✅ **Implement saga pattern for complex async flows** — `Saga<T, E>` with `step()`, `run()`, automatic reverse-order compensation on failure. `SagaBuilder` for fluent construction. `SagaState` tracking (Pending, Running, Completed, Failed, CompensationFailed). — `rye-signals/src/saga.rs`

170. ✅ **Implement optimistic updates with automatic rollback** — `optimistic_update_sync()` and `optimistic_update()` (async). `OptimisticUpdate` struct with `confirm()`/`rollback()`. `OptimisticResult` enum. Previous value tracking. — `rye-signals/src/optimistic.rs`

171. ✅ **Implement signal persistence strategies** — `PersistenceStrategy` trait, `MemoryPersistence`, `NoopPersistence`, `CustomPersistence`. `PersistedSignal<T>` with auto-save on set/update. `persist()` helper. `PersistenceType` enum (Memory, LocalStorage, SessionStorage, UrlParams, Cookie, Custom). — `rye-signals/src/persistence.rs`

172. ✅ **Implement computed signals with debounce/throttle** — `Debounced<T>` and `Throttled<T>` with `flush()`, `source_value()`. `debounced()` and `throttled()` helpers. Duration-based update gating. — `rye-signals/src/debounce.rs`

173. ✅ **Implement signal batching with priority** — `Priority` enum (High, Normal, Low). `batch_high()`, `batch_normal()`, `batch_low()`. `set_signal_priority()`, `notify_with_priority()`. Priority-ordered flush. — `rye-signals/src/priority_batch.rs`

174. ✅ **Implement reactive URL state synchronization** — `UrlState` with bidirectional sync, `on_change()` listeners, `sync_from_url()`, `clear()`. `parse_query_string()`/`build_query_string()` helpers. WASM `BroadcastChannel` + History API support. — `rye-core/src/url_state.rs`

175. ✅ **Implement cross-tab state synchronization** — `CrossTabSync<T>` with `set()`/`receive()`/`on_change()`. `CrossTabRegistry` for channel management. `CrossTabStore` for key-value sync. WASM `BroadcastChannel` support. — `rye-core/src/cross_tab.rs`

---

## Phase 17: Rendering Deep Cuts (Goals 176–185)

> Extends rendering capabilities (goals 30–40, 111–120) with advanced techniques.

176. ✅ **Implement offscreen rendering / prerendering** — `PrerenderedNode`, `PrerenderCache` with LRU eviction, `PrerenderPriority`, `PrerenderScheduler` with priority-ordered queue. Global cache via thread-local. Route preloading and tab pre-rendering support. — `rye-core/src/offscreen.rs`

177. ✅ **Implement render-to-texture for native** — `TextureId`, `TextureConfig`, `TextureFormat` (RGBA8, BGRA8, RGB8, R8), `RenderedTexture` with pixel manipulation, `TextureRenderer` for component-to-texture rendering, `DragPreview` for drag-and-drop thumbnails. — `rye-core/src/render_to_texture.rs`

178. ✅ **Implement custom render hooks** — `RenderHook` trait, `RenderHookResult` (Pass, Replace, Wrap, ModifyAttrs), `RenderContext` with attribute access, `RenderHookRegistry` for hook management. Global registry via thread-local. — `rye-core/src/render_hooks.rs`

179. ✅ **Implement shadow DOM encapsulation** — `ShadowMode` (Open, Closed), `ShadowRoot` with style sheet management, `Shadow` component with `attach_script()` generation, `ShadowStyleSheetRegistry` for global style tracking. — `rye-core/src/shadow_dom.rs`

180. ✅ **Implement CSS Houdini Paint API bridge** — `PaintWorklet` with custom painter functions, `PaintContext` with CSS custom properties, `PaintOutput` (solid color, gradient), `PaintWorkletRegistry`, `WgpuShaderFallback` for native platforms, `use_paint_worklet()` global hook. — `rye-core/src/houdini.rs`

181. ✅ **Implement element-level lazy hydration** — `ElementHydrationStrategy` (Immediate, OnVisible, OnInteraction, OnIdle, Delayed), `ElementHydrationConfig`, `ElementHydrationManager` with per-element tracking, callbacks, intersection observer script generation. Global manager via thread-local. — `rye-core/src/element_hydration.rs`

182. ✅ **Implement render delegation / render props** — `RenderProp<T>`, `IndexedRenderProp<T>`, `OptionRenderProp<T>`, `RenderPropComponent<T>`, `SwitchRenderProp<T>` with case/default pattern. Clone support via `Rc`. — `rye-core/src/render_props.rs`

183. ✅ **Implement static template extraction** — `StaticTemplate` with pre-rendered HTML, `StaticTemplateRegistry`, `is_static_node()` checker, `extract_static_html()`, `analyze_template()` with static/dynamic node counts and depth analysis. Global registry via thread-local. — `rye-core/src/static_template.rs`

184. ✅ **Implement dual-pass rendering** — `DualPassRenderer` with `SkeletonBuilder`, `SkeletonPlaceholder`, placeholder fill/drain, `finalize()`, batch patch script/JSON generation. Works with streaming SSR for immediate skeleton delivery. — `rye-core/src/dual_pass.rs`

185. ✅ **Implement component-level error boundaries with retry strategies** — `RetryStrategy` (None, Fixed, ExponentialBackoff, FallbackCached, FallbackStatic, Custom), `RetryState` (Ok, Waiting, Retrying, Failed), `RetryErrorBoundary` with `report_error()`, `retry()`, `succeed()`, `fail()`, `reset()`, `render()` with cached/static fallback. — `rye-core/src/retry_boundary.rs`

---

## Phase 18: Server & Full-Stack Deep Cuts (Goals 186–195)

> Extends server capabilities (goals 121–130) with production-grade features.

186. ✅ **Implement server-side rendering with data loading patterns** — `Loader` trait with `path()`/`load()`, `LoaderData` with status/redirect/cache-control builder, `LoaderResult` (Ok, Error, Redirect), `LoaderRegistry` with route pattern matching and param extraction, `LoaderRequest` with session/query builder. — `rye-ssr/src/server/loader.rs`

187. ✅ **Implement API routes with OpenAPI generation** — `ApiRoute` with params/responses/tags, `ApiRouteBuilder` fluent API, `ApiRouteRegistry` with `to_json()` generating OpenAPI 3.1 spec, `swagger_ui_html()` for Swagger UI at `/docs`, `ParamLocation` (Path, Query, Header, Cookie), base64-encoded spec embedding. — `rye-ssr/src/server/api_routes.rs`

188. ✅ **Implement server-sent events with typed channels** — `SseEventType` trait for type-safe events, `SseEvent<T>` with wire format serialization, `SseChannel<T>` with send/flush/peek, `SseChannelRegistry` for typed channel management, `SseReceiver<T>` with `event_source_script()` and `parse_event()`, `parse_sse_stream()` helper. — `rye-ssr/src/server/typed_sse.rs`

189. ✅ **Implement distributed SSR with session affinity** — `ServerNode` with health/load/connections, `AffinityStrategy` (Sticky, LeastConnections, RegionPreferred, RoundRobin), `SessionAffinityRouter` with session pinning, health marking, load updating, server removal with session cleanup. — `rye-ssr/src/server/session_affinity.rs`

190. ✅ **Implement partial SSR re-rendering** — `SubtreeDiff` with change detection, `to_patch_json()`/`to_patch_script()` for client-side DOM patches, `PartialRenderer` with subtree registration, `rerender()`/`rerender_batch()`, `batch_patch_script()`/`batch_patch_json()` for multi-component updates. — `rye-ssr/src/server/partial_rerender.rs`

191. ✅ **Implement server-side signal hydration** — `SignalHydrationData` with signal map and `to_json()`/`from_json()`/`to_script_tag()`, `ServerSignalSerializer` for collecting signal state, `ClientSignalDeserializer` with `get_signal()`/`get_signal_parsed()` for restoring signal state without loading flash. — `rye-ssr/src/server/signal_hydration.rs`

192. ✅ **Implement request-scoped context** — `RequestContext` with request_id, user_id, locale, theme, IP, user_agent, custom data, headers. Builder pattern with `with_user()`/`with_locale()`/`with_theme()` etc. `from_headers()` for automatic extraction from HTTP headers. `to_script_tag()` for client-side hydration. — `rye-ssr/src/server/request_context.rs`

193. ✅ **Implement SSR compression with Brotli/Zstd** — `CompressionAlgorithm` (Brotli, Zstd, Gzip, None), `CompressionConfig` with quality/min_size/content-type awareness, `CompressionMiddleware` with `process_response()` returning `CompressionDecision`, `should_compress()` checks content-type and size, `from_accept_encoding()` negotiation. — `rye-ssr/src/server/compression.rs`

194. ✅ **Implement database integration layer** — `rye-db` crate with `ConnectionPool` (max connections, min idle, acquire/release), `QueryBuilder` (SELECT/INSERT/UPDATE/DELETE with WHERE/ORDER BY/LIMIT/OFFSET), `QueryResult` with row/column access, `ReactiveQuery` with `use_query_db()` that re-runs on signal change, `ReactiveQueryCache` for result caching. — `crates/rye-db/` (`pool.rs`, `query.rs`, `reactive.rs`)

195. ✅ **Implement cron / scheduled tasks for SSR apps** — `Schedule` (Every, Cron, Once, OnStartup) with `parse()` for human-readable intervals, `ScheduledTask` with enable/disable and `is_due()` checking, `TaskScheduler` with `register()`/`tick()`/`run_task()`/`run_startup_tasks()`, global scheduler via `schedule()` and `run_due()`. — `rye-ssr/src/server/cron.rs`

---

## Phase 19: Native & Mobile Deep Cuts (Goals 196–210)

> Extends native rendering (goals 56–70, 111–120) with production mobile/desktop features.

196. ✅ **Implement native module system** — `NativePlatform` (Ios, Android, Desktop, Web), `NativeType` with Swift/Kotlin type mapping, `NativeFunction` with sync/async support, `NativeModule` with function registry, `NativeModuleBuilder` fluent API, `NativeModuleRegistry` for multi-module management, `generate_swift_bindings()`/`generate_kotlin_bindings()`/`generate_rust_bindings()` for cross-platform binding generation. — `rye-mobile/src/native_module.rs`

197. ✅ **Implement native push notifications** — `PushPermissionState` (NotDetermined, Granted, Denied, Provisional, Unsupported), `PushNotification` with title/body/data/sound/badge, `NotificationChannel` with importance levels, `NotificationAction` with foreground/destructive support, `PushNotificationManager` with permission/token/channel management and `send()` with JSON serialization. — `rye-mobile/src/push_notifications.rs`

198. ✅ **Implement native biometric authentication** — `BiometricType` (Face, Fingerprint, Iris, Voice, None), `BiometricAvailability` (Available, NoHardware, NotEnrolled, LockedOut, Unavailable), `BiometricAuthResult` (Success, Failed, Cancelled, NotAvailable, Lockout), `BiometricAuthConfig` with fallback/reason/title, `BiometricAuthManager` with `authenticate()` and availability checking. — `rye-mobile/src/biometric.rs`

199. ✅ **Implement native share sheet** — `ShareContent` (Text, Url, TextAndUrl, File, Files, Image) with `as_text()`/`has_url()`/`is_file()`, `ShareResult` (Success, Cancelled, NotAvailable, Error), `ShareConfig` with `ShareDestination` filtering, `ShareManager` with `share()`/`share_text()`/`share_url()` and availability detection. — `rye-mobile/src/share.rs`

200. ✅ **Implement native camera & photo gallery** — `CameraDirection` (Back, Front), `CaptureType` (Photo, Video), `CameraConfig` with builder, `CapturedMedia` with path/metadata, `CameraResult`/`GalleryResult` enums, `CameraManager` with `capture()`/`pick_from_gallery()`/`pick_multiple()` and permission management. — `rye-mobile/src/camera.rs`

201. ✅ **Implement native geolocation** — `GeoAccuracy` (Low/Medium/High/Best), `GeoCoordinates` with Haversine `distance_to()`, `GeoConfig` with one-shot/background/min-distance builder, `GeoResult` enum, `GeofenceRegion` with `contains()` check, `GeofenceEvent`/`GeofenceEventType`, `GeolocationManager` with tracking and geofence management. — `rye-mobile/src/geolocation.rs`

202. ✅ **Implement native contacts access** — `Contact` with phone/email/address/organization fields, `ContactField`/`ContactAddress` with `formatted()`, `ContactsConfig` with fetch flags/limit/search query, `ContactsResult` enum, `ContactsManager` with `fetch()`/`get_by_id()`/`count()` and search filtering. — `rye-mobile/src/contacts.rs`

203. ✅ **Implement native local notifications** — `LocalNotification` with title/body/data/sound/badge, `NotificationTrigger` (TimeInterval, Calendar, Daily, Weekly, OnAppForeground, Immediate), `NotificationPermissionState`, `ScheduledNotification` with delivery tracking, `LocalNotificationsManager` with `schedule()`/`cancel()`/`deliver_due()`/`clear_delivered()`. — `rye-mobile/src/local_notifications.rs`

204. ✅ **Implement native in-app purchases** — `ProductType` (Consumable, NonConsumable, AutoRenewableSubscription, NonRenewingSubscription), `Product` with price/currency/period, `PurchaseState` (Pending/Purchased/Restored/Failed/Active/Expired/...), `Purchase` with `needs_acknowledgment()`, `PurchaseResult` enum, `IapManager` with `purchase()`/`restore_purchases()`/`acknowledge()`/`consume()`. — `rye-mobile/src/iap.rs`

205. ✅ **Implement native deep linking** — `DeepLink` with `parse()` URL parser (scheme/host/path/query), `path_segment()`/`query_param()`/`path_string()`, `DeepLinkRoute` with pattern matching (`:param` syntax) and `extract_params()`, `DeepLinkManager` with `register_route()`/`handle_url()`/`last_link()`/`handled_count()`. — `rye-mobile/src/deep_link.rs`

206. ✅ **Implement native background tasks** — `BackgroundTaskType` (BackgroundFetch, BackgroundProcessing, BackgroundSync) with default timeouts, `TaskConstraints` (network/charging/idle/battery), `TaskState` (Scheduled/Running/Completed/Failed/Cancelled), `TaskOutcome` (NewData/NoData/Failed/Reschedule), `BackgroundTask` with `run()`, `BackgroundTaskScheduler` with `register()`/`schedule()`/`run_ready()`/`cancel()`. — `rye-mobile/src/background_tasks.rs`

207. ✅ **Implement native haptic feedback** — `HapticImpact` (Light/Medium/Heavy/Rigid/Soft) with duration/intensity, `HapticNotification` (Success/Warning/Error) with vibration patterns, `HapticSelection` (Default/Soft), `HapticPattern` with on/off timings and amplitudes, `HapticsManager` with `impact()`/`notification()`/`selection()`/`pattern()` and enable/disable. — `rye-mobile/src/haptics.rs`

208. ✅ **Implement native permissions manager** — `Permission` enum (Camera/Microphone/Location/Contacts/Photos/Notifications/Biometric/Calendar/Reminders/Motion/Bluetooth/LocalNetwork), `PermissionState` (NotDetermined/Granted/Denied/Restricted/Limited/NotSupported), `PermissionRequestResult`, `PermissionsManager` with `request()`/`get_state()`/`get_reactive_state()` returning `Signal<PermissionState>`, `granted_permissions()`/`denied_permissions()`. — `rye-mobile/src/permissions.rs`

209. ✅ **Implement native app lifecycle persistence** — `StorageType` (UserDefaults/SharedPreferences/IndexedDb/LocalStorage/SessionStorage/Memory) with `is_persistent()`, `StateSnapshot` with signal map and `to_json()`/`from_json()` serialization, `LifecyclePersistenceManager` with `save_signal()`/`save()`/`restore()`/`restore_from_json()`/`to_json()`. — `rye-mobile/src/lifecycle_persistence.rs`

210. ✅ **Implement native widget / live activity support** — `WidgetPlatform` (Ios, Android), `WidgetSize` (Small/Medium/Large/ExtraLarge/LockScreen) with platform support, `WidgetBinding` for signal-to-widget data binding, `WidgetDefinition` with sizes/bindings/update interval, `WidgetState` with values/visibility, `WidgetManager` with `register()`/`create_instance()`/`update_value()`. — `rye-mobile/src/widgets.rs`

---

## Phase 20: Performance & Optimization Deep Cuts (Goals 211–220)

> Extends performance work (goals 101–110) with next-level optimizations.

211. ✅ **Implement incremental hydration** — Instead of hydrating the entire page at once, hydrate components incrementally as the browser becomes idle. Priority queue based on viewport proximity and interaction likelihood. Builds on existing progressive hydration (goal 125).

212. ✅ **Implement Wasm GC proposal support** — When browsers ship WasmGC, switch from `wasm-bindgen` reference type emulation to native GC types. Reduces Wasm binary size by ~20-30% and improves JS interop performance. Feature-flagged for browsers without WasmGC support. Builds on existing Wasm optimization (goals 101–110).

213. ✅ **Implement component-level code generation** — Generate specialized Rust code per component instance at compile time. Eliminates dynamic dispatch, inlines prop access, constant-folds static template parts. ~10-15% render performance improvement. Builds on existing template macro (goal 34).

214. ✅ **Implement layout caching** — Cache Taffy layout results for identical component configurations. If a component's props and children haven't changed, reuse the cached layout. Avoids re-running flexbox math. Builds on existing layout engine (goal 57) and arena allocator (goal 102).

215. ✅ **Implement text shaping cache** — Cache `cosmic-text` shaping results for identical text + font + size combinations. Text shaping is expensive — caching eliminates redundant work for static text. Builds on existing native renderer (goal 56).

216. ✅ **Implement GPU resource pooling** — Pool GPU buffers, textures, and pipelines in wgpu. Reuse across components instead of creating/destroying. Reduces GPU memory pressure and allocation latency. Builds on existing WebGPU integration (goal 111).

217. ✅ **Implement speculative preloading** — Predict which route the user is likely to navigate to next (based on hover, scroll position, link proximity) and preload its Wasm chunk and data. Zero-cost if prediction is wrong. Builds on existing code splitting (goal 103) and router (goal 71).

218. ✅ **Implement render coalescing** — When multiple signals update in the same frame, coalesce all DOM mutations into a single batch. Extend existing batch protocol (goal 101) with frame-aware scheduling using `requestAnimationFrame`. Eliminates layout thrash from rapid signal updates.

219. ✅ **Implement Wasm precompilation** — Pre-compile Wasm to native code during `rpg build` using `wizer` or similar. Ship pre-initialized Wasm that starts faster. Reduces cold-start time by ~40%. Builds on existing streaming Wasm compilation (goal 104).

220. ✅ **Implement selective Wasm AOT** — Profile-guided AOT compilation of hot paths in Wasm to native code. Uses `cranelift` to compile frequently-executed render paths ahead of time. Hybrid interpreter + AOT execution. Builds on existing Wasm threading (goal 107).

---

## Phase 21: Developer Experience & Ecosystem (Goals 221–235)

> Extends DX (goals 82–95, 146–150) with next-generation developer tools.

221. ✅ **Implement `rpg playground` online editor** — Web-based rye code editor with live preview. Write rye components in the browser, see rendered output instantly. Shareable URLs for code snippets. Powered by Wasm. Builds on existing storybook (goal 134) and documentation site (goal 95).

222. ✅ **Implement `rpg doctor` health check** — Diagnoses common project issues: missing dependencies, outdated rye version, conflicting feature flags, broken WASM toolchain, missing target triples. Outputs actionable fixes. Builds on existing `rpg inspect` (goal 148).

223. ✅ **Implement `rpg upgrade` with automatic codemods** — Extend existing migration tooling (goal 94) with automatic codemod application during version upgrades. Breaking changes come with codemods that transform old API usage to new. Zero-manual-migration upgrades. Builds on existing `rpg upgrade` (goal 82).

224. ✅ **Implement `rpg profile` performance profiler** — CLI profiler that runs the app, collects performance data (render times, signal updates, bridge calls, memory), and outputs a flamegraph. No browser DevTools needed. Builds on existing performance monitoring (goal 93) and `rpg inspect` (goal 148).

225. ✅ **Implement `rpg bundle` size analyzer with tree map** — Visual tree map of Wasm binary contents. Shows which crates and functions contribute to bundle size. Drill-down from crate → module → function. Suggests size reduction opportunities. Builds on existing bundle size analyzer (goal 93).

226. ✅ **Implement `rpg init` interactive project wizard** — Interactive CLI that asks about project requirements (web? desktop? mobile? SSR? SSG?) and generates the optimal project configuration. Recommends feature flags, dependencies, and project structure. Builds on existing scaffolding (goal 88).

227. ✅ **Implement `rpg generate` code generation from OpenAPI** — Given an OpenAPI spec, generate typed API client, server actions, and form components for each endpoint. Type-safe end-to-end. Builds on existing server actions (goal 121) and forms (goal 73).

228. ✅ **Implement `rpg generate` from database schema** — Given a database schema (via SQLx or Diesel), generate CRUD components, forms, and API routes. Type-safe, with validation matching schema constraints. Builds on existing forms (goal 73) and server actions (goal 121).

229. ✅ **Implement VS Code extension with full LSP** — Beyond existing IDE support (goal 87), ship a full VS Code extension with: inline template syntax highlighting, prop autocomplete in templates, signal flow visualization, component preview on hover, error squiggles with fix suggestions. Builds on existing LSP (goal 87).

230. ✅ **Implement JetBrains plugin** — IntelliJ/RustRover plugin with the same features as the VS Code extension. Template syntax support, autocomplete, refactoring, preview. Builds on existing LSP (goal 87).

231. ✅ **Implement `rpg monorepo` workspace management** — First-class monorepo support. `rpg monorepo init` creates a workspace with shared dependencies, cross-component imports, and unified build. `rpg monorepo build` builds all packages. Builds on existing Cargo workspace structure (goal 9).

232. ✅ **Implement component library publishing** — `rpg publish` publishes a component library to the rye registry. Versioned, with auto-generated documentation, playground links, and migration guides. Consumers install via `rpg add @scope/component`. Builds on existing registry (goal 89).

233. ✅ **Implement `rpg theme` design token CLI** — Create, edit, and export theme files. `rpg theme create dark`, `rpg theme export --format=css`, `rpg theme diff light dark`. Visual theme editor in the browser. Builds on existing theming system (goal 90) and Figma import (goal 133).

234. ✅ **Implement `rpg docs` local documentation server** — Serve rye docs locally with live search, API reference, and interactive examples. Works offline. Auto-updates when rye version changes. Builds on existing documentation site (goal 95).

235. ✅ **Implement `rpg ci` CI/CD template generator** — Generates CI/CD configuration for popular platforms (GitHub Actions, GitLab CI, CircleCI). Includes build, test, lint, size check, deploy stages. Configurable per project type. Builds on existing CI/CD pipeline (goal 10).

---

## Phase 22: Advanced Testing & Quality (Goals 236–245)

> Extends testing (goals 85, 141–145) with advanced quality assurance.

236. ✅ **Implement integration testing harness** — `rye-testing` extension that spins up a full SSR server, makes real HTTP requests, and asserts on the rendered HTML. End-to-end testing without a browser. Builds on existing testing crate (goal 85) and SSR (goal 32).

237. ✅ **Implement E2E testing with Playwright integration** — `rpg test --e2e` runs Playwright tests against rye apps. Auto-generates test selectors from component names and test IDs. Screenshot comparison, network mocking, multi-browser. Builds on existing visual regression (goal 135).

238. ✅ **Implement component contract tests** — Auto-generated tests that verify a component's public API (props, events, slots) matches its documentation. Catches breaking changes before they ship. Builds on existing contract testing (goal 145).

239. ✅ **Implement performance regression testing** — `rpg test --perf` runs benchmarks and compares against baselines. Fails CI if render time, bundle size, or memory usage regresses beyond a threshold. Builds on existing performance monitoring (goal 93).

240. ✅ **Implement snapshot testing with semantic diffing** — Extend existing snapshot testing (goal 85) with semantic diffing — compare component structure (elements, props, children) not raw HTML. Diffs are meaningful, not whitespace-sensitive.

241. ✅ **Implement fuzz testing for template macro** — Generate random valid and invalid template syntax, verify the macro either compiles correctly or produces a valid error (never panics, never produces invalid code). Builds on existing property testing (goal 141).

242. ✅ **Implement accessibility tree snapshot testing** — `assert_a11y!(component, expected_tree)` compares the semantic accessibility tree, not the visual DOM. Ensures components are accessible at the structural level. Builds on existing a11y testing (goal 142).

243. ✅ **Implement cross-platform render equivalence tests** — Verify that the same component renders identically (structurally) on web, desktop, and mobile. Catches platform-specific rendering bugs. Builds on existing test renderer (goal 33) and native renderer (goal 56).

244. ✅ **Implement signal update ordering tests** — Verify that signal updates propagate in the correct order (topological sort of dependency graph). Catches diamond dependency bugs. Builds on existing property testing (goal 141).

245. ✅ **Implement automatic test generation from usage traces** — `rpg test --from-trace` takes a runtime trace (from goal 164) and generates a test that replays the same signal updates and verifies the same render output. Converts bug reproductions into regression tests automatically.

---

## Phase 23: Ecosystem & Interop Deep Cuts (Goals 246–250)

> Extends ecosystem integration (goals 131–140) with deeper interop.

246. ✅ **Implement React component wrapping** — `wrap_react_component!()` macro that imports a React component and makes it usable in rye templates. Props are mapped, events are translated, children are bridged. Enables incremental migration from React. Builds on existing JS interop (goal 132) and Web Components (goal 131).

247. ✅ **Implement Vue component wrapping** — `wrap_vue_component!()` macro for Vue single-file components. Mounts Vue component inside a rye wrapper, bridges props and events. Enables incremental migration from Vue. Builds on existing JS interop (goal 132).

248. ✅ **Implement Tailwind 4.0 engine integration** — Native Tailwind 4.0 (Oxide engine) integration. Faster than current built-in Tailwind (goal 72). Supports arbitrary values, container queries, 3D transforms. Zero-config. Builds on existing styling system (goal 72).

249. ✅ **Implement WebGPU compute shaders for data processing** — Beyond rendering (goal 111), use WebGPU compute shaders for data-parallel operations (image processing, ML inference, particle simulation). `use_compute_shader()` hook. Same API on web (WebGPU) and native (wgpu).

250. ✅ **Implement rye component Figma plugin** — Figma plugin that exports Figma designs directly to rye component code. Not just tokens (goal 133), but full component structure — layout, text, images, interactive states. Design-to-code in one click. Builds on existing Figma import (goal 133).

---

## Summary: How These Goals Map to Strategy

| Phase | Goals | Strategic Purpose |
|-------|-------|-------------------|
| 15: AI-Native Tooling & Agent Integration | 151–165 | Make rye the most AI-friendly framework — concrete tooling, not just principles |
| 16: Advanced Reactivity & State | 166–175 | Handle complex app state patterns — sagas, optimistic updates, cross-tab sync |
| 17: Rendering Deep Cuts | 176–185 | Next-gen rendering — offscreen, shadow DOM, static extraction, dual-pass |
| 18: Server & Full-Stack Deep Cuts | 186–195 | Production server — loaders, API routes, distributed SSR, DB integration |
| 19: Native & Mobile Deep Cuts | 196–210 | Full native API coverage — camera, biometrics, IAP, push, widgets |
| 20: Performance & Optimization Deep Cuts | 211–220 | WasmGC, AOT, layout caching, speculative preloading |
| 21: Developer Experience & Ecosystem | 221–235 | Next-gen DX — playground, doctor, codemods, monorepo, JetBrains |
| 22: Advanced Testing & Quality | 236–245 | E2E, perf regression, fuzz, a11y tree, cross-platform equivalence |
| 23: Ecosystem & Interop Deep Cuts | 246–250 | React/Vue wrapping, Tailwind 4, WebGPU compute, Figma plugin |

## Priority Order

### Do first (highest impact, lowest effort)
- 151 (`rpg explain` CLI) — core AI tooling, builds on existing error codes
- 154 (`rpg scaffold` AI templates) — builds on existing CLI
- 155 (`rpg test --generate`) — builds on existing testing
- 158 (AI error codes R800–R899) — builds on existing error system
- 166 (derived selectors) — builds on existing Store/Memo
- 174 (reactive URL state) — builds on existing router + signals

### Do second (high impact, medium effort)
- 153 (component discovery API) — new runtime API, builds on registry
- 156 (SPEC_FOR_AI auto-generation) — builds on existing spec
- 157 (MCP server) — wraps existing CLI commands
- 186 (server loaders) — builds on existing server actions
- 196 (native module system) — builds on existing platform detection
- 221 (`rpg playground`) — builds on existing storybook
- 229 (VS Code extension) — builds on existing LSP

### Do later (high impact, high effort)
- 165 (rye agent SDK) — depends on 151–164
- 190 (partial SSR re-rendering) — complex streaming + diffing
- 210 (native widgets) — platform-specific, complex
- 212 (Wasm GC) — depends on browser support
- 220 (selective Wasm AOT) — requires cranelift integration
- 246 (React wrapping) — complex interop layer
- 250 (Figma plugin) — full design-to-code pipeline
