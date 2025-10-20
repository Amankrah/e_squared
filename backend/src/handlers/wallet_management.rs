use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use actix_session::SessionExt;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    wallet_connection::{
        self, ActiveModel as WalletConnectionActiveModel, Entity as WalletConnectionEntity,
        CreateWalletConnectionRequest, UpdateWalletConnectionRequest, WalletConnectionResponse,
        BlockchainNetwork,
    },
    user::Entity as UserEntity,
};
use crate::utils::{
    errors::AppError,
    encryption::{EncryptionService, EncryptedData},
};

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

/// Derive wallet address from private key based on blockchain network
fn derive_wallet_address(blockchain: &BlockchainNetwork, private_key: &str) -> Result<String, AppError> {
    match blockchain {
        BlockchainNetwork::Ethereum | BlockchainNetwork::BNBChain => {
            // For EVM chains (Ethereum, BNB Chain), derive address from private key
            derive_evm_address(private_key)
        }
        BlockchainNetwork::Solana => {
            // For Solana, derive address from private key
            derive_solana_address(private_key)
        }
    }
}

/// Derive EVM address from private key (for Ethereum and BNB Chain)
fn derive_evm_address(private_key: &str) -> Result<String, AppError> {
    use secp256k1::{Secp256k1, SecretKey};
    use tiny_keccak::{Hasher, Keccak};

    // Remove '0x' prefix if present
    let key_str = private_key.strip_prefix("0x").unwrap_or(private_key);

    // Decode hex private key
    let key_bytes = hex::decode(key_str)
        .map_err(|_| AppError::BadRequest("Invalid private key format".to_string()))?;

    // Create secp256k1 context and secret key
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&key_bytes)
        .map_err(|_| AppError::BadRequest("Invalid private key".to_string()))?;

    // Get public key
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes = public_key.serialize_uncompressed();

    // Hash public key with Keccak256 (excluding the first byte which is 0x04)
    let mut hasher = Keccak::v256();
    hasher.update(&public_key_bytes[1..]);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    // Take last 20 bytes as address
    let address = format!("0x{}", hex::encode(&hash[12..]));
    Ok(address)
}

/// Derive Solana address from private key
fn derive_solana_address(private_key: &str) -> Result<String, AppError> {
    use ed25519_dalek::SigningKey;
    use solana_sdk::pubkey::Pubkey;

    // Decode base58 or hex private key
    let key_bytes = if private_key.len() == 64 {
        // Hex format
        hex::decode(private_key)
            .map_err(|_| AppError::BadRequest("Invalid private key format".to_string()))?
    } else {
        // Base58 format
        bs58::decode(private_key)
            .into_vec()
            .map_err(|_| AppError::BadRequest("Invalid private key format".to_string()))?
    };

    if key_bytes.len() != 32 {
        return Err(AppError::BadRequest("Invalid Solana private key length".to_string()));
    }

    // Create signing key
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    let signing_key = SigningKey::from_bytes(&key_array);

    // Get public key
    let verifying_key = signing_key.verifying_key();
    let pubkey = Pubkey::new_from_array(*verifying_key.as_bytes());

    Ok(pubkey.to_string())
}

/// Validate wallet private key format based on blockchain
fn validate_private_key(blockchain: &BlockchainNetwork, private_key: &str) -> Result<(), AppError> {
    match blockchain {
        BlockchainNetwork::Ethereum | BlockchainNetwork::BNBChain => {
            let key_str = private_key.strip_prefix("0x").unwrap_or(private_key);
            if key_str.len() != 64 {
                return Err(AppError::BadRequest("Invalid EVM private key length (expected 64 hex characters)".to_string()));
            }
            hex::decode(key_str)
                .map_err(|_| AppError::BadRequest("Invalid EVM private key format (must be hex)".to_string()))?;
            Ok(())
        }
        BlockchainNetwork::Solana => {
            // Solana private keys can be 32 bytes in hex (64 chars) or base58 encoded
            if private_key.len() == 64 {
                hex::decode(private_key)
                    .map_err(|_| AppError::BadRequest("Invalid Solana private key format".to_string()))?;
            } else {
                let decoded = bs58::decode(private_key)
                    .into_vec()
                    .map_err(|_| AppError::BadRequest("Invalid Solana private key format (must be base58 or hex)".to_string()))?;
                if decoded.len() != 32 && decoded.len() != 64 {
                    return Err(AppError::BadRequest("Invalid Solana private key length".to_string()));
                }
            }
            Ok(())
        }
    }
}

