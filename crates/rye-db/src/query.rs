//! Query builder — type-safe SQL query construction.

use std::fmt;

/// A SQL query — built by the query builder.
#[derive(Debug, Clone)]
pub struct Query {
    /// The table name.
    pub table: String,
    /// The query type.
    pub kind: QueryKind,
    /// Columns to select / insert / update.
    pub columns: Vec<String>,
    /// WHERE conditions.
    pub conditions: Vec<Condition>,
    /// ORDER BY clauses.
    pub order_by: Vec<(String, OrderDirection)>,
    /// LIMIT.
    pub limit: Option<usize>,
    /// OFFSET.
    pub offset: Option<usize>,
    /// Values for INSERT / UPDATE.
    pub values: Vec<(String, ValueType)>,
}

/// The type of SQL query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    Select,
    Insert,
    Update,
    Delete,
}

/// A WHERE condition.
#[derive(Debug, Clone)]
pub struct Condition {
    /// Column name.
    pub column: String,
    /// The operator.
    pub operator: ConditionOperator,
    /// The value to compare against.
    pub value: ValueType,
}

/// Condition operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    IsNull,
    IsNotNull,
}

impl ConditionOperator {
    /// Convert to SQL string.
    pub fn as_sql(&self) -> &'static str {
        match self {
            ConditionOperator::Eq => "=",
            ConditionOperator::Ne => "!=",
            ConditionOperator::Lt => "<",
            ConditionOperator::Le => "<=",
            ConditionOperator::Gt => ">",
            ConditionOperator::Ge => ">=",
            ConditionOperator::Like => "LIKE",
            ConditionOperator::IsNull => "IS NULL",
            ConditionOperator::IsNotNull => "IS NOT NULL",
        }
    }
}

/// ORDER BY direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    /// Convert to SQL string.
    pub fn as_sql(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// A value type for query parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// String value.
    Text(String),
    /// Boolean value.
    Bool(bool),
    /// NULL value.
    Null,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Int(v) => write!(f, "{}", v),
            ValueType::Float(v) => write!(f, "{}", v),
            ValueType::Text(v) => write!(f, "'{}'", v.replace('\'', "''")),
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::Null => write!(f, "NULL"),
        }
    }
}

/// The query result — rows returned by a SELECT query.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    /// The column names.
    pub columns: Vec<String>,
    /// The rows (each row is a vec of values).
    pub rows: Vec<Vec<ValueType>>,
}

impl QueryResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Get the number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get a row by index.
    pub fn row(&self, index: usize) -> Option<&[ValueType]> {
        self.rows.get(index).map(|r| r.as_slice())
    }

    /// Get a column index by name.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c == name)
    }

    /// Get a value from a specific row and column.
    pub fn get(&self, row: usize, column: &str) -> Option<&ValueType> {
        let col_idx = self.column_index(column)?;
        self.rows.get(row)?.get(col_idx)
    }

    /// Check if the result is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The query builder — construct SQL queries fluently.
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    /// Start a SELECT query.
    pub fn select(table: &str) -> Self {
        Self {
            query: Query {
                table: table.to_string(),
                kind: QueryKind::Select,
                columns: vec!["*".to_string()],
                conditions: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                values: Vec::new(),
            },
        }
    }

    /// Start a SELECT query with specific columns.
    pub fn select_cols(table: &str, columns: &[&str]) -> Self {
        Self {
            query: Query {
                table: table.to_string(),
                kind: QueryKind::Select,
                columns: columns.iter().map(|s| s.to_string()).collect(),
                conditions: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                values: Vec::new(),
            },
        }
    }

    /// Start an INSERT query.
    pub fn insert(table: &str) -> Self {
        Self {
            query: Query {
                table: table.to_string(),
                kind: QueryKind::Insert,
                columns: Vec::new(),
                conditions: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                values: Vec::new(),
            },
        }
    }

    /// Start an UPDATE query.
    pub fn update(table: &str) -> Self {
        Self {
            query: Query {
                table: table.to_string(),
                kind: QueryKind::Update,
                columns: Vec::new(),
                conditions: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                values: Vec::new(),
            },
        }
    }

    /// Start a DELETE query.
    pub fn delete(table: &str) -> Self {
        Self {
            query: Query {
                table: table.to_string(),
                kind: QueryKind::Delete,
                columns: Vec::new(),
                conditions: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                values: Vec::new(),
            },
        }
    }

    /// Add a WHERE condition (equals).
    pub fn where_eq(mut self, column: &str, value: ValueType) -> Self {
        self.query.conditions.push(Condition {
            column: column.to_string(),
            operator: ConditionOperator::Eq,
            value,
        });
        self
    }

    /// Add a WHERE condition (custom operator).
    pub fn where_cond(
        mut self,
        column: &str,
        operator: ConditionOperator,
        value: ValueType,
    ) -> Self {
        self.query.conditions.push(Condition {
            column: column.to_string(),
            operator,
            value,
        });
        self
    }

    /// Add a WHERE IS NULL condition.
    pub fn where_null(mut self, column: &str) -> Self {
        self.query.conditions.push(Condition {
            column: column.to_string(),
            operator: ConditionOperator::IsNull,
            value: ValueType::Null,
        });
        self
    }

    /// Add ORDER BY.
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.query.order_by.push((column.to_string(), direction));
        self
    }

    /// Set LIMIT.
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Set OFFSET.
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.offset = Some(offset);
        self
    }

    /// Add a value for INSERT/UPDATE.
    pub fn set(mut self, column: &str, value: ValueType) -> Self {
        self.query.values.push((column.to_string(), value));
        self
    }

    /// Build the query.
    pub fn build(self) -> Query {
        self.query
    }

    /// Build the SQL string.
    pub fn to_sql(self) -> String {
        self.build().to_sql()
    }
}

