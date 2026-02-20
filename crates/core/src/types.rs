//! Core types used throughout Immortal Engine
//!
//! This module contains the fundamental types that form the foundation
//! of the type system, used by the IR, components, and code generation systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Unique Identifiers
// ============================================================================

/// Type alias for entity/node unique identifiers
pub type EntityId = uuid::Uuid;

/// Type alias for field unique identifiers
pub type FieldId = uuid::Uuid;

/// Type alias for relationship unique identifiers
pub type RelationshipId = uuid::Uuid;

/// Type alias for endpoint unique identifiers
pub type EndpointId = uuid::Uuid;

// ============================================================================
// Geometry Types
// ============================================================================

/// Position on the 2D canvas
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    /// Create a new position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a position at the origin (0, 0)
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate the Euclidean distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Add an offset to this position
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Linear interpolation between two positions
    pub fn lerp(&self, other: &Position, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Size of a component on the canvas
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a zero size
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Default size for entity cards
    pub fn default_entity() -> Self {
        Self {
            width: 220.0,
            height: 180.0,
        }
    }

    /// Default size for endpoint cards
    pub fn default_endpoint() -> Self {
        Self {
            width: 200.0,
            height: 160.0,
        }
    }

    /// Calculate the area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if the size contains a point (assuming origin at 0,0)
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= 0.0 && x <= self.width && y >= 0.0 && y <= self.height
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::default_entity()
    }
}

