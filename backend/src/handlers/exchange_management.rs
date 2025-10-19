use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use actix_session::SessionExt;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    exchange_connection::{
        self, ActiveModel as ExchangeConnectionActiveModel, Entity as ExchangeConnectionEntity,
        CreateExchangeConnectionRequest, UpdateExchangeConnectionRequest, ExchangeConnectionResponse,
        SupportedExchange,
    },
    user::Entity as UserEntity,
};
use crate::utils::{
    errors::AppError,
    encryption::{EncryptionService, EncryptedData},
};
use crate::exchange_connectors::{Exchange, ExchangeFactory, ExchangeCredentials};

/// Extract authenticated user ID from session
fn get_user_id_from_session(req: &HttpRequest) -> Result<Uuid, AppError> {
    let session = req.get_session();

    if let Ok(Some(user_id_str)) = session.get::<String>("user_id") {
        if let Ok(Some(authenticated)) = session.get::<bool>("authenticated") {
            if authenticated {
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    return Ok(user_id);
                }
            }
        }
    }

    Err(AppError::Unauthorized("Authentication required".to_string()))
}


/// Create a new exchange connection
pub async fn create_exchange_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateExchangeConnectionRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from session
    let user_id = get_user_id_from_session(&req)?;

    // Check if user exists in database
    let user_exists = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if user_exists.is_none() {
        tracing::error!("User with ID {} not found in database", user_id);
        return Err(AppError::Unauthorized("User not found. Please login again.".to_string()));
    }

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Validate supported exchange
    let _exchange = SupportedExchange::from_str(&body.exchange_name)
        .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

    // Check if user already has a connection for this exchange
    let existing_connection = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .filter(exchange_connection::Column::ExchangeName.eq(&body.exchange_name))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let is_updating_existing = existing_connection.is_some();

    // Test the connection before storing - validate API credentials
    let credentials = ExchangeCredentials {
        api_key: body.api_key.clone(),
        api_secret: body.api_secret.clone(),
    };

    let exchange = Exchange::from_str(&body.exchange_name)
        .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

    let connector = ExchangeFactory::create(exchange, credentials)
        .map_err(|e| AppError::BadRequest(format!("Failed to create connector: {}", e)))?;

    let connection_valid = connector.test_connection().await
        .map_err(|e| AppError::BadRequest(format!("Connection test failed: {}", e)))?;

    if !connection_valid {
        return Err(AppError::BadRequest("Invalid API credentials. Please check your API key and secret.".to_string()));
    }

    tracing::info!("API credentials validated successfully for exchange: {}", body.exchange_name);

    // Encrypt the API credentials
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_api_key = encryption_service
        .encrypt_api_credentials(&body.api_key, &body.password, &user_id_str)?;
    let encrypted_api_secret = encryption_service
        .encrypt_api_credentials(&body.api_secret, &body.password, &user_id_str)?;

    // Passphrase support removed - focusing on API key/secret only

    // Prepare connection data - use existing ID if updating, new ID if creating
    let (connection_id, created_at_timestamp) = if let Some(ref existing) = existing_connection {
        (existing.id, existing.created_at)
    } else {
        (Uuid::new_v4(), Utc::now())
    };

    if is_updating_existing {
        tracing::info!("Updating existing exchange connection: id={}, user_id={}, exchange={}",
                       connection_id, user_id, body.exchange_name);
    } else {
        tracing::info!("Creating new exchange connection: id={}, user_id={}, exchange={}",
                       connection_id, user_id, body.exchange_name);
    }

    // Store values before using them since we need them for both ActiveModel and potential manual construction
    let stored_id = connection_id;
    let stored_user_id = user_id;
    let stored_exchange_name = body.exchange_name.clone();
    let stored_display_name = body.display_name.clone();
    let stored_encrypted_api_key = encrypted_api_key.ciphertext.clone();
    let stored_encrypted_api_secret = encrypted_api_secret.ciphertext.clone();
    let stored_api_key_nonce = encrypted_api_key.nonce.clone();
    let stored_api_secret_nonce = encrypted_api_secret.nonce.clone();
    let stored_api_key_salt = encrypted_api_key.salt.clone();
    let stored_api_secret_salt = encrypted_api_secret.salt.clone();
    let stored_created_at = created_at_timestamp;
    let stored_updated_at = Utc::now();

    let connection_model = ExchangeConnectionActiveModel {
        id: Set(stored_id),
        user_id: Set(stored_user_id),
        exchange_name: Set(stored_exchange_name.clone()),
        display_name: Set(stored_display_name.clone()),
        encrypted_api_key: Set(stored_encrypted_api_key.clone()),
        encrypted_api_secret: Set(stored_encrypted_api_secret.clone()),
        encrypted_passphrase: Set(None),
        api_key_nonce: Set(stored_api_key_nonce.clone()),
        api_secret_nonce: Set(stored_api_secret_nonce.clone()),
        passphrase_nonce: Set(None),
        api_key_salt: Set(stored_api_key_salt.clone()),
        api_secret_salt: Set(stored_api_secret_salt.clone()),
        passphrase_salt: Set(None),
        is_active: Set(true),
        last_sync: Set(None),
        connection_status: Set("connected".to_string()),
        last_error: Set(None),
        created_at: Set(stored_created_at),
        updated_at: Set(stored_updated_at),
    };

    // Handle create vs update operations
    let connection = if is_updating_existing {
        // For updates, use save() and convert ActiveModel to Model
        tracing::info!("Updating existing exchange connection");
        let save_result = connection_model.save(db.get_ref()).await;
        
        match save_result {
            Ok(connection_active) => {
                tracing::info!("Exchange connection update successful");
                // Convert ActiveModel to Model for updates
                exchange_connection::Model {
                    id: connection_active.id.unwrap(),
                    user_id: connection_active.user_id.unwrap(),
                    exchange_name: connection_active.exchange_name.unwrap(),
                    display_name: connection_active.display_name.unwrap(),
                    encrypted_api_key: connection_active.encrypted_api_key.unwrap(),
                    encrypted_api_secret: connection_active.encrypted_api_secret.unwrap(),
                    encrypted_passphrase: connection_active.encrypted_passphrase.unwrap(),
                    api_key_nonce: connection_active.api_key_nonce.unwrap(),
                    api_secret_nonce: connection_active.api_secret_nonce.unwrap(),
                    passphrase_nonce: connection_active.passphrase_nonce.unwrap(),
                    api_key_salt: connection_active.api_key_salt.unwrap(),
                    api_secret_salt: connection_active.api_secret_salt.unwrap(),
                    passphrase_salt: connection_active.passphrase_salt.unwrap(),
                    is_active: connection_active.is_active.unwrap(),
                    last_sync: connection_active.last_sync.unwrap(),
                    connection_status: connection_active.connection_status.unwrap(),
                    last_error: connection_active.last_error.unwrap(),
                    created_at: connection_active.created_at.unwrap(),
                    updated_at: connection_active.updated_at.unwrap(),
                }
            },
            Err(e) => {
                tracing::error!("Failed to update exchange connection: {:?}", e);
                return Err(AppError::InternalServerError)
            }
        }
    } else {
        // For new connections, use insert() and handle UnpackInsertId
        tracing::info!("Creating new exchange connection");
        let insert_result = connection_model.insert(db.get_ref()).await;
        
        match insert_result {
            Ok(connection) => {
                tracing::info!("Exchange connection insert successful");
                connection
            },
            Err(e) => {
                tracing::error!("Failed to insert exchange connection: {:?}", e);
                let error_debug = format!("{:?}", e);
                if error_debug.contains("UnpackInsertId") {
                    tracing::info!("Insert succeeded but couldn't unpack ID, constructing connection manually");
                    // Construct connection manually from stored values
                    exchange_connection::Model {
                        id: stored_id,
                        user_id: stored_user_id,
                        exchange_name: stored_exchange_name,
                        display_name: stored_display_name,
                        encrypted_api_key: stored_encrypted_api_key,
                        encrypted_api_secret: stored_encrypted_api_secret,
                        encrypted_passphrase: None,
                        api_key_nonce: stored_api_key_nonce,
                        api_secret_nonce: stored_api_secret_nonce,
                        passphrase_nonce: None,
                        api_key_salt: stored_api_key_salt,
                        api_secret_salt: stored_api_secret_salt,
                        passphrase_salt: None,
                        is_active: true,
                        last_sync: None,
                        connection_status: "connected".to_string(),
                        last_error: None,
                        created_at: stored_created_at,
                        updated_at: stored_updated_at,
                    }
                } else {
                    tracing::error!("Actual database error, not UnpackInsertId: {}", error_debug);
                    return Err(AppError::InternalServerError)
                }
            }
        }
    };

    // Return appropriate status code based on operation
    let response = if is_updating_existing {
        HttpResponse::Ok().json(ExchangeConnectionResponse::from(connection))
    } else {
        HttpResponse::Created().json(ExchangeConnectionResponse::from(connection))
    };

    Ok(response)
}

