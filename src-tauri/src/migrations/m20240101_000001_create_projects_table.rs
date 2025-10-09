use sea_orm_migration::prelude::*;

/// Migration: Create projects table
///
/// This migration creates the projects table with the following structure:
/// - id: Primary key (auto-increment)
/// - name: Project display name
/// - description: Project description (optional)
/// - path: File system path to the project
/// - status: Project status (active, inactive, etc.)
/// - framework: Detected or manually set framework (optional)
/// - package_manager: Package manager used (npm, yarn, pnpm, etc.)
/// - build_command: Build command for the project
/// - start_command: Start command for the project
/// - test_command: Test command for the project
/// - output_directory: Output directory for builds
/// - dev_port: Development server port
/// - prod_port: Production server port
/// - starred: Boolean flag for starred/favorite projects
/// - open_count: Number of times the project has been opened
/// - last_opened: Timestamp when project was last opened
/// - size: Project size in bytes
/// - file_count: Number of files in the project
/// - git_repository: Git repository URL
/// - git_branch: Current git branch
/// - git_commit: Last git commit hash
/// - has_uncommitted_changes: Whether there are uncommitted changes
/// - last_commit: Timestamp of last commit
/// - created_at: Timestamp when record was created
/// - updated_at: Timestamp when record was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).string().not_null())
                    .col(ColumnDef::new(Projects::Description).string().null())
                    .col(ColumnDef::new(Projects::Path).string().not_null())
                    .col(ColumnDef::new(Projects::Status).string().not_null().default("active"))
                    .col(ColumnDef::new(Projects::Framework).string().null())
                    .col(ColumnDef::new(Projects::PackageManager).string().null())
                    .col(ColumnDef::new(Projects::BuildCommand).string().null())
                    .col(ColumnDef::new(Projects::StartCommand).string().null())
                    .col(ColumnDef::new(Projects::TestCommand).string().null())
                    .col(ColumnDef::new(Projects::OutputDirectory).string().null())
                    .col(ColumnDef::new(Projects::DevPort).integer().null())
                    .col(ColumnDef::new(Projects::ProdPort).integer().null())
                    .col(ColumnDef::new(Projects::Starred).boolean().not_null().default(false))
                    .col(ColumnDef::new(Projects::OpenCount).integer().not_null().default(0))
                    .col(ColumnDef::new(Projects::LastOpened).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Projects::Size).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Projects::FileCount).integer().not_null().default(0))
                    .col(ColumnDef::new(Projects::GitRepository).string().null())
                    .col(ColumnDef::new(Projects::GitBranch).string().null())
                    .col(ColumnDef::new(Projects::GitCommit).string().null())
                    .col(ColumnDef::new(Projects::HasUncommittedChanges).boolean().not_null().default(false))
                    .col(ColumnDef::new(Projects::LastCommit).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Projects::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Projects::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    Name,
    Description,
    Path,
    Status,
    Framework,
    PackageManager,
    BuildCommand,
    StartCommand,
    TestCommand,
    OutputDirectory,
    DevPort,
    ProdPort,
    Starred,
    OpenCount,
    LastOpened,
    Size,
    FileCount,
    GitRepository,
    GitBranch,
    GitCommit,
    HasUncommittedChanges,
    LastCommit,
    CreatedAt,
    UpdatedAt,
}
