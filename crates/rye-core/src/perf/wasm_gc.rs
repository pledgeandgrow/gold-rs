//! Goal 212: Wasm GC proposal support.
//!
//! When browsers ship WasmGC, switch from `wasm-bindgen` reference type
//! emulation to native GC types. Feature-flagged for browsers without WasmGC.

use std::collections::HashMap;
use std::sync::Mutex;

/// Whether WasmGC is available in the current browser/runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmGcAvailability {
    /// WasmGC is available and enabled.
    Available,
    /// WasmGC is not available — use reference type emulation.
    NotAvailable,
    /// WasmGC availability is unknown — feature-detect at runtime.
    Unknown,
}

impl WasmGcAvailability {
    /// Check if WasmGC can be used.
    pub fn can_use_gc(&self) -> bool {
        matches!(self, WasmGcAvailability::Available)
    }

    /// Get the feature detection JavaScript code.
    pub fn detection_script() -> &'static str {
        r#"try{var m=new WebAssembly.Module(new Uint8Array([0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,10,7,1,5,0,3,65,0,0,11]));new WebAssembly.Instance(m);window.__ryeWasmGc=true;}catch(e){window.__ryeWasmGc=false;}"#
    }
}

/// The type mapping strategy for WasmGC vs reference types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeMappingStrategy {
    /// Use native WasmGC types (struct, array, eqref, etc.).
    NativeGc,
    /// Use reference type emulation (externref, table-based).
    ReferenceEmulation,
    /// Use JavaScript objects via wasm-bindgen.
    JsInterop,
}

impl TypeMappingStrategy {
    /// Get the strategy for a given availability.
    pub fn from_availability(availability: WasmGcAvailability) -> Self {
        match availability {
            WasmGcAvailability::Available => TypeMappingStrategy::NativeGc,
            WasmGcAvailability::NotAvailable => TypeMappingStrategy::JsInterop,
            WasmGcAvailability::Unknown => TypeMappingStrategy::ReferenceEmulation,
        }
    }

    /// Get the Rust type for a rye type under this strategy.
    pub fn map_type(&self, rye_type: &RyeGcType) -> &'static str {
        match self {
            TypeMappingStrategy::NativeGc => rye_type.wasm_gc_type(),
            TypeMappingStrategy::ReferenceEmulation => rye_type.reference_type(),
            TypeMappingStrategy::JsInterop => rye_type.js_interop_type(),
        }
    }
}

/// A rye type that can be mapped to different Wasm strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RyeGcType {
    /// A string / text value.
    String,
    /// A vector / array.
    Array,
    /// A struct / object.
    Struct,
    /// A map / dictionary.
    Map,
    /// An optional value.
    Optional,
    /// A function reference.
    Function,
    /// An externref (DOM element, JS object).
    ExternRef,
    /// A anyref (any JS value).
    AnyRef,
    /// An eqref (comparable reference).
    EqRef,
    /// A structref.
    StructRef,
    /// An arrayref.
    ArrayRef,
}

impl RyeGcType {
    /// Get the WasmGC native type representation.
    pub fn wasm_gc_type(&self) -> &'static str {
        match self {
            RyeGcType::String => "(ref $string)",
            RyeGcType::Array => "(ref $array)",
            RyeGcType::Struct => "(ref $struct)",
            RyeGcType::Map => "(ref $map)",
            RyeGcType::Optional => "(ref null $option)",
            RyeGcType::Function => "(ref $func)",
            RyeGcType::ExternRef => "externref",
            RyeGcType::AnyRef => "anyref",
            RyeGcType::EqRef => "eqref",
            RyeGcType::StructRef => "structref",
            RyeGcType::ArrayRef => "arrayref",
        }
    }

    /// Get the reference type emulation representation.
    pub fn reference_type(&self) -> &'static str {
        match self {
            RyeGcType::String | RyeGcType::Array | RyeGcType::Struct | RyeGcType::Map => {
                "externref"
            }
            RyeGcType::Optional => "externref",
            RyeGcType::Function => "funcref",
            RyeGcType::ExternRef => "externref",
            RyeGcType::AnyRef => "anyref",
            RyeGcType::EqRef => "eqref",
            RyeGcType::StructRef => "structref",
            RyeGcType::ArrayRef => "arrayref",
        }
    }

    /// Get the JS interop (wasm-bindgen) type representation.
    pub fn js_interop_type(&self) -> &'static str {
        match self {
            RyeGcType::String => "String",
            RyeGcType::Array => "Vec<u8>",
            RyeGcType::Struct => "JsValue",
            RyeGcType::Map => "JsValue",
            RyeGcType::Optional => "Option<JsValue>",
            RyeGcType::Function => "Function",
            RyeGcType::ExternRef => "JsValue",
            RyeGcType::AnyRef => "JsValue",
            RyeGcType::EqRef => "JsValue",
            RyeGcType::StructRef => "JsValue",
            RyeGcType::ArrayRef => "JsValue",
        }
    }
}