/// Get all exchange connections for a user
pub async fn get_exchange_connections(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    eprintln!("HANDLER CALLED - get_exchange_connections");

    // Get user ID from session
    let user_id = get_user_id_from_session(&req)?;
    eprintln!("User ID: {}", user_id);

    // Query database for connections
    let connections = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .all(db.get_ref())
        .await
        .map_err(|e| {
            eprintln!("DB ERROR: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    eprintln!("Found {} connections", connections.len());

    // Convert to response format
    let responses: Vec<ExchangeConnectionResponse> = connections
        .into_iter()
        .map(ExchangeConnectionResponse::from)
        .collect();

    eprintln!("Converted to response format");

    // Return JSON response
    let response = serde_json::json!({
        "connections": responses
    });

    eprintln!("Returning HTTP response");
    Ok(HttpResponse::Ok().json(response))
}

/// Update an exchange connection
pub async fn update_exchange_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateExchangeConnectionRequest>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    body.validate().map_err(AppError::ValidationError)?;

    // Find the connection
    let connection = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::Id.eq(connection_id))
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    let mut active_model: ExchangeConnectionActiveModel = connection.into();

    // Update display name if provided
    if let Some(display_name) = &body.display_name {
        active_model.display_name = Set(display_name.clone());
    }

    // Update API credentials if provided
    if body.api_key.is_some() || body.api_secret.is_some() {
        let encryption_service = EncryptionService::new();
        let user_id_str = user_id.to_string();

        if let Some(api_key) = &body.api_key {
            let encrypted_api_key = encryption_service
                .encrypt_api_credentials(api_key, &body.password, &user_id_str)?;

            active_model.encrypted_api_key = Set(encrypted_api_key.ciphertext);
            active_model.api_key_nonce = Set(encrypted_api_key.nonce);
            active_model.api_key_salt = Set(encrypted_api_key.salt);
        }

        if let Some(api_secret) = &body.api_secret {
            let encrypted_api_secret = encryption_service
                .encrypt_api_credentials(api_secret, &body.password, &user_id_str)?;

            active_model.encrypted_api_secret = Set(encrypted_api_secret.ciphertext);
            active_model.api_secret_nonce = Set(encrypted_api_secret.nonce);
            active_model.api_secret_salt = Set(encrypted_api_secret.salt);
        }

        // Reset connection status to pending for re-validation
        active_model.connection_status = Set("pending".to_string());
        active_model.last_error = Set(None);
    }

    active_model.updated_at = Set(Utc::now());

    let updated_connection = active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(ExchangeConnectionResponse::from(updated_connection)))
}

