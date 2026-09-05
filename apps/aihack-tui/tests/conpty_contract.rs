#![cfg(windows)]

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

fn wait_for(rx: &Receiver<Vec<u8>>, transcript: &mut Vec<u8>, needle: &[u8]) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !transcript
        .windows(needle.len())
        .any(|window| window == needle)
    {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "ConPTY output timeout: {:?}", needle);
        let chunk = rx.recv_timeout(remaining).unwrap_or_else(|error| {
            panic!(
                "ConPTY reader stopped for {:?}: {error:?}; transcript={}",
                needle,
                String::from_utf8_lossy(transcript)
            )
        });
        transcript.extend_from_slice(&chunk);
    }
}

#[test]
fn windows_conpty_save_load_across_two_production_processes_preserves_rng() {
    assert_process_save_resume(false);
}

#[test]
fn windows_conpty_campaign_v2_save_load_across_two_processes_preserves_rng() {
    assert_process_save_resume(true);
}

fn assert_process_save_resume(campaign: bool) {
    use aihack_ai_contract::CommandIntent;
    use aihack_runtime::{GameClient, GameSession};
    use aihack_tui::tui::{TuiApp, UiRuntimeConfig};
    let directory = tempfile::tempdir().unwrap();
    for restoring in [false, true] {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_aihack"));
        command.args(["--seed", "42", "--save-dir"]);
        command.arg(directory.path());
        command.cwd(env!("CARGO_MANIFEST_DIR"));
        command.env("TERM", "xterm-256color");
        let mut child = pair.slave.spawn_command(command).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let mut writer = pair.master.take_writer().unwrap();
        let (tx, rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(count) = reader.read(&mut buffer) {
                if count == 0 || tx.send(buffer[..count].to_vec()).is_err() {
                    break;
                }
            }
        });
        let mut transcript = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        if transcript.windows(4).any(|window| window == b"\x1b[6n") {
            writer.write_all(b"\x1b[1;1R").unwrap();
            writer.flush().unwrap();
        }
        wait_for(&rx, &mut transcript, b"Press Enter to Start");
        if restoring {
            writer.write_all(b"L").unwrap();
            writer.flush().unwrap();
        } else {
            writer.write_all(b"\r").unwrap();
            writer.flush().unwrap();
            wait_for(&rx, &mut transcript, b"Character Creation");
            thread::sleep(Duration::from_millis(650));
            writer
                .write_all(if campaign { b"1" } else { b"\r" })
                .unwrap();
            writer.flush().unwrap();
        }
        wait_for(&rx, &mut transcript, b"COMMANDS");
        for key in [b'.', b'S', b'Q', b'y'] {
            thread::sleep(Duration::from_millis(650));
            writer.write_all(&[key]).unwrap();
            writer.flush().unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                panic!("save/load ConPTY child did not exit");
            }
            thread::sleep(Duration::from_millis(10));
        }
        drop(writer);
        drop(pair.master);
        drop(rx);
        reader_thread.join().unwrap();
        assert!(directory.path().join("quick-save.json").is_file());
    }
    let mut actual = TuiApp::new(GameSession::try_new(7).unwrap(), UiRuntimeConfig::default())
        .with_save_directory(directory.path())
        .unwrap();
    actual.quick_load().unwrap();
    let mut expected = GameSession::try_new(42).unwrap();
    // 실제 CLI의 Title → Creation → Playing 전이를 포함한다.
    assert!(expected.submit(CommandIntent::Wait).accepted);
    assert!(
        expected
            .submit(if campaign {
                CommandIntent::StartCampaign {
                    role: aihack_ai_contract::Role::Knight,
                }
            } else {
                CommandIntent::Wait
            })
            .accepted
    );
    for _ in 0..2 {
        assert!(expected.submit(CommandIntent::Wait).accepted);
    }
    assert_eq!(actual.revision(), expected.revision());
}

