//! noro-admin — CLI управления мастер-сервером.

mod client;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::{print_json, Client};
use serde_json::json;

#[derive(Parser)]
#[command(name = "noro-admin", about = "Управление noro мастер-сервером")]
struct Cli {
    /// URL мастера.
    #[arg(
        long,
        env = "NORO_MASTER_URL",
        default_value = "http://localhost:8080",
        global = true
    )]
    server: String,
    /// Admin-токен (или пользовательский Bearer с правами).
    #[arg(long, env = "NORO_ADMIN_TOKEN", default_value = "", global = true)]
    token: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Серверы.
    Server {
        #[command(subcommand)]
        cmd: ServerCmd,
    },
    /// Сборки.
    Build {
        #[command(subcommand)]
        cmd: BuildCmd,
    },
    /// Моды.
    Mod {
        #[command(subcommand)]
        cmd: ModCmd,
    },
    /// Роли.
    Role {
        #[command(subcommand)]
        cmd: RoleCmd,
    },
    /// Пользователи.
    User {
        #[command(subcommand)]
        cmd: UserCmd,
    },
    /// Новости.
    News {
        #[command(subcommand)]
        cmd: NewsCmd,
    },
    /// Серверные ядра.
    Core {
        #[command(subcommand)]
        cmd: CoreCmd,
    },
    /// Admin-токены.
    Token {
        #[command(subcommand)]
        cmd: TokenCmd,
    },
    /// Версии лаунчера.
    Launcher {
        #[command(subcommand)]
        cmd: LauncherCmd,
    },
    /// Статистика.
    Stats,
}

#[derive(Subcommand)]
enum ServerCmd {
    List,
    Create {
        name: String,
        modloader: String,
        mc_version: String,
    },
    Delete {
        id: String,
    },
    Edit {
        id: String,
        name: String,
        description: String,
        modloader: String,
        mc_version: String,
        #[arg(long, default_value_t = true)]
        active: bool,
        #[arg(long, default_value_t = false)]
        limited: bool,
        #[arg(long, default_value_t = 0)]
        sort_order: i32,
    },
}

#[derive(Subcommand)]
enum BuildCmd {
    List {
        server_id: String,
    },
    Create {
        server_id: String,
        version: String,
        modloader: String,
        mc_version: String,
        #[arg(long)]
        modloader_version: Option<String>,
    },
    Publish {
        id: String,
    },
    Unpublish {
        id: String,
    },
    Files {
        id: String,
    },
    SetPaths {
        id: String,
        #[arg(long = "unmanaged")]
        unmanaged: Vec<String>,
        #[arg(long = "user-managed")]
        user_managed: Vec<String>,
    },
    ImportMrpack {
        id: String,
        file: String,
    },
    ImportCf {
        id: String,
        file: String,
    },
    AddFile {
        id: String,
        file: String,
        #[arg(long)]
        path: Option<String>,
    },
    Delete {
        id: String,
    },
}

#[derive(Subcommand)]
enum ModCmd {
    Search {
        build_id: String,
        query: String,
        #[arg(long)]
        mc: Option<String>,
        #[arg(long)]
        loader: Option<String>,
    },
    AddModrinth {
        build_id: String,
        version_id: String,
    },
    AddCurseForge {
        build_id: String,
        project_id: u64,
        file_id: u64,
    },
    AddUrl {
        build_id: String,
        url: String,
    },
}

#[derive(Subcommand)]
enum RoleCmd {
    List,
    Create {
        name: String,
        display_name: String,
        #[arg(long)]
        color: Option<String>,
    },
    Delete {
        id: String,
    },
    Edit {
        id: String,
        display_name: String,
        #[arg(long)]
        color: Option<String>,
        #[arg(long, default_value_t = false)]
        is_default: bool,
        #[arg(long, default_value_t = 0)]
        sort_order: i32,
    },
    GrantPerm {
        id: String,
        permission: String,
    },
    RevokePerm {
        id: String,
        permission: String,
    },
}