/// Delete an exchange connection
pub async fn delete_exchange_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    // Verify the connection belongs to the user
    let connection = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::Id.eq(connection_id))
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // Delete the connection (this will cascade delete wallet balances)
    ExchangeConnectionEntity::delete_by_id(connection.id)
        .exec(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Exchange connection deleted successfully"
    })))
}

/// Sync balances for a specific exchange connection
pub async fn sync_exchange_balances(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>, // Expecting { "password": "user_password" }
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    let password = body.get("password")
        .and_then(|p| p.as_str())
        .ok_or_else(|| AppError::BadRequest("Password is required".to_string()))?;

    // Find the connection
    let connection = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::Id.eq(connection_id))
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // Decrypt the API credentials
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_api_key = EncryptedData {
        ciphertext: connection.encrypted_api_key.clone(),
        nonce: connection.api_key_nonce.clone(),
        salt: connection.api_key_salt.clone(),
    };

    let encrypted_api_secret = EncryptedData {
        ciphertext: connection.encrypted_api_secret.clone(),
        nonce: connection.api_secret_nonce.clone(),
        salt: connection.api_secret_salt.clone(),
    };

    let api_key = encryption_service
        .decrypt_api_credentials(&encrypted_api_key, password, &user_id_str)?;
    let api_secret = encryption_service
        .decrypt_api_credentials(&encrypted_api_secret, password, &user_id_str)?;

    let credentials = ExchangeCredentials {
        api_key,
        api_secret,
    };

    let exchange = Exchange::from_str(&connection.exchange_name)
        .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

    let connector = ExchangeFactory::create(exchange, credentials)
        .map_err(|e| AppError::BadRequest(format!("Failed to create connector: {}", e)))?;

    // Fetch live balance data from the exchange (no database storage)
    let account_balances = connector.get_all_balances().await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to fetch balances: {}", e)))?;

    // Debug logging to see account balances
    tracing::info!("=== ACCOUNT BALANCES DEBUG ===");
    tracing::info!("Total USD Value: ${}", account_balances.total_usd_value);
    tracing::info!("Total BTC Value: {}", account_balances.total_btc_value);

    if let Some(ref spot) = account_balances.spot {
        tracing::info!("SPOT ACCOUNT:");
        tracing::info!("  Spot USD Value: ${:?}", spot.total_usd_value);
        tracing::info!("  Spot Balance Count: {}", spot.balances.len());
        for balance in &spot.balances {
            if balance.total > rust_decimal::Decimal::ZERO {
                tracing::info!("    {}: free={}, locked={}, total={}, usd_value=${:?}",
                    balance.asset, balance.free, balance.locked, balance.total, balance.usd_value);
            }
        }
    }

    if let Some(ref margin) = account_balances.margin {
        tracing::info!("MARGIN ACCOUNT:");
        tracing::info!("  Total Asset Value: ${}", margin.total_asset_value);
        tracing::info!("  Total Liability Value: ${}", margin.total_liability_value);
        tracing::info!("  Total Net Value: ${}", margin.total_net_value);
        tracing::info!("  Margin Balance Count: {}", margin.balances.len());
        for balance in &margin.balances {
            if balance.total > rust_decimal::Decimal::ZERO {
                tracing::info!("    {}: free={}, locked={}, total={}, usd_value=${:?}",
                    balance.asset, balance.free, balance.locked, balance.total, balance.usd_value);
            }
        }
    }


    if let Some(ref futures) = account_balances.futures_usdm {
        tracing::info!("FUTURES USD-M ACCOUNT:");
        tracing::info!("  Total Margin Balance: ${}", futures.total_margin_balance);
        tracing::info!("  Available Balance: ${}", futures.available_balance);
        tracing::info!("  Futures USD-M Balance Count: {}", futures.balances.len());
        for balance in &futures.balances {
            if balance.total > rust_decimal::Decimal::ZERO {
                tracing::info!("    {}: free={}, locked={}, total={}, usd_value=${:?}",
                    balance.asset, balance.free, balance.locked, balance.total, balance.usd_value);
            }
        }
    }

    if let Some(ref futures) = account_balances.futures_coinm {
        tracing::info!("FUTURES COIN-M ACCOUNT:");
        tracing::info!("  Total Margin Balance: ${}", futures.total_margin_balance);
        tracing::info!("  Available Balance: ${}", futures.available_balance);
        tracing::info!("  Futures COIN-M Balance Count: {}", futures.balances.len());
        for balance in &futures.balances {
            if balance.total > rust_decimal::Decimal::ZERO {
                tracing::info!("    {}: free={}, locked={}, total={}, usd_value=${:?}",
                    balance.asset, balance.free, balance.locked, balance.total, balance.usd_value);
            }
        }
    }


    tracing::info!("=== END ACCOUNT BALANCES DEBUG ===");

    // Store connection fields before moving
    let exchange_name = connection.exchange_name.clone();
    let display_name = connection.display_name.clone();

    // Update connection last sync time
    let now = Utc::now();
    let mut connection_update: ExchangeConnectionActiveModel = connection.into();
    connection_update.last_sync = Set(Some(now));
    connection_update.connection_status = Set("connected".to_string());
    connection_update.last_error = Set(None);
    connection_update.updated_at = Set(now);

    connection_update.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    // Note: We don't store balance data in database - all balance data is fetched live

    // Return live balance data
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "connection_id": connection_id,
        "exchange_name": exchange_name,
        "display_name": display_name,
        "total_usd_value": account_balances.total_usd_value.to_string(),
        "total_btc_value": account_balances.total_btc_value.to_string(),
        "accounts": {
            "spot": account_balances.spot,
            "margin": account_balances.margin,
            "futures_usdm": account_balances.futures_usdm,
            "futures_coinm": account_balances.futures_coinm,
        },
        "last_updated": now,
        "is_live": true
    })))
}


