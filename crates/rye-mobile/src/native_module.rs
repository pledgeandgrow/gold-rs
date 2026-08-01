//! Goal 196: Native module system.
//!
//! Platform-specific native modules that can be called from rye components.
//! `#[native_module]` generates bindings for iOS (Swift), Android (Kotlin), and desktop (C/C++).

use std::collections::HashMap;
use std::sync::Mutex;

/// The target platform for a native module binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativePlatform {
    /// iOS platform (Swift).
    Ios,
    /// Android platform (Kotlin).
    Android,
    /// Desktop platforms (C/C++).
    Desktop,
    /// Web platform (JavaScript).
    Web,
}

impl NativePlatform {
    /// Get the language name for this platform.
    pub fn language(&self) -> &'static str {
        match self {
            NativePlatform::Ios => "Swift",
            NativePlatform::Android => "Kotlin",
            NativePlatform::Desktop => "C/C++",
            NativePlatform::Web => "JavaScript",
        }
    }

    /// Get all platforms.
    pub fn all() -> &'static [NativePlatform] {
        &[
            NativePlatform::Ios,
            NativePlatform::Android,
            NativePlatform::Desktop,
            NativePlatform::Web,
        ]
    }
}

/// A native function signature — describes a function exposed from native code.
#[derive(Debug, Clone)]
pub struct NativeFunction {
    /// The function name.
    pub name: String,
    /// The parameter types.
    pub params: Vec<NativeType>,
    /// The return type.
    pub return_type: Option<NativeType>,
    /// Whether the function is async.
    pub is_async: bool,
    /// A doc comment for the function.
    pub doc: String,
}

/// A native type — used in function signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeType {
    /// 32-bit integer.
    Int32,
    /// 64-bit integer.
    Int64,
    /// 32-bit float.
    Float32,
    /// 64-bit float.
    Float64,
    /// Boolean.
    Bool,
    /// String.
    String,
    /// Byte array.
    Bytes,
    /// Void (no return).
    Void,
    /// Custom object type.
    Object(String),
}

impl NativeType {
    /// Get the type name as a string.
    pub fn type_name(&self) -> &str {
        match self {
            NativeType::Int32 => "i32",
            NativeType::Int64 => "i64",
            NativeType::Float32 => "f32",
            NativeType::Float64 => "f64",
            NativeType::Bool => "bool",
            NativeType::String => "String",
            NativeType::Bytes => "Vec<u8>",
            NativeType::Void => "()",
            NativeType::Object(name) => name,
        }
    }

    /// Get the Swift type equivalent.
    pub fn swift_type(&self) -> &str {
        match self {
            NativeType::Int32 => "Int32",
            NativeType::Int64 => "Int64",
            NativeType::Float32 => "Float",
            NativeType::Float64 => "Double",
            NativeType::Bool => "Bool",
            NativeType::String => "String",
            NativeType::Bytes => "Data",
            NativeType::Void => "Void",
            NativeType::Object(name) => name,
        }
    }

    /// Get the Kotlin type equivalent.
    pub fn kotlin_type(&self) -> &str {
        match self {
            NativeType::Int32 => "Int",
            NativeType::Int64 => "Long",
            NativeType::Float32 => "Float",
            NativeType::Float64 => "Double",
            NativeType::Bool => "Boolean",
            NativeType::String => "String",
            NativeType::Bytes => "ByteArray",
            NativeType::Void => "Unit",
            NativeType::Object(name) => name,
        }
    }
}

/// A native module — a collection of native functions for a specific platform.
#[derive(Debug, Clone)]
pub struct NativeModule {
    /// The module name.
    pub name: String,
    /// The platform this module targets.
    pub platform: NativePlatform,
    /// The functions exposed by this module.
    pub functions: Vec<NativeFunction>,
    /// The module's doc comment.
    pub doc: String,
}

impl NativeModule {
    /// Create a new native module.
    pub fn new(name: &str, platform: NativePlatform) -> Self {
        Self {
            name: name.to_string(),
            platform,
            functions: Vec::new(),
            doc: String::new(),
        }
    }

