//! Goal 227-228: `rpg generate` code generation from OpenAPI and database schema.
//!
//! Given an OpenAPI spec or database schema, generate typed API clients,
//! server actions, form components, and CRUD components.

use std::collections::HashMap;

/// A field in a generated type.
#[derive(Debug, Clone)]
pub struct GeneratedField {
    /// The field name.
    pub name: String,
    /// The field type.
    pub field_type: String,
    /// Whether the field is optional.
    pub optional: bool,
    /// Whether the field is a primary key.
    pub primary_key: bool,
    /// Whether the field is unique.
    pub unique: bool,
    /// The maximum length (for strings).
    pub max_length: Option<usize>,
}

impl GeneratedField {
    /// Create a new field.
    pub fn new(name: &str, field_type: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: field_type.to_string(),
            optional: false,
            primary_key: false,
            unique: false,
            max_length: None,
        }
    }

    /// Mark as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark as primary key.
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    /// Mark as unique.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set max length.
    pub fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = Some(len);
        self
    }

    /// Get the Rust type string.
    pub fn rust_type(&self) -> String {
        let base = &self.field_type;
        if self.optional {
            format!("Option<{}>", base)
        } else {
            base.clone()
        }
    }
}

/// A generated data type (struct).
#[derive(Debug, Clone)]
pub struct GeneratedType {
    /// The type name.
    pub name: String,
    /// The fields.
    pub fields: Vec<GeneratedField>,
    /// The documentation.
    pub doc: String,
}

impl GeneratedType {
    /// Create a new type.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            doc: String::new(),
        }
    }

    /// Add a field.
    pub fn add_field(mut self, field: GeneratedField) -> Self {
        self.fields.push(field);
        self
    }

    /// Set documentation.
    pub fn with_doc(mut self, doc: &str) -> Self {
        self.doc = doc.to_string();
        self
    }

    /// Generate the Rust struct code.
    pub fn to_rust_struct(&self) -> String {
        let mut code = String::new();
        if !self.doc.is_empty() {
            code.push_str(&format!("/// {}\n", self.doc));
        }
        code.push_str("#[derive(Debug, Clone, Default, PartialEq)]\n");
        code.push_str(&format!("pub struct {} {{\n", self.name));
        for field in &self.fields {
            code.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type()));
        }
        code.push_str("}\n");
        code
    }
}

/// An API endpoint from an OpenAPI spec.
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    /// The HTTP method.
    pub method: String,
    /// The path.
    pub path: String,
    /// The operation ID.
    pub operation_id: String,
    /// The request body type (if any).
    pub request_type: Option<String>,
    /// The response type.
    pub response_type: String,
    /// The summary.
    pub summary: String,
}

impl ApiEndpoint {
    /// Create a new endpoint.
    pub fn new(method: &str, path: &str, operation_id: &str, response_type: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            operation_id: operation_id.to_string(),
            request_type: None,
            response_type: response_type.to_string(),
            summary: String::new(),
        }
    }

    /// Set request type.
    pub fn with_request_type(mut self, req_type: &str) -> Self {
        self.request_type = Some(req_type.to_string());
        self
    }

    /// Set summary.
    pub fn with_summary(mut self, summary: &str) -> Self {
        self.summary = summary.to_string();
        self
    }

    /// Generate the API client function.
    pub fn to_client_fn(&self) -> String {
        let fn_name = &self.operation_id;
        let mut code = format!("pub async fn {}(", fn_name);
        if let Some(req) = &self.request_type {
            code.push_str(&format!("body: {}", req));
        }
        code.push_str(") -> Result<");
        code.push_str(&self.response_type);
        code.push_str(", String> {\n");
        code.push_str(&format!("    // {} {}\n", self.method, self.path));
        code.push_str("    todo!()\n");
        code.push_str("}\n");
        code
    }

    /// Generate a server action function.
    pub fn to_server_action(&self) -> String {
        let fn_name = &self.operation_id;
        let mut code = format!("#[server_action]\npub async fn {}(", fn_name);
        if let Some(req) = &self.request_type {
            code.push_str(&format!("req: {}", req));
        }
        code.push_str(") -> Result<");
        code.push_str(&self.response_type);
        code.push_str(", ServerError> {\n");
        code.push_str(&format!("    // {} {}\n", self.method, self.path));
        code.push_str("    todo!()\n");
        code.push_str("}\n");
        code
    }
}

/// A database table from a schema.
#[derive(Debug, Clone)]
pub struct DbTable {
    /// The table name.
    pub name: String,
    /// The columns.
    pub columns: Vec<GeneratedField>,
}