/// Bounding rectangle for components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub position: Position,
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(position: Position, size: Size) -> Self {
        Self { position, size }
    }

    /// Create a rectangle from coordinates and dimensions
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Position::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Check if a point is contained within this rectangle
    pub fn contains(&self, point: Position) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.height
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.position.x < other.position.x + other.size.width
            && self.position.x + self.size.width > other.position.x
            && self.position.y < other.position.y + other.size.height
            && self.position.y + self.size.height > other.position.y
    }

    /// Get the center point of the rectangle
    pub fn center(&self) -> Position {
        Position {
            x: self.position.x + self.size.width / 2.0,
            y: self.position.y + self.size.height / 2.0,
        }
    }

    /// Get the top-left corner
    pub fn top_left(&self) -> Position {
        self.position
    }

    /// Get the top-right corner
    pub fn top_right(&self) -> Position {
        Position::new(self.position.x + self.size.width, self.position.y)
    }

    /// Get the bottom-left corner
    pub fn bottom_left(&self) -> Position {
        Position::new(self.position.x, self.position.y + self.size.height)
    }

    /// Get the bottom-right corner
    pub fn bottom_right(&self) -> Position {
        Position::new(
            self.position.x + self.size.width,
            self.position.y + self.size.height,
        )
    }

    /// Expand the rectangle by a uniform amount
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            position: Position::new(self.position.x - amount, self.position.y - amount),
            size: Size::new(
                self.size.width + amount * 2.0,
                self.size.height + amount * 2.0,
            ),
        }
    }

    /// Get the union of two rectangles (bounding box containing both)
    pub fn union(&self, other: &Rect) -> Self {
        let min_x = self.position.x.min(other.position.x);
        let min_y = self.position.y.min(other.position.y);
        let max_x = (self.position.x + self.size.width).max(other.position.x + other.size.width);
        let max_y = (self.position.y + self.size.height).max(other.position.y + other.size.height);

        Self {
            position: Position::new(min_x, min_y),
            size: Size::new(max_x - min_x, max_y - min_y),
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: Position::default(),
            size: Size::default(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Data types supported for entity fields
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "params")]
pub enum DataType {
    // Primitive Types
    /// Variable-length string (VARCHAR)
    String,
    /// Long-form text content (TEXT/CLOB)
    Text,
    /// 32-bit signed integer
    Int32,
    /// 64-bit signed integer
    Int64,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point (double precision)
    Float64,
    /// Boolean true/false
    Bool,
    /// UUID (universally unique identifier)
    Uuid,
    /// Date and time with timezone
    DateTime,
    /// Date without time
    Date,
    /// Time without date
    Time,
    /// Binary data (BYTEA/BLOB)
    Bytes,
    /// JSON/JSONB data
    Json,

    // Complex Types
    /// Optional/nullable wrapper
    Optional(Box<DataType>),
    /// Array/list of items
    Array(Box<DataType>),

    // Reference Types
    /// Foreign key reference to another entity
    Reference {
        entity_name: String,
        field_name: String,
    },

    // Enum Type
    /// Enumeration with named variants
    Enum { name: String, variants: Vec<String> },
}

impl DataType {
    /// Convert to Rust type string
    pub fn to_rust_type(&self) -> String {
        match self {
            DataType::String | DataType::Text => "String".to_string(),
            DataType::Int32 => "i32".to_string(),
            DataType::Int64 => "i64".to_string(),
            DataType::Float32 => "f32".to_string(),
            DataType::Float64 => "f64".to_string(),
            DataType::Bool => "bool".to_string(),
            DataType::Uuid => "uuid::Uuid".to_string(),
            DataType::DateTime => "chrono::DateTime<chrono::Utc>".to_string(),
            DataType::Date => "chrono::NaiveDate".to_string(),
            DataType::Time => "chrono::NaiveTime".to_string(),
            DataType::Bytes => "Vec<u8>".to_string(),
            DataType::Json => "serde_json::Value".to_string(),
            DataType::Optional(inner) => format!("Option<{}>", inner.to_rust_type()),
            DataType::Array(inner) => format!("Vec<{}>", inner.to_rust_type()),
            DataType::Reference { entity_name, .. } => format!("{}Id", entity_name),
            DataType::Enum { name, .. } => name.clone(),
        }
    }

    /// Convert to SeaORM column type string
    pub fn to_sea_orm_type(&self) -> String {
        match self {
            DataType::String => "String(StringLen::N(255))".to_string(),
            DataType::Text => "Text".to_string(),
            DataType::Int32 => "Integer".to_string(),
            DataType::Int64 => "BigInteger".to_string(),
            DataType::Float32 => "Float".to_string(),
            DataType::Float64 => "Double".to_string(),
            DataType::Bool => "Boolean".to_string(),
            DataType::Uuid => "Uuid".to_string(),
            DataType::DateTime => "TimestampWithTimeZone".to_string(),
            DataType::Date => "Date".to_string(),
            DataType::Time => "Time".to_string(),
            DataType::Bytes => "Binary(BlobSize::Blob(None))".to_string(),
            DataType::Json => "JsonBinary".to_string(),
            DataType::Optional(inner) => inner.to_sea_orm_type(),
            DataType::Array(inner) => format!("Array(RcOrArc::new({}))", inner.to_sea_orm_type()),
            DataType::Reference { .. } => "Uuid".to_string(),
            DataType::Enum { name, .. } => format!("String(StringLen::N(50)) /* {} */", name),
        }
    }

    /// Convert to SQL type string for a specific database
    pub fn to_sql_type(&self, db: DatabaseType) -> String {
        match db {
            DatabaseType::PostgreSQL => self.to_postgres_type(),
            DatabaseType::MySQL => self.to_mysql_type(),
            DatabaseType::SQLite => self.to_sqlite_type(),
        }
    }

    /// Convert to PostgreSQL type
    pub fn to_postgres_type(&self) -> String {
        match self {
            DataType::String => "VARCHAR(255)".to_string(),
            DataType::Text => "TEXT".to_string(),
            DataType::Int32 => "INTEGER".to_string(),
            DataType::Int64 => "BIGINT".to_string(),
            DataType::Float32 => "REAL".to_string(),
            DataType::Float64 => "DOUBLE PRECISION".to_string(),
            DataType::Bool => "BOOLEAN".to_string(),
            DataType::Uuid => "UUID".to_string(),
            DataType::DateTime => "TIMESTAMP WITH TIME ZONE".to_string(),
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::Bytes => "BYTEA".to_string(),
            DataType::Json => "JSONB".to_string(),
            DataType::Optional(inner) => inner.to_postgres_type(),
            DataType::Array(inner) => format!("{}[]", inner.to_postgres_type()),
            DataType::Reference { .. } => "UUID".to_string(),
            DataType::Enum { name, .. } => format!("VARCHAR(50) /* {} */", name),
        }
    }

    /// Convert to MySQL type
    pub fn to_mysql_type(&self) -> String {
        match self {
            DataType::String => "VARCHAR(255)".to_string(),
            DataType::Text => "LONGTEXT".to_string(),
            DataType::Int32 => "INT".to_string(),
            DataType::Int64 => "BIGINT".to_string(),
            DataType::Float32 => "FLOAT".to_string(),
            DataType::Float64 => "DOUBLE".to_string(),
            DataType::Bool => "TINYINT(1)".to_string(),
            DataType::Uuid => "CHAR(36)".to_string(),
            DataType::DateTime => "DATETIME".to_string(),
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::Bytes => "BLOB".to_string(),
            DataType::Json => "JSON".to_string(),
            DataType::Optional(inner) => inner.to_mysql_type(),
            DataType::Array(inner) => format!("JSON /* array of {} */", inner.to_mysql_type()),
            DataType::Reference { .. } => "CHAR(36)".to_string(),
            DataType::Enum { name, variants } => {
                let variants_str = variants
                    .iter()
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("ENUM({}) /* {} */", variants_str, name)
            }
        }
    }

    /// Convert to SQLite type
    pub fn to_sqlite_type(&self) -> String {
        match self {
            DataType::String | DataType::Text => "TEXT".to_string(),
            DataType::Int32 | DataType::Int64 => "INTEGER".to_string(),
            DataType::Float32 | DataType::Float64 => "REAL".to_string(),
            DataType::Bool => "INTEGER".to_string(),
            DataType::Uuid => "TEXT".to_string(),
            DataType::DateTime | DataType::Date | DataType::Time => "TEXT".to_string(),
            DataType::Bytes => "BLOB".to_string(),
            DataType::Json => "TEXT".to_string(),
            DataType::Optional(inner) => inner.to_sqlite_type(),
            DataType::Array(_) => "TEXT".to_string(), // JSON array
            DataType::Reference { .. } => "TEXT".to_string(),
            DataType::Enum { .. } => "TEXT".to_string(),
        }
    }

    /// Check if this type is nullable
    pub fn is_nullable(&self) -> bool {
        matches!(self, DataType::Optional(_))
    }

    /// Check if this type is a reference to another entity
    pub fn is_reference(&self) -> bool {
        matches!(self, DataType::Reference { .. })
    }

    /// Get a user-friendly display name
    pub fn display_name(&self) -> String {
        match self {
            DataType::String => "String".to_string(),
            DataType::Text => "Text".to_string(),
            DataType::Int32 => "Integer".to_string(),
            DataType::Int64 => "Big Integer".to_string(),
            DataType::Float32 => "Float".to_string(),
            DataType::Float64 => "Double".to_string(),
            DataType::Bool => "Boolean".to_string(),
            DataType::Uuid => "UUID".to_string(),
            DataType::DateTime => "DateTime".to_string(),
            DataType::Date => "Date".to_string(),
            DataType::Time => "Time".to_string(),
            DataType::Bytes => "Binary".to_string(),
            DataType::Json => "JSON".to_string(),
            DataType::Optional(inner) => format!("{}?", inner.display_name()),
            DataType::Array(inner) => format!("[{}]", inner.display_name()),
            DataType::Reference { entity_name, .. } => format!("Ref<{}>", entity_name),
            DataType::Enum { name, .. } => format!("Enum<{}>", name),
        }
    }

    /// Get all primitive types
    pub fn primitives() -> Vec<DataType> {
        vec![
            DataType::String,
            DataType::Text,
            DataType::Int32,
            DataType::Int64,
            DataType::Float32,
            DataType::Float64,
            DataType::Bool,
            DataType::Uuid,
            DataType::DateTime,
            DataType::Date,
            DataType::Time,
            DataType::Bytes,
            DataType::Json,
        ]
    }
}

impl Default for DataType {
    fn default() -> Self {
        DataType::String
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Database Types
// ============================================================================

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    #[default]
    PostgreSQL,
    MySQL,
    SQLite,
}

impl DatabaseType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MySQL => "MySQL",
            DatabaseType::SQLite => "SQLite",
        }
    }

    /// Get default port
    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0, // SQLite doesn't use ports
        }
    }

    /// Get connection string template
    pub fn connection_template(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgres://user:password@localhost:5432/database",
            DatabaseType::MySQL => "mysql://user:password@localhost:3306/database",
            DatabaseType::SQLite => "sqlite://./data.db",
        }
    }

    /// Get all database types
    pub fn all() -> &'static [DatabaseType] {
        &[
            DatabaseType::PostgreSQL,
            DatabaseType::MySQL,
            DatabaseType::SQLite,
        ]
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// ID Types
// ============================================================================

