use chrono::Utc;
use sea_orm::*;

use crate::db::entities::device;
use crate::services::relying_party::ServiceError;

pub struct DeviceService;

impl DeviceService {
    /// Register a new device or update an existing one by FCM token.
    pub async fn register(
        db: &DatabaseConnection,
        account_id: &str,
        fcm_token: &str,
        device_name: Option<&str>,
        platform: &str,
    ) -> Result<device::Model, ServiceError> {
        if let Some(existing) = device::Entity::find()
            .filter(device::Column::FcmToken.eq(fcm_token))
            .one(db)
            .await
            .map_err(ServiceError::Db)?
        {
            let update = device::ActiveModel {
                id: Set(existing.id.clone()),
                account_id: Set(account_id.to_string()),
                device_name: Set(device_name.map(|s| s.to_string())),
                is_active: Set(true),
                updated_at: Set(Utc::now().into()),
                ..Default::default()
            };
            device::Entity::update(update)
                .exec(db)
                .await
                .map_err(ServiceError::Db)?;
            return device::Entity::find_by_id(&existing.id)
                .one(db)
                .await
                .map_err(ServiceError::Db)?
                .ok_or(ServiceError::NotFound("device".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let model = device::ActiveModel {
            id: Set(id.clone()),
            account_id: Set(account_id.to_string()),
            fcm_token: Set(fcm_token.to_string()),
            device_name: Set(device_name.map(|s| s.to_string())),
            platform: Set(platform.to_string()),
            is_active: Set(true),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        device::Entity::insert(model)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        device::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("device".to_string()))
    }

    /// Update a device's FCM token and/or name.
    pub async fn update(
        db: &DatabaseConnection,
        device_id: &str,
        fcm_token: Option<&str>,
        device_name: Option<&str>,
    ) -> Result<device::Model, ServiceError> {
        let _ = device::Entity::find_by_id(device_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "device '{device_id}' not found"
            )))?;

        let mut update = device::ActiveModel {
            id: Set(device_id.to_string()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };

        if let Some(token) = fcm_token {
            update.fcm_token = Set(token.to_string());
        }
        if let Some(name) = device_name {
            update.device_name = Set(Some(name.to_string()));
        }

        device::Entity::update(update)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        device::Entity::find_by_id(device_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("device".to_string()))
    }

    /// Soft-delete a device.
    pub async fn deactivate(db: &DatabaseConnection, device_id: &str) -> Result<(), ServiceError> {
        let _ = device::Entity::find_by_id(device_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "device '{device_id}' not found"
            )))?;

        let update = device::ActiveModel {
            id: Set(device_id.to_string()),
            is_active: Set(false),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };
        device::Entity::update(update)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;
        Ok(())
    }

    /// Find all active devices for an account.
    pub async fn find_active_by_account(
        db: &DatabaseConnection,
        account_id: &str,
    ) -> Result<Vec<device::Model>, ServiceError> {
        device::Entity::find()
            .filter(device::Column::AccountId.eq(account_id))
            .filter(device::Column::IsActive.eq(true))
            .all(db)
            .await
            .map_err(ServiceError::Db)
    }
}