impl Query {
    /// Convert the query to a SQL string.
    pub fn to_sql(&self) -> String {
        match self.kind {
            QueryKind::Select => self.to_select_sql(),
            QueryKind::Insert => self.to_insert_sql(),
            QueryKind::Update => self.to_update_sql(),
            QueryKind::Delete => self.to_delete_sql(),
        }
    }

    fn to_select_sql(&self) -> String {
        let cols = self.columns.join(", ");
        let mut sql = format!("SELECT {} FROM {}", cols, self.table);

        if !self.conditions.is_empty() {
            let where_clause: Vec<String> = self
                .conditions
                .iter()
                .map(|c| {
                    if matches!(
                        c.operator,
                        ConditionOperator::IsNull | ConditionOperator::IsNotNull
                    ) {
                        format!("{} {}", c.column, c.operator.as_sql())
                    } else {
                        format!("{} {} {}", c.column, c.operator.as_sql(), c.value)
                    }
                })
                .collect();
            sql.push_str(&format!(" WHERE {}", where_clause.join(" AND ")));
        }

        if !self.order_by.is_empty() {
            let order: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, dir)| format!("{} {}", col, dir.as_sql()))
                .collect();
            sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    fn to_insert_sql(&self) -> String {
        let cols: Vec<&str> = self.values.iter().map(|(c, _)| c.as_str()).collect();
        let vals: Vec<String> = self.values.iter().map(|(_, v)| v.to_string()).collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            cols.join(", "),
            vals.join(", ")
        )
    }

    fn to_update_sql(&self) -> String {
        let sets: Vec<String> = self
            .values
            .iter()
            .map(|(c, v)| format!("{} = {}", c, v))
            .collect();
        let mut sql = format!("UPDATE {} SET {}", self.table, sets.join(", "));

        if !self.conditions.is_empty() {
            let where_clause: Vec<String> = self
                .conditions
                .iter()
                .map(|c| format!("{} {} {}", c.column, c.operator.as_sql(), c.value))
                .collect();
            sql.push_str(&format!(" WHERE {}", where_clause.join(" AND ")));
        }

        sql
    }

    fn to_delete_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", self.table);

        if !self.conditions.is_empty() {
            let where_clause: Vec<String> = self
                .conditions
                .iter()
                .map(|c| {
                    if matches!(
                        c.operator,
                        ConditionOperator::IsNull | ConditionOperator::IsNotNull
                    ) {
                        format!("{} {}", c.column, c.operator.as_sql())
                    } else {
                        format!("{} {} {}", c.column, c.operator.as_sql(), c.value)
                    }
                })
                .collect();
            sql.push_str(&format!(" WHERE {}", where_clause.join(" AND ")));
        }

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_all() {
        let sql = QueryBuilder::select("users").to_sql();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_select_columns() {
        let sql = QueryBuilder::select_cols("users", &["id", "name", "email"]).to_sql();
        assert_eq!(sql, "SELECT id, name, email FROM users");
    }

    #[test]
    fn test_select_where() {
        let sql = QueryBuilder::select("users")
            .where_eq("id", ValueType::Int(42))
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE id = 42");
    }

    #[test]
    fn test_select_where_multiple() {
        let sql = QueryBuilder::select("users")
            .where_eq("active", ValueType::Bool(true))
            .where_cond("age", ConditionOperator::Gt, ValueType::Int(18))
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE active = true AND age > 18");
    }

    #[test]
    fn test_select_where_null() {
        let sql = QueryBuilder::select("users")
            .where_null("deleted_at")
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE deleted_at IS NULL");
    }

    #[test]
    fn test_select_order_by() {
        let sql = QueryBuilder::select("users")
            .order_by("name", OrderDirection::Asc)
            .order_by("id", OrderDirection::Desc)
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC, id DESC");
    }

    #[test]
    fn test_select_limit_offset() {
        let sql = QueryBuilder::select("users").limit(10).offset(20).to_sql();
        assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_insert() {
        let sql = QueryBuilder::insert("users")
            .set("name", ValueType::Text("Alice".to_string()))
            .set("age", ValueType::Int(30))
            .to_sql();
        assert_eq!(sql, "INSERT INTO users (name, age) VALUES ('Alice', 30)");
    }

    #[test]
    fn test_update() {
        let sql = QueryBuilder::update("users")
            .set("name", ValueType::Text("Bob".to_string()))
            .where_eq("id", ValueType::Int(1))
            .to_sql();
        assert_eq!(sql, "UPDATE users SET name = 'Bob' WHERE id = 1");
    }

    #[test]
    fn test_delete() {
        let sql = QueryBuilder::delete("users")
            .where_eq("id", ValueType::Int(1))
            .to_sql();
        assert_eq!(sql, "DELETE FROM users WHERE id = 1");
    }

    #[test]
    fn test_delete_all() {
        let sql = QueryBuilder::delete("users").to_sql();
        assert_eq!(sql, "DELETE FROM users");
    }

    #[test]
    fn test_value_type_display() {
        assert_eq!(ValueType::Int(42).to_string(), "42");
        assert_eq!(ValueType::Float(1.5).to_string(), "1.5");
        assert_eq!(ValueType::Text("hello".to_string()).to_string(), "'hello'");
        assert_eq!(ValueType::Bool(true).to_string(), "true");
        assert_eq!(ValueType::Null.to_string(), "NULL");
    }

    #[test]
    fn test_value_type_text_escape() {
        assert_eq!(ValueType::Text("it's".to_string()).to_string(), "'it''s'");
    }

    #[test]
    fn test_condition_operator_as_sql() {
        assert_eq!(ConditionOperator::Eq.as_sql(), "=");
        assert_eq!(ConditionOperator::Ne.as_sql(), "!=");
        assert_eq!(ConditionOperator::Lt.as_sql(), "<");
        assert_eq!(ConditionOperator::Like.as_sql(), "LIKE");
        assert_eq!(ConditionOperator::IsNull.as_sql(), "IS NULL");
    }

    #[test]
    fn test_order_direction_as_sql() {
        assert_eq!(OrderDirection::Asc.as_sql(), "ASC");
        assert_eq!(OrderDirection::Desc.as_sql(), "DESC");
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn test_query_result_get() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![ValueType::Int(1), ValueType::Text("Alice".to_string())],
                vec![ValueType::Int(2), ValueType::Text("Bob".to_string())],
            ],
        };

        assert_eq!(result.row_count(), 2);
        assert_eq!(
            result.get(0, "name"),
            Some(&ValueType::Text("Alice".to_string()))
        );
        assert_eq!(result.get(1, "id"), Some(&ValueType::Int(2)));
        assert_eq!(result.get(5, "name"), None);
    }

    #[test]
    fn test_query_result_column_index() {
        let result = QueryResult {
            columns: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            rows: vec![],
        };
        assert_eq!(result.column_index("b"), Some(1));
        assert_eq!(result.column_index("z"), None);
    }

    #[test]
    fn test_select_like() {
        let sql = QueryBuilder::select("users")
            .where_cond(
                "name",
                ConditionOperator::Like,
                ValueType::Text("%alice%".to_string()),
            )
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE name LIKE '%alice%'");
    }
}
