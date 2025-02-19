use bevy::app::{plugin_group, Plugin};

plugin_group! {
    /// This plugin group will add all the default plugins for a *Bevy* application:
    pub struct DefaultPlugins {
        bevy::app:::PanicHandlerPlugin,
        bevy::log:::LogPlugin,
        bevy::core:::TaskPoolPlugin,
        bevy::core:::TypeRegistrationPlugin,
        bevy::core:::FrameCountPlugin,
        bevy::time:::TimePlugin,
        bevy::transform:::TransformPlugin,
        bevy::hierarchy:::HierarchyPlugin,
        bevy::diagnostic:::DiagnosticsPlugin,
        bevy::input:::InputPlugin,
        #[custom(cfg(not(feature = "bevy::window")))]
        bevy::app:::ScheduleRunnerPlugin,
        #[cfg(feature = "bevy::window")]
        bevy::window:::WindowPlugin,
        #[cfg(feature = "bevy::window")]
        bevy::a11y:::AccessibilityPlugin,
        #[custom(cfg(not(target_arch = "wasm32")))]
        bevy::app:::TerminalCtrlCHandlerPlugin,
        #[cfg(feature = "bevy::asset")]
        bevy::asset:::AssetPlugin,
        #[cfg(feature = "bevy::scene")]
        bevy::scene:::ScenePlugin,
        #[cfg(feature = "bevy::winit")]
        bevy::winit:::WinitPlugin,
        #[cfg(feature = "bevy::render")]
        bevy::render:::RenderPlugin,
        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats.
        #[cfg(feature = "bevy::render")]
        bevy::render::texture:::ImagePlugin,
        #[cfg(feature = "bevy::render")]
        #[custom(cfg(all(not(target_arch = "wasm32"), feature = "multi_threaded")))]
        bevy::render::pipelined_rendering:::PipelinedRenderingPlugin,
        #[cfg(feature = "bevy::core_pipeline")]
        bevy::core_pipeline:::CorePipelinePlugin,
        #[cfg(feature = "bevy::sprite")]
        bevy::sprite:::SpritePlugin,
        #[cfg(feature = "bevy::text")]
        bevy::text:::TextPlugin,
        #[cfg(feature = "bevy::ui")]
        bevy::ui:::UiPlugin,
        #[cfg(feature = "bevy::pbr")]
        bevy::pbr:::PbrPlugin,
        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats.
        #[cfg(feature = "bevy::gltf")]
        bevy::gltf:::GltfPlugin,
        #[cfg(feature = "bevy::audio")]
        bevy::audio:::AudioPlugin,
        #[cfg(feature = "bevy::gilrs")]
        bevy::gilrs:::GilrsPlugin,
        #[cfg(feature = "bevy::animation")]
        bevy::animation:::AnimationPlugin,
        #[cfg(feature = "bevy::gizmos")]
        bevy::gizmos:::GizmoPlugin,
        #[cfg(feature = "bevy::state")]
        bevy::state::app:::StatesPlugin,
        #[cfg(feature = "bevy::dev_tools")]
        bevy::dev_tools:::DevToolsPlugin,
        #[cfg(feature = "bevy::ci_testing")]
        bevy::dev_tools::ci_testing:::CiTestingPlugin,
        #[plugin_group]
        #[cfg(feature = "bevy::picking")]
        bevy::picking:::DefaultPickingPlugins,
        #[doc(hidden)]
        :IgnoreAmbiguitiesPlugin,
    }
    /// [`DefaultPlugins`] obeys *Cargo* *feature* flags. Users may exert control over this plugin group
    /// by disabling `default-features` in their `Cargo.toml` and enabling only those features
    /// that they wish to use.
    ///
    /// [`DefaultPlugins`] contains all the plugins typically required to build
    /// a *Bevy* application which includes a *window* and presentation components.
    /// For the absolute minimum number of plugins needed to run a Bevy application, see [`MinimalPlugins`].
}

#[derive(Default)]
struct IgnoreAmbiguitiesPlugin;

impl Plugin for IgnoreAmbiguitiesPlugin {
    #[allow(unused_variables)] // Variables are used depending on enabled features
    fn build(&self, app: &mut bevy::app::App) {
        // bevy::ui owns the Transform and cannot be animated
        #[cfg(all(feature = "bevy::animation", feature = "bevy::ui"))]
        if app.is_plugin_added::<bevy::animation::AnimationPlugin>()
            && app.is_plugin_added::<bevy::ui::UiPlugin>()
        {
            app.ignore_ambiguity(
                bevy::app::PostUpdate,
                bevy::animation::advance_animations,
                bevy::ui::ui_layout_system,
            );
            app.ignore_ambiguity(
                bevy::app::PostUpdate,
                bevy::animation::animate_targets,
                bevy::ui::ui_layout_system,
            );
        }
    }
}

plugin_group! {
    /// This plugin group will add the minimal plugins for a *Bevy* application:
    pub struct MinimalPlugins {
        bevy::core:::TaskPoolPlugin,
        bevy::core:::TypeRegistrationPlugin,
        bevy::core:::FrameCountPlugin,
        bevy::time:::TimePlugin,
        bevy::app:::ScheduleRunnerPlugin,
        #[cfg(feature = "bevy::ci_testing")]
        bevy::dev_tools::ci_testing:::CiTestingPlugin,
    }
    /// This plugin group represents the absolute minimum, bare-bones, bevy application.
    /// Use this if you want to have absolute control over the plugins used.
    ///
    /// It includes a [schedule runner (`ScheduleRunnerPlugin`)](crate::app::ScheduleRunnerPlugin)
    /// to provide functionality that would otherwise be driven by a windowed application's
    /// *event loop* or *message loop*.
}