/// Primary key type for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IdType {
    /// UUID v4 (recommended)
    #[default]
    Uuid,
    /// Auto-incrementing integer
    Serial,
    /// Collision-resistant unique ID
    Cuid,
    /// ULID (Universally Unique Lexicographically Sortable Identifier)
    Ulid,
}

impl IdType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            IdType::Uuid => "UUID",
            IdType::Serial => "Serial (Auto-increment)",
            IdType::Cuid => "CUID",
            IdType::Ulid => "ULID",
        }
    }

    /// Get the Rust type for this ID type
    pub fn to_rust_type(&self) -> &'static str {
        match self {
            IdType::Uuid => "uuid::Uuid",
            IdType::Serial => "i64",
            IdType::Cuid => "String",
            IdType::Ulid => "String",
        }
    }

    /// Get all ID types
    pub fn all() -> &'static [IdType] {
        &[IdType::Uuid, IdType::Serial, IdType::Cuid, IdType::Ulid]
    }
}

impl std::fmt::Display for IdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Relationship Types
// ============================================================================

/// Entity relationship types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// One record relates to exactly one other record
    OneToOne,
    /// One record relates to many others (e.g., User has many Posts)
    OneToMany,
    /// Many records relate to one (inverse of OneToMany)
    ManyToOne,
    /// Many-to-many through junction table
    ManyToMany { junction_table: String },
}