/// Create a new wallet connection
pub async fn create_wallet_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateWalletConnectionRequest>,
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

    // Validate supported blockchain network
    let blockchain = BlockchainNetwork::from_str(&body.blockchain_network)
        .ok_or_else(|| AppError::BadRequest("Unsupported blockchain network".to_string()))?;

    // Validate private key format
    validate_private_key(&blockchain, &body.private_key)?;

    // Derive wallet address from private key
    let wallet_address = derive_wallet_address(&blockchain, &body.private_key)?;

    tracing::info!("Derived wallet address: {} for network: {}", wallet_address, blockchain.as_str());

    // Check if user already has this wallet connected
    let existing_connection = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .filter(wallet_connection::Column::BlockchainNetwork.eq(blockchain.as_str()))
        .filter(wallet_connection::Column::WalletAddress.eq(&wallet_address))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_connection.is_some() {
        return Err(AppError::BadRequest("This wallet is already connected".to_string()));
    }

    // Encrypt the private key
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_private_key = encryption_service
        .encrypt_api_credentials(&body.private_key, &body.password, &user_id_str)?;

    let connection_id = Uuid::new_v4();
    let now = Utc::now();

    tracing::info!("Creating wallet connection: id={}, user_id={}, network={}, address={}",
                   connection_id, user_id, blockchain.as_str(), wallet_address);

    let connection_model = WalletConnectionActiveModel {
        id: Set(connection_id),
        user_id: Set(user_id),
        blockchain_network: Set(blockchain.as_str().to_string()),
        wallet_address: Set(wallet_address.clone()),
        display_name: Set(body.display_name.clone()),
        encrypted_private_key: Set(encrypted_private_key.ciphertext),
        private_key_nonce: Set(encrypted_private_key.nonce),
        private_key_salt: Set(encrypted_private_key.salt),
        is_active: Set(true),
        last_used: Set(None),
        connection_status: Set("connected".to_string()),
        last_error: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let connection = connection_model.insert(db.get_ref()).await
        .map_err(|e| {
            tracing::error!("Failed to insert wallet connection: {:?}", e);
            AppError::InternalServerError
        })?;

    Ok(HttpResponse::Created().json(WalletConnectionResponse::from(connection)))
}

/// Get all wallet connections for a user
pub async fn get_wallet_connections(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    let connections = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let responses: Vec<WalletConnectionResponse> = connections
        .into_iter()
        .map(WalletConnectionResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "wallets": responses
    })))
}

/// Get a specific wallet connection
pub async fn get_wallet_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    let connection = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::Id.eq(connection_id))
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Wallet connection not found".to_string()))?;

    Ok(HttpResponse::Ok().json(WalletConnectionResponse::from(connection)))
}

/// Update a wallet connection
pub async fn update_wallet_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateWalletConnectionRequest>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    body.validate().map_err(AppError::ValidationError)?;

    // Find the connection
    let connection = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::Id.eq(connection_id))
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Wallet connection not found".to_string()))?;

    let mut active_model: WalletConnectionActiveModel = connection.into();

    // Update display name if provided
    if let Some(display_name) = &body.display_name {
        active_model.display_name = Set(display_name.clone());
    }

    active_model.updated_at = Set(Utc::now());

    let updated_connection = active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(WalletConnectionResponse::from(updated_connection)))
}

/// Delete a wallet connection
pub async fn delete_wallet_connection(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let connection_id_str = path.into_inner();
    let connection_id = Uuid::parse_str(&connection_id_str)
        .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string()))?;

    let user_id = get_user_id_from_session(&req)?;

    // Verify the connection belongs to the user
    let connection = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::Id.eq(connection_id))
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Wallet connection not found".to_string()))?;

    // Delete the connection
    WalletConnectionEntity::delete_by_id(connection.id)
        .exec(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Wallet connection deleted successfully"
    })))
}

/// Get wallet balance for a specific connection
pub async fn get_wallet_balance(
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
    let connection = WalletConnectionEntity::find()
        .filter(wallet_connection::Column::Id.eq(connection_id))
        .filter(wallet_connection::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Wallet connection not found".to_string()))?;

    // Decrypt the private key
    let encryption_service = EncryptionService::new();
    let user_id_str = user_id.to_string();

    let encrypted_private_key = EncryptedData {
        ciphertext: connection.encrypted_private_key.clone(),
        nonce: connection.private_key_nonce.clone(),
        salt: connection.private_key_salt.clone(),
    };

    let _private_key = encryption_service
        .decrypt_api_credentials(&encrypted_private_key, password, &user_id_str)?;

    // TODO: Implement actual balance fetching from blockchain
    // For now, return a placeholder response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "wallet_address": connection.wallet_address,
        "blockchain_network": connection.blockchain_network,
        "balance": "0.0",
        "message": "Balance fetching will be implemented with DEX connectors"
    })))
}