/// Get LIVE wallet balances directly from exchange (no database storage)
pub async fn get_live_wallet_balances(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>, // Expecting { "password": "user_password" }
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    let password = body.get("password")
        .and_then(|p| p.as_str())
        .ok_or_else(|| AppError::BadRequest("Password is required".to_string()))?;

    // Find the connection
    let connection = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::Id.eq(connection_id))
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // Decrypt the API credentials
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_api_key = EncryptedData {
        ciphertext: connection.encrypted_api_key.clone(),
        nonce: connection.api_key_nonce.clone(),
        salt: connection.api_key_salt.clone(),
    };

    let encrypted_api_secret = EncryptedData {
        ciphertext: connection.encrypted_api_secret.clone(),
        nonce: connection.api_secret_nonce.clone(),
        salt: connection.api_secret_salt.clone(),
    };

    let api_key = encryption_service
        .decrypt_api_credentials(&encrypted_api_key, password, &user_id_str)?;
    let api_secret = encryption_service
        .decrypt_api_credentials(&encrypted_api_secret, password, &user_id_str)?;

    let credentials = ExchangeCredentials {
        api_key,
        api_secret,
    };

    let exchange = Exchange::from_str(&connection.exchange_name)
        .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

    let connector = ExchangeFactory::create(exchange, credentials)
        .map_err(|e| AppError::BadRequest(format!("Failed to create connector: {}", e)))?;

    // Fetch live balance data from the exchange (no database storage)
    let account_balances = connector.get_all_balances().await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to fetch live balances: {}", e)))?;

    // Debug logging for live balance fetch
    tracing::info!("=== LIVE BALANCE FETCH DEBUG ===");
    tracing::info!("Connection: {} ({})", connection.display_name, connection.exchange_name);
    tracing::info!("Total USD Value: ${}", account_balances.total_usd_value);
    if let Some(ref spot) = account_balances.spot {
        tracing::info!("Spot balances found: {} assets", spot.balances.len());
    }
    if let Some(ref margin) = account_balances.margin {
        tracing::info!("Margin balances found: {} assets", margin.balances.len());
    }
    if let Some(ref futures_usdm) = account_balances.futures_usdm {
        tracing::info!("Futures USD-M balances found: {} assets", futures_usdm.balances.len());
    }
    if let Some(ref futures_coinm) = account_balances.futures_coinm {
        tracing::info!("Futures COIN-M balances found: {} assets", futures_coinm.balances.len());
    }
    tracing::info!("=== END LIVE BALANCE FETCH DEBUG ===");

    // Convert to response format
    let response = serde_json::json!({
        "exchange_connection_id": connection_id,
        "exchange_name": connection.exchange_name,
        "display_name": connection.display_name,
        "total_usd_value": account_balances.total_usd_value.to_string(),
        "total_btc_value": account_balances.total_btc_value.to_string(),
        "accounts": {
            "spot": account_balances.spot,
            "margin": account_balances.margin,
            "futures_usdm": account_balances.futures_usdm,
            "futures_coinm": account_balances.futures_coinm,
        },
        "last_updated": chrono::Utc::now(),
        "is_live": true
    });

    Ok(HttpResponse::Ok().json(response))
}


