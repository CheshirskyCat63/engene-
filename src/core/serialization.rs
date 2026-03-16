use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataAuthority {
    Authoritative,
    Derived,
    DebugOnly,
}

#[derive(Clone, Debug)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn is_compatible(&self, other: &SchemaVersion) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug)]
pub struct FieldMigration {
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub description: String,
    pub kind: MigrationKind,
}

#[derive(Clone, Debug)]
pub enum MigrationKind {
    AddField { name: String, default_value: String },
    RemoveField { name: String },
    RenameField { old_name: String, new_name: String },
    TransformField { name: String, transform: String },
}

#[derive(Clone, Debug)]
pub struct TypeSchema {
    pub type_name: String,
    pub version: SchemaVersion,
    pub authority: DataAuthority,
    pub deterministic: bool,
    pub fields: Vec<FieldSchema>,
}

#[derive(Clone, Debug)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    pub authority: DataAuthority,
    pub deterministic: bool,
}

pub struct SerializationRegistry {
    schemas: HashMap<String, TypeSchema>,
    migrations: Vec<FieldMigration>,
}

impl SerializationRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            migrations: Vec::new(),
        }
    }

    pub fn register_schema(&mut self, schema: TypeSchema) {
        self.schemas.insert(schema.type_name.clone(), schema);
    }

    pub fn register_migration(&mut self, migration: FieldMigration) {
        self.migrations.push(migration);
    }

    pub fn get_schema(&self, type_name: &str) -> Option<&TypeSchema> {
        self.schemas.get(type_name)
    }

    pub fn authoritative_types(&self) -> Vec<&str> {
        self.schemas.values()
            .filter(|s| s.authority == DataAuthority::Authoritative)
            .map(|s| s.type_name.as_str())
            .collect()
    }

    pub fn deterministic_types(&self) -> Vec<&str> {
        self.schemas.values()
            .filter(|s| s.deterministic)
            .map(|s| s.type_name.as_str())
            .collect()
    }

    pub fn migrations_for(&self, _type_name: &str, from: &SchemaVersion) -> Vec<&FieldMigration> {
        self.migrations.iter()
            .filter(|m| {
                m.from_version.major == from.major
                    && m.from_version.minor >= from.minor
            })
            .collect()
    }

    pub fn validate_schemas(&self) -> Vec<String> {
        let mut issues = Vec::new();
        for schema in self.schemas.values() {
            if schema.deterministic && schema.authority != DataAuthority::Authoritative {
                issues.push(format!(
                    "'{}' is marked deterministic but not authoritative",
                    schema.type_name
                ));
            }
            for field in &schema.fields {
                if field.deterministic && !schema.deterministic {
                    issues.push(format!(
                        "'{}.{}' is deterministic but parent type is not",
                        schema.type_name, field.name
                    ));
                }
            }
        }
        issues
    }

    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
}

impl Default for SerializationRegistry {
    fn default() -> Self { Self::new() }
}
