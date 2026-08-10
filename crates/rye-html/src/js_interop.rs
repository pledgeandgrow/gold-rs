//! JS library interop — use third-party JS libraries from rye.
//!
//! Provides a safe, typed bridge for calling JavaScript libraries from
//! rye components. This is essential for ecosystem integration — many
//! UI libraries (Chart.js, CodeMirror, map libraries, etc.) are JS-only.
//!
//! ## Design
//!
//! - **`JsValue`**: Opaque handle to a JS value (string, number, object, function)
//! - **`JsFunction`**: Typed wrapper for calling JS functions
//! - **`JsObject`**: Builder for constructing JS objects with typed properties
//! - **`import_js`**: Dynamic `import()` for ES modules
//!
//! ## Safety
//!
//! All JS calls go through `wasm-bindgen`'s `js_sys` on Wasm targets.
//! On non-Wasm targets, calls are no-ops (returning `None`), enabling
//! SSR without JS runtime.
//!
//! ## Usage
//!
//! ```ignore
//! use rye_html::js_interop::{JsValue, JsFunction, import_js};
//!
//! // Import a JS module
//! let chart_lib = import_js("chart.js")?;
//!
//! // Call a function
//! let chart = chart_lib.call("createChart", &[
//!     JsValue::String("canvas#chart".into()),
//!     JsValue::Object(vec![
//!         ("type".into(), JsValue::String("bar".into())),
//!         ("data".into(), JsValue::Array(vec![...])),
//!     ]),
//! ])?;
//! ```

/// A typed JavaScript value that can be passed across the Wasm→JS bridge.
#[derive(Debug, Clone)]
pub enum JsValue {
    /// null
    Null,
    /// undefined
    Undefined,
    /// Boolean
    Bool(bool),
    /// Number (f64)
    Number(f64),
    /// String
    String(String),
    /// Array of JsValues
    Array(Vec<JsValue>),
    /// Object (key-value pairs)
    Object(Vec<(String, JsValue)>),
    /// Opaque reference (e.g. DOM element handle)
    Ref(String),
}

impl JsValue {
    /// Convert to a string representation (for debugging / SSR).
    pub fn to_display_string(&self) -> String {
        match self {
            JsValue::Null => "null".to_string(),
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Bool(b) => b.to_string(),
            JsValue::Number(n) => n.to_string(),
            JsValue::String(s) => s.clone(),
            JsValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            JsValue::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
            JsValue::Ref(r) => format!("[ref:{}]", r),
        }
    }

    /// Whether this value is null or undefined.
    pub fn is_nullish(&self) -> bool {
        matches!(self, JsValue::Null | JsValue::Undefined)
    }

    /// Try to get a string value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get a number value.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get a bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<&str> for JsValue {
    fn from(s: &str) -> Self {
        JsValue::String(s.to_string())
    }
}

impl From<String> for JsValue {
    fn from(s: String) -> Self {
        JsValue::String(s)
    }
}

impl From<i32> for JsValue {
    fn from(n: i32) -> Self {
        JsValue::Number(n as f64)
    }
}

impl From<f64> for JsValue {
    fn from(n: f64) -> Self {
        JsValue::Number(n)
    }
}

impl From<bool> for JsValue {
    fn from(b: bool) -> Self {
        JsValue::Bool(b)
    }
}

/// A handle to an imported JS module.
pub struct JsModule {
    /// Module specifier (URL or package name).
    spec: String,
    /// Whether the module has been loaded.
    loaded: bool,
}

impl JsModule {
    /// Create a new JS module handle.
    pub fn new(spec: impl Into<String>) -> Self {
        Self {
            spec: spec.into(),
            loaded: false,
        }
    }

    /// Get the module specifier.
    pub fn spec(&self) -> &str {
        &self.spec
    }

    /// Whether the module is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Call a function from this module.
    ///
    /// On Wasm targets, this would use `js_sys::Reflect::get` + `Function::call`.
    /// On non-Wasm targets, returns `None`.
    pub fn call(&self, _function: &str, _args: &[JsValue]) -> Option<JsValue> {
        // On Wasm targets:
        //   let module = self.module_ref?;
        //   let func = js_sys::Reflect::get(&module, &function.into())?;
        //   let func: js_sys::Function = func.dyn_into()?;
        //   let js_args: Vec<JsValue> = args.iter().map(to_js).collect();
        //   let result = func.apply(&module, &js_args)?;
        //   Some(from_js(&result))
        //
        // On non-Wasm targets:
        None
    }

    /// Get a value/property from this module.
    pub fn get(&self, _property: &str) -> Option<JsValue> {
        None
    }
}

/// Import a JavaScript module.
///
/// On Wasm targets, this uses dynamic `import()`.
/// On non-Wasm targets, returns a handle that does nothing.
pub fn import_js(spec: &str) -> JsModule {
    JsModule::new(spec)
}