/// Get LIVE balances from all user's exchange connections (requires password)
pub async fn get_all_live_user_balances(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>, // Expecting { "password": "user_password" }
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    let password = body.get("password")
        .and_then(|p| p.as_str())
        .ok_or_else(|| AppError::BadRequest("Password is required for live balance fetching".to_string()))?;

    let connections = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .filter(exchange_connection::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut all_summaries = Vec::new();
    let mut grand_total_usd = rust_decimal::Decimal::ZERO;

    for connection in connections {
        let connection_id = connection.id;
        
        // Try to fetch live balances for this connection
        match get_live_balances_for_connection(&connection, password, user_id).await {
            Ok(account_balances) => {
                grand_total_usd += account_balances.total_usd_value;
                
                all_summaries.push(serde_json::json!({
                    "exchange_connection_id": connection_id,
                    "exchange_name": connection.exchange_name,
                    "display_name": connection.display_name,
                    "total_usd_value": account_balances.total_usd_value.to_string(),
                    "accounts": {
                        "spot": account_balances.spot,
                        "margin": account_balances.margin,
                        "futures_usdm": account_balances.futures_usdm,
                        "futures_coinm": account_balances.futures_coinm,
                    },
                    "status": "connected",
                    "last_updated": chrono::Utc::now(),
                    "is_live": true
                }));
            }
            Err(e) => {
                tracing::error!("Failed to fetch live balances for {}: {:?}", connection.display_name, e);
                all_summaries.push(serde_json::json!({
                    "exchange_connection_id": connection_id,
                    "exchange_name": connection.exchange_name,
                    "display_name": connection.display_name,
                    "total_usd_value": "0",
                    "accounts": null,
                    "status": "error",
                    "error": format!("{:?}", e),
                    "last_updated": chrono::Utc::now(),
                    "is_live": false
                }));
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balances": all_summaries,
        "total_usd_value": grand_total_usd.to_string(),
        "is_live": true,
        "last_updated": chrono::Utc::now()
    })))
}