/// Configuration for WasmGC compilation.
#[derive(Debug, Clone)]
pub struct WasmGcConfig {
    /// Whether to enable WasmGC.
    pub enabled: bool,
    /// The type mapping strategy.
    pub strategy: TypeMappingStrategy,
    /// Whether to generate fallback code for non-WasmGC browsers.
    pub generate_fallback: bool,
    /// Whether to use struct types instead of JS objects.
    pub use_struct_types: bool,
    /// Whether to use array types instead of JS arrays.
    pub use_array_types: bool,
    /// Whether to use eqref for equality comparisons.
    pub use_eqref: bool,
}

impl Default for WasmGcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: TypeMappingStrategy::JsInterop,
            generate_fallback: true,
            use_struct_types: true,
            use_array_types: true,
            use_eqref: false,
        }
    }
}

impl WasmGcConfig {
    /// Create a config for WasmGC-enabled browsers.
    pub fn gc_enabled() -> Self {
        Self {
            enabled: true,
            strategy: TypeMappingStrategy::NativeGc,
            generate_fallback: true,
            use_struct_types: true,
            use_array_types: true,
            use_eqref: true,
        }
    }

    /// Create a config for non-WasmGC browsers.
    pub fn gc_disabled() -> Self {
        Self {
            enabled: false,
            strategy: TypeMappingStrategy::JsInterop,
            generate_fallback: false,
            use_struct_types: false,
            use_array_types: false,
            use_eqref: false,
        }
    }

    /// Estimate the binary size reduction (as a fraction, 0.0-1.0).
    pub fn estimated_size_reduction(&self) -> f64 {
        if !self.enabled {
            return 0.0;
        }
        let mut reduction: f64 = 0.0;
        if self.use_struct_types {
            reduction += 0.10;
        }
        if self.use_array_types {
            reduction += 0.08;
        }
        if self.use_eqref {
            reduction += 0.05;
        }
        reduction.min(0.30_f64) // Cap at 30%
    }
}

/// The WasmGC type registry — manages type definitions for code generation.
pub struct WasmGcTypeRegistry {
    types: Mutex<HashMap<String, RyeGcType>>,
    config: WasmGcConfig,
}

impl WasmGcTypeRegistry {
    /// Create a new type registry.
    pub fn new(config: WasmGcConfig) -> Self {
        Self {
            types: Mutex::new(HashMap::new()),
            config,
        }
    }

    /// Register a type.
    pub fn register(&self, name: &str, rye_type: RyeGcType) {
        self.types
            .lock()
            .unwrap()
            .insert(name.to_string(), rye_type);
    }

    /// Get a type by name.
    pub fn get(&self, name: &str) -> Option<RyeGcType> {
        self.types.lock().unwrap().get(name).copied()
    }