impl RelationType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationType::OneToOne => "One to One",
            RelationType::OneToMany => "One to Many",
            RelationType::ManyToOne => "Many to One",
            RelationType::ManyToMany { .. } => "Many to Many",
        }
    }

    /// Get arrow symbol for visual representation
    pub fn arrow_symbol(&self) -> &'static str {
        match self {
            RelationType::OneToOne => "1 ─── 1",
            RelationType::OneToMany => "1 ───< *",
            RelationType::ManyToOne => "* >─── 1",
            RelationType::ManyToMany { .. } => "* >──< *",
        }
    }

    /// Check if this relationship requires a junction table
    pub fn requires_junction_table(&self) -> bool {
        matches!(self, RelationType::ManyToMany { .. })
    }

    /// Get the inverse relationship type
    pub fn inverse(&self) -> Self {
        match self {
            RelationType::OneToOne => RelationType::OneToOne,
            RelationType::OneToMany => RelationType::ManyToOne,
            RelationType::ManyToOne => RelationType::OneToMany,
            RelationType::ManyToMany { junction_table } => RelationType::ManyToMany {
                junction_table: junction_table.clone(),
            },
        }
    }
}

impl Default for RelationType {
    fn default() -> Self {
        RelationType::OneToMany
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Referential Actions
// ============================================================================

/// Actions for foreign key constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReferentialAction {
    /// Delete related records when parent is deleted
    Cascade,
    /// Set foreign key to NULL when parent is deleted
    SetNull,
    /// Prevent deletion if related records exist
    #[default]
    Restrict,
    /// Do nothing (database default)
    NoAction,
    /// Set to default value
    SetDefault,
}

impl ReferentialAction {
    /// Get SQL keyword
    pub fn to_sql(&self) -> &'static str {
        match self {
            ReferentialAction::Cascade => "CASCADE",
            ReferentialAction::SetNull => "SET NULL",
            ReferentialAction::Restrict => "RESTRICT",
            ReferentialAction::NoAction => "NO ACTION",
            ReferentialAction::SetDefault => "SET DEFAULT",
        }
    }

    /// Get all referential actions
    pub fn all() -> &'static [ReferentialAction] {
        &[
            ReferentialAction::Cascade,
            ReferentialAction::SetNull,
            ReferentialAction::Restrict,
            ReferentialAction::NoAction,
            ReferentialAction::SetDefault,
        ]
    }
}

impl std::fmt::Display for ReferentialAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

// ============================================================================
// Validation Types
// ============================================================================

