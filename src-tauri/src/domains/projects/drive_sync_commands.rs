//! Tauri adapter for external-drive sync. The domain is `drive_sync` in
//! portal-core; this resolves the drive and the project, then runs it off the
//! async runtime.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::disk::disk::{disk_usage, DiskUsage};
use crate::domains::projects::drive_sync::{
    self, DriveProjectStatus, DriveSyncDirection, DriveSyncResult, SyncDrive,
};
use crate::domains::projects::move_lock;
use crate::entities::project;

/// One apply at a time: they share each drive's manifest.
static APPLY: Mutex<()> = Mutex::new(());

/// Every volume but the system drive. USB disks often report as fixed, so
/// `removable` is information only.
fn volumes() -> Vec<DiskUsage> {
    let system = if cfg!(windows) {
        std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into())
    } else {
        "/".into()
    };
    let mut all = disk_usage();
    all.retain(|d| move_lock::norm(&d.mount_point) != move_lock::norm(&system));
    all
}

/// The drive comes from the UI; only a mounted volume is accepted.
fn volume(drive: &str) -> Result<DiskUsage, String> {
    volumes()
        .into_iter()
        .find(|d| d.mount_point == drive)
        .ok_or_else(|| "That drive is not available".to_string())
}

struct Subject {
    name: String,
    path: PathBuf,
    /// The other active projects, for "covered by a parent".
    others: Vec<(String, PathBuf)>,
}

impl Subject {
    async fn load(db: &DatabaseManager, id: i32) -> Result<Self, String> {
        let fail = |e: sea_orm::DbErr| format!("Failed to load projects: {e}");
        let conn = db.get_connection();
        let row = project::Entity::find_by_id(id)
            .one(conn)
            .await
            .map_err(fail)?
            .ok_or("Project not found")?;
        let others = project::Entity::find()
            .filter(project::Column::Archived.eq(false))
            .filter(project::Column::Id.ne(id))
            .all(conn)
            .await
            .map_err(fail)?
            .into_iter()
            .map(|p| (p.name, PathBuf::from(p.path)))
            .collect();
        Ok(Self {
            name: row.name,
            path: PathBuf::from(row.path),
            others,
        })
    }

    fn parent(&self) -> Option<String> {
        self.path
            .is_dir()
            .then(|| drive_sync::covered_by(&self.path, &self.others))
            .flatten()
    }
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

// All local_only: these read and write project folders, `.env` files included.

#[portal_rpc_macro::rpc(local_only)]
pub async fn drive_sync_drives() -> Result<Vec<SyncDrive>, String> {
    blocking(|| {
        Ok(volumes()
            .into_iter()
            .map(|d| SyncDrive {
                has_mirror: Path::new(&d.mount_point)
                    .join(".portal-sync/portal-sync.json")
                    .is_file(),
                mount_point: d.mount_point,
                label: d.name,
                fs_kind: d.fs_kind,
                total_bytes: d.total_bytes,
                available_bytes: d.available_bytes,
                removable: d.is_removable,
            })
            .collect())
    })
    .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn drive_sync_status(
    drive: String,
    project_id: i32,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<DriveProjectStatus, String> {
    let subject = Subject::load(db.inner(), project_id).await?;
    blocking(move || {
        let volume = volume(&drive)?;
        move_lock::ensure_unlocked(&subject.path)?;
        if let Some(parent) = subject.parent() {
            return Ok(DriveProjectStatus::covered(parent));
        }
        drive_sync::status(&subject.path, Path::new(&volume.mount_point))
    })
    .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn drive_sync_apply(
    drive: String,
    project_id: i32,
    direction: DriveSyncDirection,
    force: bool,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<DriveSyncResult, String> {
    let subject = Subject::load(db.inner(), project_id).await?;
    blocking(move || {
        let _one = APPLY.lock().unwrap_or_else(|e| e.into_inner());
        // Looked up under the lock so free space reflects earlier copies.
        let volume = volume(&drive)?;
        move_lock::ensure_unlocked(&subject.path)?;
        if let Some(parent) = subject.parent() {
            return Err(format!("Covered by {parent}; sync that project instead"));
        }
        let mount = Path::new(&volume.mount_point);
        match direction {
            DriveSyncDirection::ToDrive => drive_sync::to_drive(
                &subject.path,
                mount,
                &subject.name,
                force,
                volume.available_bytes,
            ),
            DriveSyncDirection::ToLocal => drive_sync::to_local(&subject.path, mount),
        }
    })
    .await
}
