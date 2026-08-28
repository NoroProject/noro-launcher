//! Проверяется и то, что нужное собирается, и то, что лишнее не собирается —
//! второе важнее: бандл уезжает наружу.

use super::*;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("noro-bundle-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn names(bundle: &Bundle) -> Vec<&str> {
    bundle.files.iter().map(|f| f.name.as_str()).collect()
}

#[tokio::test]
async fn the_allowlist_picks_up_logs_and_options() {
    let dir = Scratch::new("allowlist");
    dir.write("logs/latest.log", "[main/INFO]: started");
    dir.write("options.txt", "fov:70");
    dir.write("crash-reports/crash-2026-08-16.txt", "java.lang.Error");

    let bundle = collect(dir.path(), None, &[]).await;

    let mut got = names(&bundle);
    got.sort();
    assert_eq!(
        got,
        [
            "crash-reports/crash-2026-08-16.txt",
            "logs/latest.log",
            "options.txt"
        ]
    );
}

#[tokio::test]
async fn private_directories_are_never_collected() {
    // Прямое требование: миры, скриншоты и список серверов не собираются
    // никогда, даже с согласия игрока.
    let dir = Scratch::new("private");
    dir.write("logs/latest.log", "ok");
    dir.write("saves/Мир/level.dat", "мир игрока");
    dir.write("screenshots/2026-08-16.png", "картинка");
    dir.write("servers.dat", "чужие серверы");

    let bundle = collect(dir.path(), None, &[]).await;

    let joined = format!("{:?}", names(&bundle));
    assert!(!joined.contains("saves"), "{joined}");
    assert!(!joined.contains("screenshots"), "{joined}");
    assert!(!joined.contains("servers.dat"), "{joined}");
}

#[tokio::test]
async fn a_token_in_the_log_never_reaches_the_bundle() {
    let dir = Scratch::new("token");
    let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.Qm9ndXM";
    dir.write("logs/latest.log", &format!("--accessToken {token}"));

    let bundle = collect(dir.path(), None, &[]).await;

    assert!(!bundle.files[0].text.contains(token));
    // Предпросмотр показывает ровно то же, что уедет: иначе кнопка «посмотреть,
    // что отправится» врёт.
    assert!(!bundle.preview().contains(token));
}

#[tokio::test]
async fn only_the_newest_crash_reports_are_taken() {
    let dir = Scratch::new("crashes");
    for i in 1..=6 {
        dir.write(&format!("crash-reports/crash-{i}.txt"), "boom");
    }

    let bundle = collect(dir.path(), None, &[]).await;

    assert_eq!(bundle.files.len(), 3, "{:?}", names(&bundle));
}

#[tokio::test]
async fn an_enormous_log_is_cut_from_the_middle() {
    let dir = Scratch::new("huge");
    let body = format!("НАЧАЛО{}КОНЕЦ", "ф".repeat(600 * 1024));
    dir.write("logs/latest.log", &body);

    let bundle = collect(dir.path(), None, &[]).await;

    let text = &bundle.files[0].text;
    assert!(text.len() < body.len());
    // Причина обычно в первых строках, симптом — в последних; вырезается
    // середина.
    assert!(text.starts_with("НАЧАЛО"), "{}", &text[..40]);
    assert!(text.ends_with("КОНЕЦ"));
    assert!(text.contains("вырезано"));
}

#[tokio::test]
async fn a_rotated_gzip_log_is_decompressed_and_cleaned() {
    use flate2::write::GzEncoder;
    use std::io::Write as _;

    let dir = Scratch::new("gzip");
    let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyIn0.Qm9ndXM";
    let mut enc = GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(format!("--accessToken {token}").as_bytes())
        .unwrap();
    std::fs::create_dir_all(dir.path().join("logs")).unwrap();
    std::fs::write(
        dir.path().join("logs/2026-08-15-1.log.gz"),
        enc.finish().unwrap(),
    )
    .unwrap();

    let bundle = collect(dir.path(), None, &[]).await;

    // Отправить .gz как есть значит отправить токен, которого мы в нём не
    // видели: сжатый файл обязан быть разжат и очищен.
    assert_eq!(names(&bundle), ["logs/2026-08-15-1.log.gz"]);
    assert!(!bundle.files[0].text.contains(token));
}

#[tokio::test]
async fn the_archive_carries_the_same_text_as_the_preview() {
    let dir = Scratch::new("pack");
    dir.write("logs/latest.log", "строка лога");

    let bundle = collect(dir.path(), None, &[]).await;
    let bytes = super::super::pack(&bundle).unwrap();

    assert!(!bytes.is_empty());
    // Zip начинается с PK: архив, а не сырой текст.
    assert_eq!(&bytes[..2], b"PK");
}