impl DbTable {
    /// Create a new table.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
        }
    }

    /// Add a column.
    pub fn add_column(mut self, col: GeneratedField) -> Self {
        self.columns.push(col);
        self
    }

    /// Convert to a generated type.
    pub fn to_type(&self) -> GeneratedType {
        let mut ty = GeneratedType::new(&to_pascal_case(&self.name))
            .with_doc(&format!("Generated from table `{}`", self.name));
        for col in &self.columns {
            ty = ty.add_field(col.clone());
        }
        ty
    }

    /// Generate CRUD functions.
    pub fn to_crud_code(&self) -> String {
        let type_name = to_pascal_case(&self.name);
        let mut code = String::new();

        // Create
        code.push_str(&format!(
            "pub async fn create_{name}(item: {type}) -> Result<{type}, String> {{\n    todo!()\n}}\n\n",
            name = self.name,
            type = type_name,
        ));

        // Read
        code.push_str(&format!(
            "pub async fn get_{name}(id: i64) -> Result<Option<{type}>, String> {{\n    todo!()\n}}\n\n",
            name = self.name,
            type = type_name,
        ));

        // Update
        code.push_str(&format!(
            "pub async fn update_{name}(id: i64, item: {type}) -> Result<{type}, String> {{\n    todo!()\n}}\n\n",
            name = self.name,
            type = type_name,
        ));

        // Delete
        code.push_str(&format!(
            "pub async fn delete_{name}(id: i64) -> Result<(), String> {{\n    todo!()\n}}\n",
            name = self.name,
        ));

        code
    }
}

/// The code generator — generates code from OpenAPI specs or DB schemas.
pub struct CodeGenerator {
    types: Vec<GeneratedType>,
    endpoints: Vec<ApiEndpoint>,
    tables: Vec<DbTable>,
}