#[test]
fn windows_conpty_runs_one_event_per_state_accepts_mouse_and_restores_ansi_terminal_state() {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_aihack"));
    command.args(["--seed", "42"]);
    command.cwd(env!("CARGO_MANIFEST_DIR"));
    command.env("TERM", "xterm-256color");
    let mut child = pair.slave.spawn_command(command).unwrap();
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let mut writer = pair.master.take_writer().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader_thread = thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if tx.send(buffer[..count].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut transcript = Vec::new();
    let cursor_query = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    transcript.extend_from_slice(&cursor_query);
    if transcript
        .windows(b"\x1b[6n".len())
        .any(|window| window == b"\x1b[6n")
    {
        writer.write_all(b"\x1b[1;1R").unwrap();
        writer.flush().unwrap();
    }
    wait_for(&rx, &mut transcript, b"Press Enter to Start");

    writer.write_all(b"\r").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"Character Creation");

    thread::sleep(Duration::from_millis(600));
    writer.write_all(b"\r").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"COMMANDS");

    thread::sleep(Duration::from_millis(600));
    writer.write_all(b"\x1b[<0;28;11M\x1b[<0;28;11m").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"H1");

    writer.write_all(b"i").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"INVENTORY");

    thread::sleep(Duration::from_millis(600));
    writer.write_all(b"\x1b").unwrap();
    writer.flush().unwrap();
    transcript.extend_from_slice(&rx.recv_timeout(Duration::from_secs(2)).unwrap());
    writer.write_all(b".").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"H2");

    writer.write_all(b"Q").unwrap();
    writer.flush().unwrap();
    thread::sleep(Duration::from_millis(650));
    writer.write_all(b"y").unwrap();
    writer.flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "ConPTY child did not exit");
        thread::yield_now();
    };
    drop(writer);
    drop(pair.master);
    while let Ok(chunk) = rx.recv_timeout(Duration::from_millis(50)) {
        transcript.extend_from_slice(&chunk);
    }
    reader_thread.join().unwrap();

    assert!(status.success(), "ConPTY child exit={}", status.exit_code());
    for sequence in [
        b"?1049h".as_slice(),
        b"?1049l".as_slice(),
        b"?25l".as_slice(),
        b"?25h".as_slice(),
    ] {
        assert!(
            transcript
                .windows(sequence.len())
                .any(|window| window == sequence),
            "missing terminal lifecycle sequence: {:?}",
            sequence
        );
    }
    // Windows crossterm은 mouse/raw mode를 ANSI가 아니라 Console API로 전환한다.
    // 위 실제 SGR mouse click 성공과 package-local lifecycle failure matrix가 그 경계를 나눠 검증한다.
}

#[test]
fn windows_conpty_repeated_enter_bytes_do_not_cross_two_state_transitions() {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_aihack"));
    command.args(["--seed", "42"]);
    command.cwd(env!("CARGO_MANIFEST_DIR"));
    command.env("TERM", "xterm-256color");
    let mut child = pair.slave.spawn_command(command).unwrap();
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let mut writer = pair.master.take_writer().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader_thread = thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if tx.send(buffer[..count].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut transcript = Vec::new();
    let cursor_query = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    transcript.extend_from_slice(&cursor_query);
    if transcript
        .windows(b"\x1b[6n".len())
        .any(|window| window == b"\x1b[6n")
    {
        writer.write_all(b"\x1b[1;1R").unwrap();
        writer.flush().unwrap();
    }
    wait_for(&rx, &mut transcript, b"Press Enter to Start");

    transcript.clear();
    writer.write_all(b"\r\r").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"Character Creation");
    let settle_deadline = Instant::now() + Duration::from_millis(250);
    while Instant::now() < settle_deadline {
        if let Ok(chunk) = rx.recv_timeout(Duration::from_millis(20)) {
            transcript.extend_from_slice(&chunk);
        }
    }
    assert!(
        !transcript
            .windows(b"COMMANDS".len())
            .any(|window| window == b"COMMANDS"),
        "adjacent Enter bytes crossed CharacterCreation into Playing"
    );

    thread::sleep(Duration::from_millis(600));
    writer.write_all(b"\r").unwrap();
    writer.flush().unwrap();
    wait_for(&rx, &mut transcript, b"COMMANDS");
    thread::sleep(Duration::from_millis(600));
    writer.write_all(b"Q").unwrap();
    writer.flush().unwrap();
    thread::sleep(Duration::from_millis(650));
    writer.write_all(b"y").unwrap();
    writer.flush().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "ConPTY child did not exit");
        thread::yield_now();
    };
    drop(writer);
    drop(pair.master);
    while rx.recv_timeout(Duration::from_millis(50)).is_ok() {}
    reader_thread.join().unwrap();
    assert!(status.success(), "ConPTY child exit={}", status.exit_code());
}