// Helper function to get live balances for a specific connection
async fn get_live_balances_for_connection(
    connection: &crate::models::exchange_connection::Model,
    password: &str,
    user_id: Uuid,
) -> Result<crate::exchange_connectors::common_types::AccountBalances, AppError> {
    // Decrypt the API credentials
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_api_key = EncryptedData {
        ciphertext: connection.encrypted_api_key.clone(),
        nonce: connection.api_key_nonce.clone(),
        salt: connection.api_key_salt.clone(),
    };

    let encrypted_api_secret = EncryptedData {
        ciphertext: connection.encrypted_api_secret.clone(),
        nonce: connection.api_secret_nonce.clone(),
        salt: connection.api_secret_salt.clone(),
    };

    let api_key = encryption_service
        .decrypt_api_credentials(&encrypted_api_key, password, &user_id_str)
        .map_err(|e| AppError::BadRequest(format!("Decryption failed - wrong password? Error: {:?}", e)))?;
    let api_secret = encryption_service
        .decrypt_api_credentials(&encrypted_api_secret, password, &user_id_str)
        .map_err(|e| AppError::BadRequest(format!("Decryption failed - wrong password? Error: {:?}", e)))?;

    let credentials = ExchangeCredentials {
        api_key,
        api_secret,
    };

    let exchange = Exchange::from_str(&connection.exchange_name)
        .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

    let connector = ExchangeFactory::create(exchange, credentials)
        .map_err(|e| AppError::BadRequest(format!("Failed to create connector: {}", e)))?;

    // Fetch live balance data from the exchange
    connector.get_all_balances().await
        .map_err(|e| AppError::ExternalServiceError(format!("Binance API Error: {}", e)))
}

/// Get specific account type data (spot/margin/futures) - replacement for exchange_connector_handler
pub async fn get_account_by_type(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let account_type = path.into_inner();
    
    // Get user_id from session or query parameters for non-authenticated access
    let user_id = if let Ok(user_id) = get_user_id_from_session(&req) {
        user_id
    } else {
        // Try to get from query parameters
        let user_id_str = query.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("user_id parameter required".to_string()))?;

        Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?
    };

    let connections = ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .filter(exchange_connection::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let responses: Vec<serde_json::Value> = connections
        .into_iter()
        .map(|connection| {
            serde_json::json!({
                "connection_id": connection.id,
                "exchange_name": connection.exchange_name,
                "display_name": connection.display_name,
                "account_type": account_type,
                "status": "requires_sync",
                "message": format!("Please sync this connection to view {} account data", account_type),
                "last_update": connection.updated_at
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Test exchange connection - replacement for exchange_connector_handler
pub async fn test_connection_status(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = query.get("connection_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("connection_id parameter required".to_string()))?;

    let connection_id = Uuid::parse_str(connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    // Get user_id from session or query
    let user_id = if let Ok(user_id) = get_user_id_from_session(&req) {
        user_id
    } else {
        let user_id_str = query.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("user_id parameter required".to_string()))?;

        Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?
    };

    let connection = ExchangeConnectionEntity::find_by_id(connection_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // Verify connection belongs to user
    if connection.user_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "connection_id": connection_id,
        "exchange_name": connection.exchange_name,
        "display_name": connection.display_name,
        "status": connection.connection_status,
        "is_active": connection.is_active,
        "last_sync": connection.last_sync,
        "last_error": connection.last_error,
        "message": "Use the sync endpoint with your password to test live connection",
        "tested_at": chrono::Utc::now()
    })))
}