    /// Set the doc comment.
    pub fn with_doc(mut self, doc: &str) -> Self {
        self.doc = doc.to_string();
        self
    }

    /// Add a function to the module.
    pub fn add_function(&mut self, func: NativeFunction) {
        self.functions.push(func);
    }

    /// Get a function by name.
    pub fn get_function(&self, name: &str) -> Option<&NativeFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Generate the Rust binding code for this module.
    pub fn generate_rust_bindings(&self) -> String {
        let mut code = format!(
            "// Native module: {} ({})\n// {}\n\n",
            self.name,
            self.platform.language(),
            self.doc
        );

        for func in &self.functions {
            let params: Vec<String> = func
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("arg_{}: {}", i, t.type_name()))
                .collect();

            let ret = func
                .return_type
                .as_ref()
                .map(|t| t.type_name())
                .unwrap_or("()");

            if func.is_async {
                code.push_str(&format!(
                    "pub async fn {}({}) -> {} {{\n    // FFI call to {} native code\n    unimplemented!()\n}}\n\n",
                    func.name, params.join(", "), ret, self.platform.language()
                ));
            } else {
                code.push_str(&format!(
                    "pub fn {}({}) -> {} {{\n    // FFI call to {} native code\n    unimplemented!()\n}}\n\n",
                    func.name, params.join(", "), ret, self.platform.language()
                ));
            }
        }

        code
    }

    /// Generate Swift bindings for this module.
    pub fn generate_swift_bindings(&self) -> String {
        let mut code = format!(
            "// Swift bindings for rye native module: {}\n// {}\n\nimport Foundation\n\n",
            self.name, self.doc
        );

        for func in &self.functions {
            let params: Vec<String> = func
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("arg{}: {}", i, t.swift_type()))
                .collect();

            let ret = func
                .return_type
                .as_ref()
                .map(|t| t.swift_type())
                .unwrap_or("Void");

            code.push_str(&format!(
                "func {}({}) -> {} {{\n    // Call rye native bridge\n}}\n\n",
                func.name, params.join(", "), ret
            ));
        }

        code
    }

    /// Generate Kotlin bindings for this module.
    pub fn generate_kotlin_bindings(&self) -> String {
        let mut code = format!(
            "// Kotlin bindings for rye native module: {}\n// {}\n\npackage rye.native\n\n",
            self.name, self.doc
        );

        for func in &self.functions {
            let params: Vec<String> = func
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("arg{}: {}", i, t.kotlin_type()))
                .collect();

            let ret = func
                .return_type
                .as_ref()
                .map(|t| t.kotlin_type())
                .unwrap_or("Unit");

            if func.is_async {
                code.push_str(&format!(
                    "suspend fun {}({}): {} {{\n    // Call rye native bridge via JNI\n}}\n\n",
                    func.name, params.join(", "), ret
                ));
            } else {
                code.push_str(&format!(
                    "fun {}({}): {} {{\n    // Call rye native bridge via JNI\n}}\n\n",
                    func.name, params.join(", "), ret
                ));
            }
        }

        code
    }
}

/// A builder for native modules.
pub struct NativeModuleBuilder {
    module: NativeModule,
}

impl NativeModuleBuilder {
    /// Create a new builder.
    pub fn new(name: &str, platform: NativePlatform) -> Self {
        Self {
            module: NativeModule::new(name, platform),
        }
    }

    /// Set the doc comment.
    pub fn doc(mut self, doc: &str) -> Self {
        self.module.doc = doc.to_string();
        self
    }

    /// Add a function.
    pub fn function(mut self, name: &str, params: Vec<NativeType>, return_type: Option<NativeType>, is_async: bool) -> Self {
        self.module.functions.push(NativeFunction {
            name: name.to_string(),
            params,
            return_type,
            is_async,
            doc: String::new(),
        });
        self
    }

    /// Build the module.
    pub fn build(self) -> NativeModule {
        self.module
    }
}

/// The native module registry — manages all registered native modules.
pub struct NativeModuleRegistry {
    modules: Mutex<HashMap<String, NativeModule>>,
}