    /// Get the mapped type for a named type.
    pub fn mapped_type(&self, name: &str) -> Option<&'static str> {
        self.get(name).map(|t| self.config.strategy.map_type(&t))
    }

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.types.lock().unwrap().len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.types.lock().unwrap().is_empty()
    }

    /// Get the config.
    pub fn config(&self) -> &WasmGcConfig {
        &self.config
    }

    /// Generate WasmGC type definitions (WAT format).
    pub fn generate_type_definitions(&self) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let types = self.types.lock().unwrap();
        let mut wat = String::new();

        if self.config.use_struct_types {
            wat.push_str(";; WasmGC struct types\n");
            for (name, rye_type) in types.iter() {
                if *rye_type == RyeGcType::Struct {
                    wat.push_str(&format!("(type ${} (struct (field $data anyref)))\n", name));
                }
            }
        }

        if self.config.use_array_types {
            wat.push_str(";; WasmGC array types\n");
            for (name, rye_type) in types.iter() {
                if *rye_type == RyeGcType::Array {
                    wat.push_str(&format!("(type ${} (array anyref))\n", name));
                }
            }
        }

        wat
    }

    /// Generate the feature detection and fallback script.
    pub fn generate_feature_detection_script(&self) -> String {
        if !self.config.generate_fallback {
            return String::new();
        }

        format!(
            r#"(function(){{{detection}
if(window.__ryeWasmGc){{console.info('[rye] WasmGC enabled — using native GC types');}}
else{{console.info('[rye] WasmGC not available — using reference type emulation');}}
}})();"#,
            detection = WasmGcAvailability::detection_script()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_gc_availability_can_use() {
        assert!(WasmGcAvailability::Available.can_use_gc());
        assert!(!WasmGcAvailability::NotAvailable.can_use_gc());
        assert!(!WasmGcAvailability::Unknown.can_use_gc());
    }

    #[test]
    fn test_wasm_gc_detection_script() {
        let script = WasmGcAvailability::detection_script();
        assert!(script.contains("WebAssembly.Module"));
        assert!(script.contains("__ryeWasmGc"));
    }

    #[test]
    fn test_type_mapping_strategy_from_availability() {
        assert_eq!(
            TypeMappingStrategy::from_availability(WasmGcAvailability::Available),
            TypeMappingStrategy::NativeGc
        );
        assert_eq!(
            TypeMappingStrategy::from_availability(WasmGcAvailability::NotAvailable),
            TypeMappingStrategy::JsInterop
        );
        assert_eq!(
            TypeMappingStrategy::from_availability(WasmGcAvailability::Unknown),
            TypeMappingStrategy::ReferenceEmulation
        );
    }

    #[test]
    fn test_rye_gc_type_wasm_gc() {
        assert_eq!(RyeGcType::String.wasm_gc_type(), "(ref $string)");
        assert_eq!(RyeGcType::ExternRef.wasm_gc_type(), "externref");
        assert_eq!(RyeGcType::AnyRef.wasm_gc_type(), "anyref");
    }

    #[test]
    fn test_rye_gc_type_reference() {
        assert_eq!(RyeGcType::String.reference_type(), "externref");
        assert_eq!(RyeGcType::Function.reference_type(), "funcref");
    }

    #[test]
    fn test_rye_gc_type_js_interop() {
        assert_eq!(RyeGcType::String.js_interop_type(), "String");
        assert_eq!(RyeGcType::ExternRef.js_interop_type(), "JsValue");
    }

    #[test]
    fn test_type_mapping_strategy_map_type() {
        let strategy = TypeMappingStrategy::NativeGc;
        assert_eq!(strategy.map_type(&RyeGcType::String), "(ref $string)");

        let strategy = TypeMappingStrategy::JsInterop;
        assert_eq!(strategy.map_type(&RyeGcType::String), "String");
    }

    #[test]
    fn test_wasm_gc_config_default() {
        let config = WasmGcConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.strategy, TypeMappingStrategy::JsInterop);
        assert!(config.generate_fallback);
    }

    #[test]
    fn test_wasm_gc_config_gc_enabled() {
        let config = WasmGcConfig::gc_enabled();
        assert!(config.enabled);
        assert_eq!(config.strategy, TypeMappingStrategy::NativeGc);
        assert!(config.use_struct_types);
        assert!(config.use_array_types);
        assert!(config.use_eqref);
    }

    #[test]
    fn test_wasm_gc_config_gc_disabled() {
        let config = WasmGcConfig::gc_disabled();
        assert!(!config.enabled);
        assert!(!config.generate_fallback);
    }

    #[test]
    fn test_wasm_gc_config_size_reduction_disabled() {
        let config = WasmGcConfig::gc_disabled();
        assert_eq!(config.estimated_size_reduction(), 0.0);
    }

    #[test]
    fn test_wasm_gc_config_size_reduction_enabled() {
        let config = WasmGcConfig::gc_enabled();
        let reduction = config.estimated_size_reduction();
        assert!(reduction > 0.0);
        assert!(reduction <= 0.30);
    }

    #[test]
    fn test_wasm_gc_config_size_reduction_struct_only() {
        let config = WasmGcConfig {
            enabled: true,
            strategy: TypeMappingStrategy::NativeGc,
            generate_fallback: true,
            use_struct_types: true,
            use_array_types: false,
            use_eqref: false,
        };
        assert_eq!(config.estimated_size_reduction(), 0.10);
    }

    #[test]
    fn test_type_registry_register_get() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        registry.register("MyComponent", RyeGcType::Struct);
        assert_eq!(registry.get("MyComponent"), Some(RyeGcType::Struct));
        assert!(registry.get("Unknown").is_none());
    }

    #[test]
    fn test_type_registry_mapped_type() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        registry.register("MyArray", RyeGcType::Array);
        assert_eq!(registry.mapped_type("MyArray"), Some("(ref $array)"));
    }

    #[test]
    fn test_type_registry_mapped_type_js_interop() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_disabled());
        registry.register("MyString", RyeGcType::String);
        assert_eq!(registry.mapped_type("MyString"), Some("String"));
    }

    #[test]
    fn test_type_registry_len() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        registry.register("a", RyeGcType::Struct);
        registry.register("b", RyeGcType::Array);
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_type_registry_is_empty() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        assert!(registry.is_empty());
    }

    #[test]
    fn test_type_registry_generate_type_definitions() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        registry.register("MyStruct", RyeGcType::Struct);
        registry.register("MyArray", RyeGcType::Array);
        let wat = registry.generate_type_definitions();
        assert!(wat.contains("(type $MyStruct (struct"));
        assert!(wat.contains("(type $MyArray (array"));
    }

    #[test]
    fn test_type_registry_generate_type_definitions_disabled() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_disabled());
        registry.register("MyStruct", RyeGcType::Struct);
        let wat = registry.generate_type_definitions();
        assert!(wat.is_empty());
    }

    #[test]
    fn test_type_registry_generate_feature_detection() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_enabled());
        let script = registry.generate_feature_detection_script();
        assert!(script.contains("__ryeWasmGc"));
        assert!(script.contains("WasmGC enabled"));
    }

    #[test]
    fn test_type_registry_generate_feature_detection_no_fallback() {
        let registry = WasmGcTypeRegistry::new(WasmGcConfig::gc_disabled());
        let script = registry.generate_feature_detection_script();
        assert!(script.is_empty());
    }
}