/// Field validation rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Validation {
    /// Field must have a value
    Required,
    /// Minimum string length
    MinLength(usize),
    /// Maximum string length
    MaxLength(usize),
    /// Minimum numeric value
    Min(f64),
    /// Maximum numeric value
    Max(f64),
    /// Regex pattern validation
    Pattern { regex: String, message: String },
    /// Valid email address
    Email,
    /// Valid URL
    Url,
    /// Valid UUID format
    Uuid,
    /// Valid phone number
    Phone,
    /// Value must be in a list
    OneOf(Vec<String>),
    /// Custom validation with expression
    Custom { name: String, expression: String },
}

impl Validation {
    /// Get a user-friendly error message
    pub fn error_message(&self) -> String {
        match self {
            Validation::Required => "This field is required".to_string(),
            Validation::MinLength(n) => format!("Minimum length is {} characters", n),
            Validation::MaxLength(n) => format!("Maximum length is {} characters", n),
            Validation::Min(n) => format!("Minimum value is {}", n),
            Validation::Max(n) => format!("Maximum value is {}", n),
            Validation::Pattern { message, .. } => message.clone(),
            Validation::Email => "Must be a valid email address".to_string(),
            Validation::Url => "Must be a valid URL".to_string(),
            Validation::Uuid => "Must be a valid UUID".to_string(),
            Validation::Phone => "Must be a valid phone number".to_string(),
            Validation::OneOf(values) => format!("Must be one of: {}", values.join(", ")),
            Validation::Custom { name, .. } => format!("Failed validation: {}", name),
        }
    }

    /// Convert to validator crate attribute
    pub fn to_validator_attribute(&self) -> Option<String> {
        match self {
            Validation::Required => None, // Handled by Option type
            Validation::MinLength(n) => Some(format!("length(min = {})", n)),
            Validation::MaxLength(n) => Some(format!("length(max = {})", n)),
            Validation::Min(n) => Some(format!("range(min = {})", n)),
            Validation::Max(n) => Some(format!("range(max = {})", n)),
            Validation::Pattern { regex, .. } => Some(format!("regex(path = \"{}\")", regex)),
            Validation::Email => Some("email".to_string()),
            Validation::Url => Some("url".to_string()),
            Validation::Uuid => None, // Type system handles this
            Validation::Phone => Some("phone".to_string()),
            Validation::OneOf(_) => None, // Custom validator needed
            Validation::Custom { name, .. } => Some(format!("custom(function = \"{}\")", name)),
        }
    }
}

impl std::fmt::Display for Validation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Validation::Required => write!(f, "required"),
            Validation::MinLength(n) => write!(f, "min_length({})", n),
            Validation::MaxLength(n) => write!(f, "max_length({})", n),
            Validation::Min(n) => write!(f, "min({})", n),
            Validation::Max(n) => write!(f, "max({})", n),
            Validation::Pattern { regex, .. } => write!(f, "pattern({})", regex),
            Validation::Email => write!(f, "email"),
            Validation::Url => write!(f, "url"),
            Validation::Uuid => write!(f, "uuid"),
            Validation::Phone => write!(f, "phone"),
            Validation::OneOf(values) => write!(f, "one_of({:?})", values),
            Validation::Custom { name, .. } => write!(f, "custom({})", name),
        }
    }
}

// ============================================================================
// Configuration Values
// ============================================================================

/// Dynamic configuration values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Try to get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(v) => Some(*v),
            ConfigValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Try to get as string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    /// Try to get as array
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as object
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(v) => Some(v),
            _ => None,
        }
    }

    /// Check if null
    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }

    /// Check if boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, ConfigValue::Bool(_))
    }

    /// Check if numeric (int or float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, ConfigValue::Int(_) | ConfigValue::Float(_))
    }

    /// Check if string
    pub fn is_string(&self) -> bool {
        matches!(self, ConfigValue::String(_))
    }

    /// Check if array
    pub fn is_array(&self) -> bool {
        matches!(self, ConfigValue::Array(_))
    }

    /// Check if object
    pub fn is_object(&self) -> bool {
        matches!(self, ConfigValue::Object(_))
    }
}

impl Default for ConfigValue {
    fn default() -> Self {
        ConfigValue::Null
    }
}

impl From<bool> for ConfigValue {
    fn from(v: bool) -> Self {
        ConfigValue::Bool(v)
    }
}

impl From<i32> for ConfigValue {
    fn from(v: i32) -> Self {
        ConfigValue::Int(v as i64)
    }
}