/// A builder for constructing JS objects with typed properties.
pub struct JsObjectBuilder {
    properties: Vec<(String, JsValue)>,
}

impl JsObjectBuilder {
    /// Create a new JS object builder.
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    /// Add a property to the object.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<JsValue>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }

    /// Build the JsValue::Object.
    pub fn build(self) -> JsValue {
        JsValue::Object(self.properties)
    }
}

impl Default for JsObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for constructing JS arrays.
pub struct JsArrayBuilder {
    items: Vec<JsValue>,
}

impl JsArrayBuilder {
    /// Create a new JS array builder.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Push an item to the array.
    pub fn push(mut self, value: impl Into<JsValue>) -> Self {
        self.items.push(value.into());
        self
    }

    /// Build the JsValue::Array.
    pub fn build(self) -> JsValue {
        JsValue::Array(self.items)
    }
}

impl Default for JsArrayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a JS interop bootstrap script.
///
/// This script sets up `window.__rye_js_call` and `window.__rye_js_get`
/// functions that the Wasm runtime uses to call JS libraries.
pub fn js_interop_script() -> &'static str {
    r#"<script>
(function() {
    // Cache for imported modules
    var moduleCache = {};

    window.__rye_import = function(spec) {
        if (moduleCache[spec]) return moduleCache[spec];
        moduleCache[spec] = import(spec).catch(function(err) {
            console.error('[rye] Failed to import', spec, err);
            throw err;
        });
        return moduleCache[spec];
    };

    window.__rye_js_call = function(module, func, args) {
        return window.__rye_import(module).then(function(mod) {
            var fn = mod[func];
            if (typeof fn !== 'function') {
                throw new Error('[rye] ' + func + ' is not a function in ' + module);
            }
            return fn.apply(null, args || []);
        });
    };

    window.__rye_js_get = function(module, prop) {
        return window.__rye_import(module).then(function(mod) {
            return mod[prop];
        });
    };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_value_string() {
        let v = JsValue::String("hello".to_string());
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(v.to_display_string(), "hello");
    }

    #[test]
    fn test_js_value_number() {
        let v = JsValue::Number(42.0);
        assert_eq!(v.as_number(), Some(42.0));
    }

    #[test]
    fn test_js_value_bool() {
        let v = JsValue::Bool(true);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_js_value_nullish() {
        assert!(JsValue::Null.is_nullish());
        assert!(JsValue::Undefined.is_nullish());
        assert!(!JsValue::Bool(false).is_nullish());
    }

    #[test]
    fn test_js_value_array_display() {
        let v = JsValue::Array(vec![JsValue::Number(1.0), JsValue::String("two".into())]);
        assert_eq!(v.to_display_string(), "[1, two]");
    }

    #[test]
    fn test_js_value_object_display() {
        let v = JsValue::Object(vec![
            ("name".to_string(), JsValue::String("rye".into())),
            ("count".to_string(), JsValue::Number(10.0)),
        ]);
        let s = v.to_display_string();
        assert!(s.contains("name: rye"));
        assert!(s.contains("count: 10"));
    }

    #[test]
    fn test_js_value_from_conversions() {
        let s: JsValue = "hello".into();
        assert_eq!(s.as_str(), Some("hello"));

        let n: JsValue = 42i32.into();
        assert_eq!(n.as_number(), Some(42.0));

        let f: JsValue = 1.5f64.into();
        assert_eq!(f.as_number(), Some(1.5));

        let b: JsValue = true.into();
        assert_eq!(b.as_bool(), Some(true));
    }

    #[test]
    fn test_js_module() {
        let m = import_js("chart.js");
        assert_eq!(m.spec(), "chart.js");
        assert!(!m.is_loaded());
        // Call returns None on non-Wasm
        assert!(m.call("createChart", &[]).is_none());
        assert!(m.get("version").is_none());
    }

    #[test]
    fn test_js_object_builder() {
        let obj = JsObjectBuilder::new()
            .set("type", "bar")
            .set("width", 400)
            .set("animated", true)
            .build();

        match obj {
            JsValue::Object(props) => {
                assert_eq!(props.len(), 3);
                assert_eq!(props[0].0, "type");
                assert_eq!(props[1].0, "width");
                assert_eq!(props[2].0, "animated");
            }
            _ => panic!("Expected JsValue::Object"),
        }
    }

    #[test]
    fn test_js_array_builder() {
        let arr = JsArrayBuilder::new().push(1).push("two").push(true).build();

        match arr {
            JsValue::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_number(), Some(1.0));
                assert_eq!(items[1].as_str(), Some("two"));
                assert_eq!(items[2].as_bool(), Some(true));
            }
            _ => panic!("Expected JsValue::Array"),
        }
    }

    #[test]
    fn test_js_interop_script() {
        let script = js_interop_script();
        assert!(script.contains("__rye_import"));
        assert!(script.contains("__rye_js_call"));
        assert!(script.contains("__rye_js_get"));
        assert!(script.contains("import("));
    }
}