#[derive(Subcommand)]
enum UserCmd {
    List,
    Ban {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
    Unban {
        id: String,
    },
    AddRole {
        id: String,
        role_id: String,
    },
    RemoveRole {
        id: String,
        role_id: String,
    },
    AddPerm {
        id: String,
        permission: String,
    },
    RemovePerm {
        id: String,
        permission: String,
    },
}

#[derive(Subcommand)]
enum NewsCmd {
    Create {
        title: String,
        body: String,
        #[arg(long)]
        pinned: bool,
    },
    Edit {
        id: String,
        title: String,
        body: String,
        #[arg(long)]
        preview: Option<String>,
        #[arg(long)]
        pinned: bool,
    },
    Delete {
        id: String,
    },
}

#[derive(Subcommand)]
enum CoreCmd {
    List {
        #[arg(long)]
        server_id: Option<String>,
    },
    Upload {
        server_id: String,
        file: String,
        #[arg(long)]
        version: Option<String>,
    },
    Activate {
        id: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Subcommand)]
enum TokenCmd {
    List,
    Create {
        name: String,
        #[arg(long)]
        perm: Vec<String>,
    },
    Revoke {
        id: String,
    },
}

#[derive(Subcommand)]
enum LauncherCmd {
    Versions,
    Github,
    Build { tag: String },
    Log { job_id: String },
    Deploy { version_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let c = Client::new(cli.server.clone(), cli.token.clone());

    match cli.command {
        Command::Server { cmd } => server(&c, cmd).await?,
        Command::Build { cmd } => build(&c, cmd).await?,
        Command::Mod { cmd } => mods(&c, cmd).await?,
        Command::Role { cmd } => role(&c, cmd).await?,
        Command::User { cmd } => user(&c, cmd).await?,
        Command::News { cmd } => news(&c, cmd).await?,
        Command::Core { cmd } => core(&c, cmd).await?,
        Command::Token { cmd } => token(&c, cmd).await?,
        Command::Launcher { cmd } => launcher(&c, cmd).await?,
        Command::Stats => print_json(&c.get("/api/admin/stats").await?),
    }
    Ok(())
}

async fn server(c: &Client, cmd: ServerCmd) -> Result<()> {
    let v = match cmd {
        ServerCmd::List => c.get("/api/admin/servers").await?,
        ServerCmd::Create {
            name,
            modloader,
            mc_version,
        } => {
            c.post(
                "/api/admin/servers",
                json!({ "name": name, "modloader": modloader, "mc_version": mc_version }),
            )
            .await?
        }
        ServerCmd::Delete { id } => c.delete(&format!("/api/admin/servers/{id}")).await?,
        ServerCmd::Edit {
            id,
            name,
            description,
            modloader,
            mc_version,
            active,
            limited,
            sort_order,
        } => {
            c.put(
                &format!("/api/admin/servers/{id}"),
                json!({
                    "name": name,
                    "description": description,
                    "modloader": modloader,
                    "mc_version": mc_version,
                    "active": active,
                    "limited": limited,
                    "sort_order": sort_order
                }),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}

async fn build(c: &Client, cmd: BuildCmd) -> Result<()> {
    let v = match cmd {
        BuildCmd::List { server_id } => c.get(&format!("/api/admin/builds?server_id={server_id}")).await?,
        BuildCmd::Create { server_id, version, modloader, mc_version, modloader_version } => {
            c.post(
                "/api/admin/builds",
                json!({ "server_id": server_id, "version": version, "modloader": modloader, "mc_version": mc_version, "modloader_version": modloader_version }),
            )
            .await?
        }
        BuildCmd::Publish { id } => {
            eprintln!("публикация (bootstrap может занять время)...");
            c.post(&format!("/api/admin/builds/{id}/publish"), json!({})).await?
        }
        BuildCmd::Unpublish { id } => c.post(&format!("/api/admin/builds/{id}/unpublish"), json!({})).await?,
        BuildCmd::Files { id } => c.get(&format!("/api/admin/builds/{id}/files")).await?,
        BuildCmd::SetPaths { id, unmanaged, user_managed } => {
            c.put(
                &format!("/api/admin/builds/{id}/paths"),
                json!({ "unmanaged_paths": unmanaged, "user_managed_paths": user_managed }),
            )
            .await?
        }
        BuildCmd::ImportMrpack { id, file } => {
            c.upload(&format!("/api/admin/builds/{id}/import/mrpack"), &file, None).await?
        }
        BuildCmd::ImportCf { id, file } => {
            c.upload(&format!("/api/admin/builds/{id}/import/curseforge"), &file, None).await?
        }
        BuildCmd::AddFile { id, file, path } => {
            c.upload(&format!("/api/admin/builds/{id}/files"), &file, path.as_deref()).await?
        }
        BuildCmd::Delete { id } => c.delete(&format!("/api/admin/builds/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}

async fn mods(c: &Client, cmd: ModCmd) -> Result<()> {
    let v = match cmd {
        ModCmd::Search {
            build_id,
            query,
            mc,
            loader,
        } => {
            let mut path = format!(
                "/api/admin/builds/{build_id}/mods/search?q={}",
                urlencode(&query)
            );
            if let Some(mc) = mc {
                path.push_str(&format!("&mc={mc}"));
            }
            if let Some(l) = loader {
                path.push_str(&format!("&loader={l}"));
            }
            c.get(&path).await?
        }
        ModCmd::AddModrinth {
            build_id,
            version_id,
        } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-modrinth"),
                json!({ "version_id": version_id }),
            )
            .await?
        }
        ModCmd::AddCurseForge {
            build_id,
            project_id,
            file_id,
        } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-curseforge"),
                json!({ "project_id": project_id, "file_id": file_id }),
            )
            .await?
        }
        ModCmd::AddUrl { build_id, url } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-url"),
                json!({ "url": url }),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}

async fn role(c: &Client, cmd: RoleCmd) -> Result<()> {
    let v = match cmd {
        RoleCmd::List => c.get("/api/admin/roles").await?,
        RoleCmd::Create {
            name,
            display_name,
            color,
        } => {
            c.post(
                "/api/admin/roles",
                json!({ "name": name, "display_name": display_name, "color": color }),
            )
            .await?
        }
        RoleCmd::Delete { id } => c.delete(&format!("/api/admin/roles/{id}")).await?,
        RoleCmd::Edit {
            id,
            display_name,
            color,
            is_default,
            sort_order,
        } => {
            c.put(
                &format!("/api/admin/roles/{id}"),
                json!({
                    "display_name": display_name,
                    "color": color,
                    "is_default": is_default,
                    "sort_order": sort_order
                }),
            )
            .await?
        }
        RoleCmd::GrantPerm { id, permission } => {
            c.post(
                &format!("/api/admin/roles/{id}/permissions"),
                json!({ "permission": permission }),
            )
            .await?
        }
        RoleCmd::RevokePerm { id, permission } => {
            c.delete(&format!(
                "/api/admin/roles/{id}/permissions/{}",
                urlencode(&permission)
            ))
            .await?
        }
    };
    print_json(&v);
    Ok(())
}

async fn user(c: &Client, cmd: UserCmd) -> Result<()> {
    let v = match cmd {
        UserCmd::List => c.get("/api/admin/users").await?,
        UserCmd::Ban { id, reason } => {
            c.put(
                &format!("/api/admin/users/{id}/ban"),
                json!({ "banned": true, "reason": reason }),
            )
            .await?
        }
        UserCmd::Unban { id } => {
            c.put(
                &format!("/api/admin/users/{id}/ban"),
                json!({ "banned": false }),
            )
            .await?
        }
        UserCmd::AddRole { id, role_id } => {
            c.post(&format!("/api/admin/users/{id}/roles/{role_id}"), json!({}))
                .await?
        }
        UserCmd::RemoveRole { id, role_id } => {
            c.delete(&format!("/api/admin/users/{id}/roles/{role_id}"))
                .await?
        }
        UserCmd::AddPerm { id, permission } => {
            c.post(
                &format!("/api/admin/users/{id}/permissions"),
                json!({ "permission": permission }),
            )
            .await?
        }
        UserCmd::RemovePerm { id, permission } => {
            c.delete(&format!(
                "/api/admin/users/{id}/permissions/{}",
                urlencode(&permission)
            ))
            .await?
        }
    };
    print_json(&v);
    Ok(())
}

async fn news(c: &Client, cmd: NewsCmd) -> Result<()> {
    let v = match cmd {
        NewsCmd::Create { title, body, pinned } => {
            c.post("/api/admin/news", json!({ "title": title, "body": body, "pinned": pinned })).await?
        }
        NewsCmd::Edit { id, title, body, preview, pinned } => {
            c.put(
                &format!("/api/admin/news/{id}"),
                json!({ "title": title, "body": body, "preview_img_url": preview, "pinned": pinned }),
            )
            .await?
        }
        NewsCmd::Delete { id } => c.delete(&format!("/api/admin/news/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}

async fn core(c: &Client, cmd: CoreCmd) -> Result<()> {
    let v = match cmd {
        CoreCmd::List { server_id } => {
            let path = match server_id {
                Some(id) => format!("/api/admin/cores?server_id={id}"),
                None => "/api/admin/cores".to_string(),
            };
            c.get(&path).await?
        }
        CoreCmd::Upload {
            server_id,
            file,
            version,
        } => {
            let mut fields = vec![("server_id".to_string(), server_id)];
            if let Some(version) = version {
                fields.push(("version".to_string(), version));
            }
            c.upload_fields("/api/admin/cores", &file, fields).await?
        }
        CoreCmd::Activate { id } => {
            c.post(&format!("/api/admin/cores/{id}/activate"), json!({}))
                .await?
        }
        CoreCmd::Delete { id } => c.delete(&format!("/api/admin/cores/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}

async fn token(c: &Client, cmd: TokenCmd) -> Result<()> {
    let v = match cmd {
        TokenCmd::List => c.get("/api/admin/tokens").await?,
        TokenCmd::Create { name, perm } => {
            c.post(
                "/api/admin/tokens",
                json!({ "name": name, "permissions": perm }),
            )
            .await?
        }
        TokenCmd::Revoke { id } => c.delete(&format!("/api/admin/tokens/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}

async fn launcher(c: &Client, cmd: LauncherCmd) -> Result<()> {
    let v = match cmd {
        LauncherCmd::Versions => c.get("/api/admin/launcher/versions").await?,
        LauncherCmd::Github => c.get("/api/admin/launcher/github").await?,
        LauncherCmd::Build { tag } => {
            c.post("/api/admin/launcher/build", json!({ "tag": tag }))
                .await?
        }
        LauncherCmd::Log { job_id } => {
            c.get(&format!("/api/admin/launcher/build/{job_id}/log"))
                .await?
        }
        LauncherCmd::Deploy { version_id } => {
            c.post(
                &format!("/api/admin/launcher/deploy/{version_id}"),
                json!({}),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}

/// Минимальный URL-энкодер для path/query сегментов.
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