impl From<i64> for ConfigValue {
    fn from(v: i64) -> Self {
        ConfigValue::Int(v)
    }
}

impl From<f64> for ConfigValue {
    fn from(v: f64) -> Self {
        ConfigValue::Float(v)
    }
}

impl From<f32> for ConfigValue {
    fn from(v: f32) -> Self {
        ConfigValue::Float(v as f64)
    }
}

impl From<String> for ConfigValue {
    fn from(v: String) -> Self {
        ConfigValue::String(v)
    }
}

impl From<&str> for ConfigValue {
    fn from(v: &str) -> Self {
        ConfigValue::String(v.to_string())
    }
}

impl<T: Into<ConfigValue>> From<Vec<T>> for ConfigValue {
    fn from(v: Vec<T>) -> Self {
        ConfigValue::Array(v.into_iter().map(Into::into).collect())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Position tests
    #[test]
    fn test_position_new() {
        let pos = Position::new(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_position_zero() {
        let pos = Position::zero();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position_offset() {
        let pos = Position::new(10.0, 20.0);
        let new_pos = pos.offset(5.0, -10.0);
        assert_eq!(new_pos.x, 15.0);
        assert_eq!(new_pos.y, 10.0);
    }

    #[test]
    fn test_position_lerp() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(10.0, 20.0);
        let mid = p1.lerp(&p2, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 10.0);
    }

    #[test]
    fn test_position_add_sub() {
        let p1 = Position::new(10.0, 20.0);
        let p2 = Position::new(5.0, 5.0);
        let sum = p1 + p2;
        let diff = p1 - p2;
        assert_eq!(sum.x, 15.0);
        assert_eq!(sum.y, 25.0);
        assert_eq!(diff.x, 5.0);
        assert_eq!(diff.y, 15.0);
    }

    // Size tests
    #[test]
    fn test_size_new() {
        let size = Size::new(100.0, 50.0);
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn test_size_area() {
        let size = Size::new(10.0, 5.0);
        assert_eq!(size.area(), 50.0);
    }

    #[test]
    fn test_size_default_entity() {
        let size = Size::default_entity();
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }

    // Rect tests
    #[test]
    fn test_rect_contains() {
        let rect = Rect::from_xywh(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(Position::new(50.0, 30.0)));
        assert!(!rect.contains(Position::new(5.0, 30.0)));
        assert!(!rect.contains(Position::new(150.0, 30.0)));
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::from_xywh(0.0, 0.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 25.0);
    }

    #[test]
    fn test_rect_corners() {
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.top_left(), Position::new(10.0, 20.0));
        assert_eq!(rect.top_right(), Position::new(110.0, 20.0));
        assert_eq!(rect.bottom_left(), Position::new(10.0, 70.0));
        assert_eq!(rect.bottom_right(), Position::new(110.0, 70.0));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::from_xywh(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::from_xywh(25.0, 25.0, 50.0, 50.0);
        let r3 = Rect::from_xywh(100.0, 100.0, 50.0, 50.0);
        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_rect_union() {
        let r1 = Rect::from_xywh(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::from_xywh(25.0, 25.0, 50.0, 50.0);
        let union = r1.union(&r2);
        assert_eq!(union.position.x, 0.0);
        assert_eq!(union.position.y, 0.0);
        assert_eq!(union.size.width, 75.0);
        assert_eq!(union.size.height, 75.0);
    }

    // DataType tests
    #[test]
    fn test_data_type_rust_type() {
        assert_eq!(DataType::String.to_rust_type(), "String");
        assert_eq!(DataType::Int32.to_rust_type(), "i32");
        assert_eq!(DataType::Bool.to_rust_type(), "bool");
        assert_eq!(DataType::Uuid.to_rust_type(), "uuid::Uuid");
    }

    #[test]
    fn test_data_type_optional() {
        let opt = DataType::Optional(Box::new(DataType::String));
        assert_eq!(opt.to_rust_type(), "Option<String>");
        assert!(opt.is_nullable());
        assert!(!DataType::String.is_nullable());
    }

    #[test]
    fn test_data_type_array() {
        let arr = DataType::Array(Box::new(DataType::Int32));
        assert_eq!(arr.to_rust_type(), "Vec<i32>");
    }

    #[test]
    fn test_data_type_reference() {
        let ref_type = DataType::Reference {
            entity_name: "User".to_string(),
            field_name: "id".to_string(),
        };
        assert!(ref_type.is_reference());
        assert!(!DataType::String.is_reference());
    }

    #[test]
    fn test_data_type_sql_postgres() {
        assert_eq!(DataType::String.to_postgres_type(), "VARCHAR(255)");
        assert_eq!(DataType::Text.to_postgres_type(), "TEXT");
        assert_eq!(DataType::Uuid.to_postgres_type(), "UUID");
        assert_eq!(DataType::Json.to_postgres_type(), "JSONB");
    }

    #[test]
    fn test_data_type_display() {
        assert_eq!(DataType::String.display_name(), "String");
        assert_eq!(DataType::DateTime.display_name(), "DateTime");
    }

    // DatabaseType tests
    #[test]
    fn test_database_type() {
        assert_eq!(DatabaseType::PostgreSQL.display_name(), "PostgreSQL");
        assert_eq!(DatabaseType::PostgreSQL.default_port(), 5432);
        assert_eq!(DatabaseType::MySQL.default_port(), 3306);
        assert_eq!(DatabaseType::SQLite.default_port(), 0);
    }

    // IdType tests
    #[test]
    fn test_id_type() {
        assert_eq!(IdType::Uuid.to_rust_type(), "uuid::Uuid");
        assert_eq!(IdType::Serial.to_rust_type(), "i64");
    }

    // RelationType tests
    #[test]
    fn test_relation_type() {
        assert_eq!(RelationType::OneToOne.display_name(), "One to One");
        assert_eq!(RelationType::OneToMany.arrow_symbol(), "1 ───< *");
    }

    #[test]
    fn test_relation_type_inverse() {
        assert_eq!(RelationType::OneToMany.inverse(), RelationType::ManyToOne);
        assert_eq!(RelationType::OneToOne.inverse(), RelationType::OneToOne);
    }

    #[test]
    fn test_relation_type_junction() {
        let m2m = RelationType::ManyToMany {
            junction_table: "post_tags".to_string(),
        };
        assert!(m2m.requires_junction_table());
        assert!(!RelationType::OneToMany.requires_junction_table());
    }

    // ReferentialAction tests
    #[test]
    fn test_referential_action() {
        assert_eq!(ReferentialAction::Cascade.to_sql(), "CASCADE");
        assert_eq!(ReferentialAction::SetNull.to_sql(), "SET NULL");
        assert_eq!(ReferentialAction::Restrict.to_sql(), "RESTRICT");
    }

    // Validation tests
    #[test]
    fn test_validation_error_message() {
        assert_eq!(
            Validation::Required.error_message(),
            "This field is required"
        );
        assert_eq!(
            Validation::MinLength(5).error_message(),
            "Minimum length is 5 characters"
        );
        assert_eq!(
            Validation::Email.error_message(),
            "Must be a valid email address"
        );
    }

    #[test]
    fn test_validation_to_validator() {
        assert_eq!(
            Validation::Email.to_validator_attribute(),
            Some("email".to_string())
        );
        assert_eq!(
            Validation::MinLength(3).to_validator_attribute(),
            Some("length(min = 3)".to_string())
        );
    }

    // ConfigValue tests
    #[test]
    fn test_config_value_from() {
        let v: ConfigValue = true.into();
        assert!(v.as_bool().unwrap());

        let v: ConfigValue = 42i64.into();
        assert_eq!(v.as_int(), Some(42));

        let v: ConfigValue = 3.14f64.into();
        assert!((v.as_float().unwrap() - 3.14).abs() < 0.001);

        let v: ConfigValue = "hello".into();
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_config_value_null() {
        let v = ConfigValue::Null;
        assert!(v.is_null());
        assert!(!v.is_bool());
        assert!(!v.is_numeric());
    }

    #[test]
    fn test_config_value_type_checks() {
        assert!(ConfigValue::Bool(true).is_bool());
        assert!(ConfigValue::Int(42).is_numeric());
        assert!(ConfigValue::Float(3.14).is_numeric());
        assert!(ConfigValue::String("test".to_string()).is_string());
        assert!(ConfigValue::Array(vec![]).is_array());
        assert!(ConfigValue::Object(HashMap::new()).is_object());
    }
}