impl NativeModuleRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
        }
    }

    /// Register a native module.
    pub fn register(&self, module: NativeModule) {
        self.modules.lock().unwrap().insert(module.name.clone(), module);
    }

    /// Get a module by name.
    pub fn get(&self, name: &str) -> Option<NativeModule> {
        self.modules.lock().unwrap().get(name).cloned()
    }

    /// Get all module names.
    pub fn module_names(&self) -> Vec<String> {
        self.modules.lock().unwrap().keys().cloned().collect()
    }

    /// Get the number of registered modules.
    pub fn len(&self) -> usize {
        self.modules.lock().unwrap().len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.modules.lock().unwrap().is_empty()
    }

    /// Get all modules for a specific platform.
    pub fn modules_for_platform(&self, platform: NativePlatform) -> Vec<NativeModule> {
        self.modules
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.platform == platform)
            .cloned()
            .collect()
    }

    /// Generate all Rust bindings.
    pub fn generate_all_rust_bindings(&self) -> String {
        self.modules
            .lock()
            .unwrap()
            .values()
            .map(|m| m.generate_rust_bindings())
            .collect::<Vec<_>>()
            .join("\n---\n\n")
    }

    /// Remove a module by name.
    pub fn remove(&self, name: &str) -> bool {
        self.modules.lock().unwrap().remove(name).is_some()
    }
}

