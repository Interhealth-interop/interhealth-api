use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use mongodb::bson::{doc, oid::ObjectId};
use crate::utils::AppError;
use crate::infrastructure::repositories::{
    CompanyRepository, UserRepository, DatabaseTableRepository, DatabaseColumnRepository,
    DatabaseViewMappingRepository, DatabaseTransformationRepository, DatabaseModelRepository,
    DatabaseModelValueRepository,
    DatabaseConfigurationRepository, DatabaseViewRepository,
    CreateCompanyDto,
};
use crate::domain::entities::{DatabaseModelValue, DatabaseModelValueClient, FieldMapping, ValueMappingItem};
use crate::utils::AppResult;

fn parse_seed_file<T: DeserializeOwned>(file_name: &str, content: &str) -> AppResult<Option<T>> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    serde_json::from_str::<T>(trimmed)
        .map(Some)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse seed file '{}': {}", file_name, e)))
}

#[derive(Debug, Serialize, Deserialize)]
struct CompanySeedFile {
    company: CompanyData,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserSeedFile {
    user: Vec<UserData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseViewMappingSeedFile {
    #[serde(default)]
    database_view_mapping: Vec<DatabaseViewMappingData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseModelSeedFile {
    #[serde(default)]
    database_model: Vec<DatabaseModelData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseModelValueSeedFile {
    #[serde(default)]
    data: Vec<DatabaseModelValueData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseTransformationSeedFile {
    #[serde(default)]
    database_transformation: Vec<DatabaseTransformationData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseTablesSeedFile {
    #[serde(default)]
    database_tables: Vec<DatabaseTableData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConnectionSeedFile {
    database_connection: DatabaseConnectionData,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseIntegrationSeedFile {
    #[serde(default)]
    database_integration: Vec<DatabaseIntegrationData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompanyData {
    #[serde(default)]
    id: Option<String>,
    code: String,
    name: String,
    email: String,
    phone: String,
    address: String,
    city: String,
    state: String,
    zipcode: String,
    country: String,
    cnpj: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConnectionData {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(rename = "type")]
    db_type: String,
    version: Option<String>,
    host: String,
    port: i32,
    database: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseIntegrationData {
    #[serde(default)]
    id: Option<String>,
    name: String,
    description: String,
    #[serde(rename = "entityType")]
    entity_type: String,
    #[serde(rename = "databaseConfigurationId")]
    database_configuration_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "companyId")]
    company_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    name: String,
    email: String,
    password: String,
    primary_document: String,
    #[serde(rename = "type")]
    user_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseTableData {
    name: String,
    description: String,
    table_reference: String,
    table_type: String,
    entity_type: String,
    columns: Vec<ColumnData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ColumnData {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "dataType")]
    data_type: String,
    #[serde(rename = "isPrimaryKey")]
    is_primary_key: bool,
    #[serde(rename = "isNullable")]
    is_nullable: bool,
    description: String,
    #[serde(rename = "maxLength")]
    max_length: Option<i32>,
    #[serde(rename = "minLength")]
    min_length: Option<i32>,
    #[serde(rename = "isForeignKey")]
    is_foreign_key: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseViewMappingData {
    name: String,
    description: String,
    #[serde(rename = "originEntityType")]
    origin_entity_type: String,
    #[serde(rename = "originTableReference")]
    origin_table_reference: String,
    #[serde(rename = "destinyEntityType")]
    destiny_entity_type: String,
    #[serde(rename = "destinyTableReference")]
    destiny_table_reference: String,
    #[serde(rename = "databaseTableOriginId")]
    database_table_origin_id: Option<String>,
    #[serde(rename = "databaseTableDestinyId")]
    database_table_destiny_id: Option<String>,
    #[serde(rename = "dataViewId")]
    data_view_id: Option<String>,
    #[serde(rename = "fieldMappings")]
    field_mappings: Vec<FieldMappingData>,
    #[serde(default = "default_status")]
    status: String,
}

fn default_status() -> String {
    "draft".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct FieldMappingData {
    #[serde(rename = "fieldOrigin")]
    field_origin: String,
    #[serde(rename = "fieldDestiny")]
    field_destiny: String,
    #[serde(rename = "referenceDestiny", skip_serializing_if = "Option::is_none")]
    reference_destiny: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "relationshipDestiny", skip_serializing_if = "Option::is_none")]
    relationship_destiny: Option<String>,
    #[serde(rename = "dataType")]
    data_type: String,
    #[serde(rename = "isNullable")]
    is_nullable: bool,
    #[serde(rename = "minLength")]
    min_length: Option<i32>,
    #[serde(rename = "maxLength")]
    max_length: Option<i32>,
    #[serde(rename = "isEnumerable")]
    is_enumerable: Option<bool>,
    #[serde(rename = "transformationId")]
    transformation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseTransformationData {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_field: String,
    value_mappings: HashMap<String, ValueMappingItemData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValueMappingItemData {
    code: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseModelData {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_field: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseModelValueData {
    id: String,
    #[serde(default)]
    owner_id: Option<String>,
    #[serde(rename = "type")]
    type_field: String,
    code: String,
    description: String,
    #[serde(default)]
    clients: Vec<ClientMappingData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientMappingData {
    source_key: String,
    source_description: String,
    #[serde(default)]
    status: Option<String>,
    company_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelValueData {
    code: String,
    description: String,
}

pub async fn seed_database(
    company_repo: Arc<CompanyRepository>,
    user_repo: Arc<UserRepository>,
    database_configuration_repo: Arc<DatabaseConfigurationRepository>,
    database_view_repo: Arc<DatabaseViewRepository>,
    table_repo: Arc<DatabaseTableRepository>,
    column_repo: Arc<DatabaseColumnRepository>,
    view_mapping_repo: Arc<DatabaseViewMappingRepository>,
    model_repo: Arc<DatabaseModelRepository>,
    transformation_repo: Arc<DatabaseTransformationRepository>,
    database_model_value_repo: Arc<DatabaseModelValueRepository>,
) -> AppResult<()> {
    println!("🌱 Starting database seeding...");

    let company_json = include_str!("./tables/company.json");
    let user_json = include_str!("./tables/user.json");
    let database_connection_json = include_str!("./tables/database_connection.json");
    let database_integration_json = include_str!("./tables/database_integration.json");
    let tables_json = include_str!("./tables/database_tables.json");
    let view_mapping_json = include_str!("./tables/database_view_mapping.json");
    let model_json = include_str!("./tables/database_model.json");
    let model_value_json = include_str!("./tables/database_model_value.json");

    let company_seed = parse_seed_file::<CompanySeedFile>("tables/company.json", company_json)?
        .ok_or_else(|| AppError::BadRequest("Seed file 'tables/company.json' is empty. It must contain a 'company' object.".to_string()))?;

    let users = parse_seed_file::<UserSeedFile>("tables/user.json", user_json)?
        .map(|d| d.user)
        .unwrap_or_default();

    let database_view_mappings = parse_seed_file::<DatabaseViewMappingSeedFile>(
        "tables/database_view_mapping.json",
        view_mapping_json,
    )?
    .map(|d| d.database_view_mapping)
    .unwrap_or_default();

    let database_models = parse_seed_file::<DatabaseModelSeedFile>("tables/database_model.json", model_json)?
        .map(|d| d.database_model)
        .unwrap_or_default();

    let database_model_values = parse_seed_file::<DatabaseModelValueSeedFile>(
        "tables/database_model_value.json",
        model_value_json,
    )?
    .map(|d| d.data)
    .unwrap_or_default();

    let database_tables = parse_seed_file::<DatabaseTablesSeedFile>("tables/database_tables.json", tables_json)?
        .map(|d| d.database_tables)
        .unwrap_or_default();

    let database_connection = parse_seed_file::<DatabaseConnectionSeedFile>(
        "tables/database_connection.json",
        database_connection_json,
    )?
    .map(|d| d.database_connection);

    let database_integrations = parse_seed_file::<DatabaseIntegrationSeedFile>(
        "tables/database_integration.json",
        database_integration_json,
    )?
    .map(|d| d.database_integration)
    .unwrap_or_default();

    let existing_company = if let Some(id) = company_seed.company.id.as_deref() {
        company_repo.find_by_id(id).await?
    } else {
        None
    }
    .or(company_repo.find_by_code(&company_seed.company.code).await?);

    let company_id = if let Some(company) = existing_company {
        println!("✓ Company '{}' already exists, skipping...", company_seed.company.name);
        company
            .id
            .ok_or_else(|| AppError::Database("Existing company has no id".to_string()))?
            .to_string()
    } else {
        println!("📦 Creating company: {}", company_seed.company.name);

        let dto = CreateCompanyDto {
            code: company_seed.company.code,
            name: company_seed.company.name,
            cnpj: company_seed.company.cnpj,
            address: Some(company_seed.company.address),
            number: None,
            phone: Some(company_seed.company.phone),
            email: Some(company_seed.company.email),
            city: Some(company_seed.company.city),
            state: Some(company_seed.company.state),
            zipcode: Some(company_seed.company.zipcode),
            country: Some(company_seed.company.country),
        };

        let company = if let Some(id) = company_seed.company.id.as_deref() {
            company_repo.create_with_id(id, dto).await?
        } else {
            company_repo.create(dto).await?
        };

        println!("  ✓ Company created successfully");
        company
            .id
            .ok_or_else(|| AppError::Database("Created company has no id".to_string()))?
            .to_string()
    };

    println!("\n🔌 Seeding database connection...");
    let database_configuration_id = if let Some(conn) = database_connection {
        let existing = if let Some(id) = conn.id.as_deref() {
            database_configuration_repo.find_by_id(id).await?
        } else {
            None
        }
        .or(
            database_configuration_repo
                .find_by_name_and_company_id(&conn.name, &company_id)
                .await?,
        );

        if let Some(existing) = existing {
            println!("  ✓ Database connection '{}' already exists, skipping...", conn.name);
            existing
                .id
                .ok_or_else(|| AppError::Database("Existing database configuration has no id".to_string()))?
                .to_string()
        } else {
            let created = if let Some(id) = conn.id.as_deref() {
                database_configuration_repo
                    .create_with_id(
                        id,
                        conn.name.clone(),
                        conn.db_type,
                        conn.version,
                        conn.host,
                        conn.port,
                        conn.database,
                        conn.username,
                        conn.password,
                        company_id.clone(),
                    )
                    .await?
            } else {
                database_configuration_repo
                    .create(
                        conn.name.clone(),
                        conn.db_type,
                        conn.version,
                        conn.host,
                        conn.port,
                        conn.database,
                        conn.username,
                        conn.password,
                        company_id.clone(),
                    )
                    .await?
            };

            println!("  ✓ Database connection '{}' created", conn.name);
            created
                .id
                .ok_or_else(|| AppError::Database("Created database configuration has no id".to_string()))?
                .to_string()
        }
    } else {
        String::new()
    };

    println!("\n🧩 Seeding database integration...");
    for integration in database_integrations {
        let existing = if let Some(id) = integration.id.as_deref() {
            database_view_repo.find_by_id(id).await?
        } else {
            None
        }
        .or(
            database_view_repo
                .find_by_name_and_company_id(&integration.name, &company_id)
                .await?,
        );

        if existing.is_some() {
            println!("  ✓ Database integration '{}' already exists, skipping...", integration.name);
            continue;
        }

        let config_id = integration
            .database_configuration_id
            .clone()
            .unwrap_or_else(|| database_configuration_id.clone());

        if config_id.is_empty() {
            return Err(AppError::BadRequest(
                "database_integration requires a databaseConfigurationId (or provide database_connection seed)".to_string(),
            ));
        }

        let created = if let Some(id) = integration.id.as_deref() {
            database_view_repo
                .create_with_id(
                    id,
                    integration.name.clone(),
                    integration.description,
                    integration.entity_type,
                    None,
                    None,
                    config_id,
                    company_id.clone(),
                    None,
                )
                .await?
        } else {
            database_view_repo
                .create(
                    integration.name.clone(),
                    integration.description,
                    integration.entity_type,
                    None,
                    None,
                    config_id,
                    company_id.clone(),
                    None,
                )
                .await?
        };

        let _ = created;
        println!("  ✓ Database integration '{}' created", integration.name);
    }

    println!("\n👥 Seeding users...");
    for user_data in users {
        let existing_user = user_repo.find_by_email(&user_data.email).await?;
        if existing_user.is_some() {
            println!("  ✓ User '{}' already exists, skipping...", user_data.email);
            continue;
        }

        user_repo.create(crate::infrastructure::repositories::user::CreateUserDto {
            name: user_data.name.clone(),
            email: user_data.email.clone(),
            password: user_data.password,
            user_type: user_data.user_type,
            primary_document: Some(user_data.primary_document),
            status: true,
            company_id: Some(company_id.clone()),
        }).await?;
        println!("  ✓ User '{}' created", user_data.name);
    }

    println!("\n📊 Seeding database tables...");
    let mut table_count = 0;
    let mut column_count = 0;

    for table_data in database_tables {
        let filter = doc! {
            "entity_type": &table_data.entity_type,
            "table_reference": &table_data.table_reference,
            "company_id": &company_id,
        };

        let existing_table = table_repo.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let table_id = if let Some(table) = existing_table {
            println!("  ✓ Table '{}' already exists, skipping create...", table_data.name);
            table.id
                .ok_or_else(|| AppError::Database("Existing database table has no id".to_string()))?
                .to_string()
        } else {
            let table = table_repo.create(
                table_data.name.clone(),
                table_data.description,
                Some(table_data.table_reference.clone()),
                Some(table_data.table_type.clone()),
                table_data.entity_type.clone(),
                company_id.clone(),
            ).await?;

            table_count += 1;
            println!("  ✓ Table '{}' created", table_data.name);
            table.id
                .ok_or_else(|| AppError::Database("Created database table has no id".to_string()))?
                .to_string()
        };

        let existing_columns = column_repo.find_by_table_id(&table_id).await?;

        for column_data in table_data.columns {
            if existing_columns.iter().any(|c| c.name == column_data.name) {
                continue;
            }

            column_repo.create(
                column_data.name,
                column_data.reference,
                column_data.data_type,
                column_data.is_nullable,
                column_data.is_primary_key,
                column_data.is_foreign_key.unwrap_or(false),
                column_data.description,
                column_data.max_length,
                column_data.min_length,
                table_id.clone(),
                company_id.clone(),
            ).await?;

            column_count += 1;
        }
    }
    
    println!("  ✓ Created {} tables with {} columns", table_count, column_count);

    println!("\n🔗 Seeding database view mappings...");
    let mut mapping_count = 0;

    for mapping_data in database_view_mappings {
        if view_mapping_repo.find_by_name(&mapping_data.name).await?.is_some() {
            println!("  ✓ Mapping '{}' already exists, skipping...", mapping_data.name);
            continue;
        }

        let field_mappings: Vec<FieldMapping> = mapping_data.field_mappings.into_iter().map(|fm| {
            FieldMapping {
                field_origin: fm.field_origin,
                field_destiny: fm.field_destiny,
                reference_destiny: fm.reference_destiny,
                relationship_destiny: fm.relationship_destiny,
                data_type: fm.data_type,
                is_nullable: fm.is_nullable,
                min_length: fm.min_length.unwrap_or(0),
                max_length: fm.max_length.unwrap_or(255),
                is_enumerable: fm.is_enumerable.unwrap_or(false),
                transformation_id: fm.transformation_id,
                reference: fm.reference,
            }
        }).collect();

        let origin_id = if let Some(id) = mapping_data.database_table_origin_id {
            id
        } else {
            let filter = doc! {
                "entity_type": &mapping_data.origin_entity_type,
                "table_reference": &mapping_data.origin_table_reference,
                "company_id": &company_id,
            };
            let table = table_repo.collection.find_one(filter, None).await
                .map_err(|e| AppError::Database(e.to_string()))?
                .ok_or_else(|| AppError::NotFound(format!("Origin table not found for entity_type: {} with table_reference: {}", mapping_data.origin_entity_type, mapping_data.origin_table_reference)))?;
            table.id.unwrap().to_string()
        };

        let destiny_id = if let Some(id) = mapping_data.database_table_destiny_id {
            id
        } else {
            let filter = doc! {
                "entity_type": &mapping_data.destiny_entity_type,
                "table_reference": &mapping_data.destiny_table_reference,
                "company_id": &company_id,
            };
            let table = table_repo.collection.find_one(filter, None).await
                .map_err(|e| AppError::Database(e.to_string()))?
                .ok_or_else(|| AppError::NotFound(format!("Destiny table not found for entity_type: {} with table_reference: {}", mapping_data.destiny_entity_type, mapping_data.destiny_table_reference)))?;
            table.id.unwrap().to_string()
        };

        view_mapping_repo.create(
            mapping_data.name.clone(),
            mapping_data.description,
            mapping_data.origin_entity_type,
            origin_id,
            destiny_id,
            mapping_data.data_view_id.unwrap_or_default(),
            field_mappings,
            mapping_data.status,
        ).await?;
        
        mapping_count += 1;
        println!("  ✓ Mapping '{}' created", mapping_data.name);
    }
    
    println!("  ✓ Created {} view mappings", mapping_count);

    println!("\n📚 Seeding database models...");
    let mut model_count = 0;

    for model_data in database_models {
        if model_repo.find_by_id(&model_data.id).await?.is_some() {
            println!("  ✓ Model '{}' already exists, skipping...", model_data.id);
            continue;
        }

        let values: Vec<crate::domain::entities::ModelValue> = Vec::new();

        model_repo.create_with_id(
            Some(model_data.id.clone()),
            model_data.name.clone(),
            model_data.type_field,
            model_data.description,
            values,
        ).await?;
        
        model_count += 1;
        println!("  ✓ Model '{}' created with ID: {}", model_data.name, model_data.id);
    }
    
    println!("  ✓ Created {} models", model_count);

    println!("\n🧩 Seeding database model values (database_model_values)...");
    let mut model_value_count = 0;
    for mv in database_model_values {
        let Some(owner_id) = mv.owner_id.as_deref() else {
            continue;
        };

        // Persist nested database_model_values document (as requested)
        let id = ObjectId::parse_str(&mv.id)
            .map_err(|_| AppError::BadRequest("Invalid database_model_value id format".to_string()))?;
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;

        let clients: Vec<DatabaseModelValueClient> = mv
            .clients
            .iter()
            .map(|c| {
                let company_object_id = ObjectId::parse_str(&c.company_id)
                    .map_err(|_| AppError::BadRequest("Invalid company_id format".to_string()))?;
                Ok(DatabaseModelValueClient {
                    source_key: c.source_key.clone(),
                    source_description: c.source_description.clone(),
                    status: c.status.clone().unwrap_or_else(|| "pending".to_string()),
                    company_id: company_object_id,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        database_model_value_repo
            .upsert_with_id(DatabaseModelValue {
                id: Some(id),
                owner_id: owner_object_id,
                type_field: mv.type_field.clone(),
                code: mv.code.clone(),
                description: mv.description.clone(),
                clients,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .await?;

        model_value_count += 1;
    }
    println!("  ✓ Seeded {} model values", model_value_count);

    println!("\n✅ Database seeding completed successfully!");
    Ok(())
}