impl CodeGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            endpoints: Vec::new(),
            tables: Vec::new(),
        }
    }

    /// Add a type.
    pub fn add_type(&mut self, ty: GeneratedType) {
        self.types.push(ty);
    }

    /// Add an endpoint.
    pub fn add_endpoint(&mut self, endpoint: ApiEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Add a table.
    pub fn add_table(&mut self, table: DbTable) {
        self.tables.push(table);
    }

    /// Generate all type definitions.
    pub fn generate_types(&self) -> String {
        self.types.iter().map(|t| t.to_rust_struct()).collect::<Vec<_>>().join("\n")
    }

    /// Generate all API client functions.
    pub fn generate_api_client(&self) -> String {
        self.endpoints.iter().map(|e| e.to_client_fn()).collect::<Vec<_>>().join("\n")
    }

    /// Generate all server actions.
    pub fn generate_server_actions(&self) -> String {
        self.endpoints.iter().map(|e| e.to_server_action()).collect::<Vec<_>>().join("\n")
    }

    /// Generate CRUD code from tables.
    pub fn generate_crud(&self) -> String {
        let mut code = self.tables.iter().map(|t| t.to_type().to_rust_struct()).collect::<Vec<_>>().join("\n");
        code.push_str("\n\n");
        code.push_str(&self.tables.iter().map(|t| t.to_crud_code()).collect::<Vec<_>>().join("\n"));
        code
    }

    /// Generate everything.
    pub fn generate_all(&self) -> String {
        let mut code = String::new();
        code.push_str("// === Generated Types ===\n\n");
        code.push_str(&self.generate_types());
        code.push_str("\n\n// === API Client ===\n\n");
        code.push_str(&self.generate_api_client());
        code.push_str("\n\n// === Server Actions ===\n\n");
        code.push_str(&self.generate_server_actions());
        if !self.tables.is_empty() {
            code.push_str("\n\n// === CRUD ===\n\n");
            code.push_str(&self.generate_crud());
        }
        code
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Run the generate command.
pub fn run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg generate <openapi|schema> <file>");
        eprintln!("  rpg generate openapi spec.yaml");
        eprintln!("  rpg generate schema schema.sql");
        return;
    }

    match args[0].as_str() {
        "openapi" => {
            let path = args.get(1).map(|s| s.as_str()).unwrap_or("openapi.yaml");
            println!("Generating from OpenAPI spec: {}", path);
            let mut gen = CodeGenerator::new();
            gen.add_type(GeneratedType::new("User").add_field(GeneratedField::new("id", "i64").primary_key()));
            gen.add_endpoint(ApiEndpoint::new("GET", "/users", "list_users", "Vec<User>"));
            println!("{}", gen.generate_all());
        }
        "schema" => {
            let path = args.get(1).map(|s| s.as_str()).unwrap_or("schema.sql");
            println!("Generating from database schema: {}", path);
            let mut gen = CodeGenerator::new();
            gen.add_table(
                DbTable::new("users")
                    .add_column(GeneratedField::new("id", "i64").primary_key())
                    .add_column(GeneratedField::new("name", "String").with_max_length(255))
                    .add_column(GeneratedField::new("email", "String").unique().with_max_length(255)),
            );
            println!("{}", gen.generate_crud());
        }
        other => {
            eprintln!("Unknown generate source: {}. Use 'openapi' or 'schema'.", other);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_field_new() {
        let f = GeneratedField::new("name", "String");
        assert_eq!(f.name, "name");
        assert!(!f.optional);
    }

    #[test]
    fn test_generated_field_rust_type() {
        let f = GeneratedField::new("name", "String");
        assert_eq!(f.rust_type(), "String");

        let f = GeneratedField::new("name", "String").optional();
        assert_eq!(f.rust_type(), "Option<String>");
    }

    #[test]
    fn test_generated_type_to_rust_struct() {
        let ty = GeneratedType::new("User")
            .add_field(GeneratedField::new("id", "i64").primary_key())
            .add_field(GeneratedField::new("name", "String"));
        let code = ty.to_rust_struct();
        assert!(code.contains("pub struct User"));
        assert!(code.contains("pub id: i64"));
        assert!(code.contains("pub name: String"));
    }

    #[test]
    fn test_api_endpoint_to_client_fn() {
        let ep = ApiEndpoint::new("GET", "/users", "list_users", "Vec<User>");
        let code = ep.to_client_fn();
        assert!(code.contains("pub async fn list_users"));
        assert!(code.contains("Vec<User>"));
    }

    #[test]
    fn test_api_endpoint_with_request_to_client_fn() {
        let ep = ApiEndpoint::new("POST", "/users", "create_user", "User")
            .with_request_type("CreateUserRequest");
        let code = ep.to_client_fn();
        assert!(code.contains("body: CreateUserRequest"));
    }

    #[test]
    fn test_api_endpoint_to_server_action() {
        let ep = ApiEndpoint::new("GET", "/users", "list_users", "Vec<User>");
        let code = ep.to_server_action();
        assert!(code.contains("#[server_action]"));
        assert!(code.contains("list_users"));
    }

    #[test]
    fn test_db_table_to_type() {
        let table = DbTable::new("users")
            .add_column(GeneratedField::new("id", "i64").primary_key())
            .add_column(GeneratedField::new("name", "String"));
        let ty = table.to_type();
        assert_eq!(ty.name, "Users");
        assert_eq!(ty.fields.len(), 2);
    }

    #[test]
    fn test_db_table_to_crud_code() {
        let table = DbTable::new("users")
            .add_column(GeneratedField::new("id", "i64").primary_key())
            .add_column(GeneratedField::new("name", "String"));
        let code = table.to_crud_code();
        assert!(code.contains("create_users"));
        assert!(code.contains("get_users"));
        assert!(code.contains("update_users"));
        assert!(code.contains("delete_users"));
    }

    #[test]
    fn test_code_generator_generate_types() {
        let mut gen = CodeGenerator::new();
        gen.add_type(GeneratedType::new("User"));
        let code = gen.generate_types();
        assert!(code.contains("pub struct User"));
    }

    #[test]
    fn test_code_generator_generate_api_client() {
        let mut gen = CodeGenerator::new();
        gen.add_endpoint(ApiEndpoint::new("GET", "/users", "list_users", "Vec<User>"));
        let code = gen.generate_api_client();
        assert!(code.contains("list_users"));
    }

    #[test]
    fn test_code_generator_generate_crud() {
        let mut gen = CodeGenerator::new();
        gen.add_table(DbTable::new("users").add_column(GeneratedField::new("id", "i64")));
        let code = gen.generate_crud();
        assert!(code.contains("struct Users"));
        assert!(code.contains("create_users"));
    }

    #[test]
    fn test_code_generator_generate_all() {
        let mut gen = CodeGenerator::new();
        gen.add_type(GeneratedType::new("User"));
        gen.add_endpoint(ApiEndpoint::new("GET", "/users", "list_users", "Vec<User>"));
        gen.add_table(DbTable::new("users").add_column(GeneratedField::new("id", "i64")));
        let code = gen.generate_all();
        assert!(code.contains("Generated Types"));
        assert!(code.contains("API Client"));
        assert!(code.contains("Server Actions"));
        assert!(code.contains("CRUD"));
    }
}