impl Default for NativeModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_platform_language() {
        assert_eq!(NativePlatform::Ios.language(), "Swift");
        assert_eq!(NativePlatform::Android.language(), "Kotlin");
        assert_eq!(NativePlatform::Desktop.language(), "C/C++");
        assert_eq!(NativePlatform::Web.language(), "JavaScript");
    }

    #[test]
    fn test_native_platform_all() {
        assert_eq!(NativePlatform::all().len(), 4);
    }

    #[test]
    fn test_native_type_type_name() {
        assert_eq!(NativeType::Int32.type_name(), "i32");
        assert_eq!(NativeType::Bool.type_name(), "bool");
        assert_eq!(NativeType::String.type_name(), "String");
        assert_eq!(NativeType::Void.type_name(), "()");
    }

    #[test]
    fn test_native_type_swift() {
        assert_eq!(NativeType::Int32.swift_type(), "Int32");
        assert_eq!(NativeType::Float32.swift_type(), "Float");
        assert_eq!(NativeType::Bytes.swift_type(), "Data");
    }

    #[test]
    fn test_native_type_kotlin() {
        assert_eq!(NativeType::Int32.kotlin_type(), "Int");
        assert_eq!(NativeType::Int64.kotlin_type(), "Long");
        assert_eq!(NativeType::Bytes.kotlin_type(), "ByteArray");
    }

    #[test]
    fn test_native_module_new() {
        let module = NativeModule::new("camera", NativePlatform::Ios);
        assert_eq!(module.name, "camera");
        assert_eq!(module.platform, NativePlatform::Ios);
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_native_module_add_function() {
        let mut module = NativeModule::new("camera", NativePlatform::Ios);
        module.add_function(NativeFunction {
            name: "take_photo".to_string(),
            params: vec![NativeType::String],
            return_type: Some(NativeType::Bytes),
            is_async: true,
            doc: "Take a photo".to_string(),
        });
        assert_eq!(module.functions.len(), 1);
        assert!(module.get_function("take_photo").is_some());
        assert!(module.get_function("nonexistent").is_none());
    }

    #[test]
    fn test_native_module_with_doc() {
        let module = NativeModule::new("camera", NativePlatform::Ios)
            .with_doc("Camera access module");
        assert_eq!(module.doc, "Camera access module");
    }

    #[test]
    fn test_native_module_builder() {
        let module = NativeModuleBuilder::new("haptics", NativePlatform::Android)
            .doc("Haptic feedback module")
            .function(
                "vibrate",
                vec![NativeType::Int32],
                None,
                false,
            )
            .build();

        assert_eq!(module.name, "haptics");
        assert_eq!(module.platform, NativePlatform::Android);
        assert_eq!(module.doc, "Haptic feedback module");
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "vibrate");
    }

    #[test]
    fn test_generate_rust_bindings() {
        let mut module = NativeModule::new("test", NativePlatform::Desktop);
        module.add_function(NativeFunction {
            name: "get_version".to_string(),
            params: vec![],
            return_type: Some(NativeType::String),
            is_async: false,
            doc: "Get version".to_string(),
        });

        let code = module.generate_rust_bindings();
        assert!(code.contains("pub fn get_version"));
        assert!(code.contains("-> String"));
    }

    #[test]
    fn test_generate_rust_bindings_async() {
        let mut module = NativeModule::new("test", NativePlatform::Ios);
        module.add_function(NativeFunction {
            name: "fetch_data".to_string(),
            params: vec![NativeType::String],
            return_type: Some(NativeType::Bytes),
            is_async: true,
            doc: "Fetch data".to_string(),
        });

        let code = module.generate_rust_bindings();
        assert!(code.contains("pub async fn fetch_data"));
    }

    #[test]
    fn test_generate_swift_bindings() {
        let mut module = NativeModule::new("test", NativePlatform::Ios);
        module.add_function(NativeFunction {
            name: "get_name".to_string(),
            params: vec![],
            return_type: Some(NativeType::String),
            is_async: false,
            doc: "".to_string(),
        });

        let code = module.generate_swift_bindings();
        assert!(code.contains("import Foundation"));
        assert!(code.contains("func get_name"));
    }

    #[test]
    fn test_generate_kotlin_bindings() {
        let mut module = NativeModule::new("test", NativePlatform::Android);
        module.add_function(NativeFunction {
            name: "get_name".to_string(),
            params: vec![],
            return_type: Some(NativeType::String),
            is_async: false,
            doc: "".to_string(),
        });

        let code = module.generate_kotlin_bindings();
        assert!(code.contains("package rye.native"));
        assert!(code.contains("fun get_name"));
    }

    #[test]
    fn test_generate_kotlin_bindings_async() {
        let mut module = NativeModule::new("test", NativePlatform::Android);
        module.add_function(NativeFunction {
            name: "fetch".to_string(),
            params: vec![],
            return_type: Some(NativeType::Bytes),
            is_async: true,
            doc: "".to_string(),
        });

        let code = module.generate_kotlin_bindings();
        assert!(code.contains("suspend fun fetch"));
    }

    #[test]
    fn test_registry_register_get() {
        let registry = NativeModuleRegistry::new();
        let module = NativeModule::new("camera", NativePlatform::Ios);
        registry.register(module);

        assert!(registry.get("camera").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_module_names() {
        let registry = NativeModuleRegistry::new();
        registry.register(NativeModule::new("a", NativePlatform::Ios));
        registry.register(NativeModule::new("b", NativePlatform::Android));

        let names = registry.module_names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_registry_len() {
        let registry = NativeModuleRegistry::new();
        assert!(registry.is_empty());
        registry.register(NativeModule::new("a", NativePlatform::Ios));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_modules_for_platform() {
        let registry = NativeModuleRegistry::new();
        registry.register(NativeModule::new("ios_mod", NativePlatform::Ios));
        registry.register(NativeModule::new("android_mod", NativePlatform::Android));
        registry.register(NativeModule::new("ios_mod2", NativePlatform::Ios));

        let ios_modules = registry.modules_for_platform(NativePlatform::Ios);
        assert_eq!(ios_modules.len(), 2);
    }

    #[test]
    fn test_registry_generate_all_rust() {
        let registry = NativeModuleRegistry::new();
        let mut module = NativeModule::new("test", NativePlatform::Desktop);
        module.add_function(NativeFunction {
            name: "hello".to_string(),
            params: vec![],
            return_type: Some(NativeType::String),
            is_async: false,
            doc: "".to_string(),
        });
        registry.register(module);

        let code = registry.generate_all_rust_bindings();
        assert!(code.contains("pub fn hello"));
    }

    #[test]
    fn test_registry_remove() {
        let registry = NativeModuleRegistry::new();
        registry.register(NativeModule::new("temp", NativePlatform::Web));
        assert!(registry.remove("temp"));
        assert!(registry.get("temp").is_none());
        assert!(!registry.remove("nonexistent"));
    }
}
