use super::*;
use bridge::GameLogLevel::*;

fn p(line: &str) -> Parsed<'_> {
    parse(line)
}

#[test]
fn forge_1_7_10() {
    let parsed = p("[06:18:16] [Client thread/INFO] [FML]: Found 448 ObjectHolder annotations");
    assert_eq!(parsed.thread, Some("Client thread"));
    assert_eq!(parsed.level, Some(Info));
    assert_eq!(parsed.logger, Some("FML"));
    assert_eq!(parsed.body, "Found 448 ObjectHolder annotations");
}

#[test]
fn a_bracket_in_the_message_stays_in_the_message() {
    let parsed = p("[06:17:57] [main/INFO] [FML]: [AppEng] Core Init");
    assert_eq!(parsed.logger, Some("FML"));
    assert_eq!(parsed.body, "[AppEng] Core Init");
}

#[test]
fn vanilla_has_no_logger() {
    let parsed = p("[06:18:02] [main/INFO]: Setting user: Dalynkaa");
    assert_eq!(parsed.level, Some(Info));
    assert_eq!(parsed.logger, None);
    assert_eq!(parsed.body, "Setting user: Dalynkaa");
}

#[test]
fn neoforge_dates_its_lines() {
    let parsed = p(
        "[03Oct2026 06:18:16.123] [main/WARN] [net.neoforged.fml.loading.moddiscovery/]: Odd jar",
    );
    assert_eq!(parsed.level, Some(Warn));
    assert_eq!(
        parsed.logger,
        Some("net.neoforged.fml.loading.moddiscovery")
    );
    assert_eq!(parsed.body, "Odd jar");
}

#[test]
fn fabric_puts_the_logger_in_parentheses() {
    let parsed = p("[06:18:16] [main/INFO] (FabricLoader) Loading 123 mods");
    assert_eq!(parsed.logger, Some("FabricLoader"));
    assert_eq!(parsed.body, "Loading 123 mods");
}

#[test]
fn authlib_injector() {
    let parsed = p("[authlib-injector] [INFO] Httpd is running on port 65172");
    assert_eq!(parsed.logger, Some("authlib-injector"));
    assert_eq!(parsed.level, Some(Info));
    assert_eq!(parsed.body, "Httpd is running on port 65172");
}

#[test]
fn a_redirected_print_names_its_class() {
    let parsed = p("[06:17:59] [main/INFO] [STDOUT]: [pcl.opensecurity.util.SoundUnpack:load:32]: Extracting file: klaxon2.ogg");
    assert_eq!(parsed.logger, Some("SoundUnpack"));
    assert_eq!(parsed.body, "Extracting file: klaxon2.ogg");
    assert!(!parsed.thrown);
}

#[test]
fn a_printed_exception_is_an_error_whatever_log4j_says() {
    let line = "[05:34:43] [main/INFO]: [java.lang.Throwable$WrappedPrintStream:println:763]: java.lang.NoClassDefFoundError: java/util/jar/Pack200";
    let parsed = p(line);
    assert!(parsed.thrown);
    assert_eq!(
        parsed.body,
        "java.lang.NoClassDefFoundError: java/util/jar/Pack200"
    );

    let frame = "[05:34:43] [main/INFO]: [java.lang.Throwable$WrappedPrintStream:println:763]:     at cpw.mods.fml.common.patcher.ClassPatchManager.setup(ClassPatchManager.java:161)";
    let parsed = p(frame);
    assert!(parsed.thrown);
    assert!(continues(frame, parsed.body));
}

#[test]
fn a_plain_line_is_all_message() {
    let parsed = p("WARNING: A terminally deprecated method in java.lang.System has been called");
    assert_eq!(parsed.level, None);
    assert_eq!(parsed.logger, None);
    assert_eq!(
        parsed.body,
        "WARNING: A terminally deprecated method in java.lang.System has been called"
    );
}

#[test]
fn stack_trace_tails_continue_the_line_above() {
    assert!(continues(
        "\tat net.minecraft.client.Minecraft.run(Minecraft.java:961)",
        "at net.minecraft.client.Minecraft.run(Minecraft.java:961)"
    ));
    assert!(continues(
        "Caused by: java.lang.ClassNotFoundException: x",
        "Caused by: java.lang.ClassNotFoundException: x"
    ));
    assert!(continues("\t... 10 more", "... 10 more"));
    assert!(!continues("at the end of the day", "at the end of the day"));
}
