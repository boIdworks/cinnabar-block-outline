//! Wire format and recorder tests.

use super::*;

#[test]
fn var_u32_round_trips() {
    for value in [0, 1, 127, 128, 300, 16_383, 16_384, u32::MAX] {
        let mut out = Vec::new();
        write_var_u32(&mut out, value);
        assert_eq!(Reader::new(&out).var_u32(), Some(value));
    }
}

#[test]
fn system_message_decodes_as_text() {
    let mut batch = Vec::new();
    packets::system_message(&mut batch, "hello");
    let packets = split_batch(&batch).unwrap();
    assert_eq!(packets[0].id, TEXT);
    let text = packets::read_text(&packets[0]).unwrap();
    assert_eq!((text.message.as_str(), text.author), ("hello", None));
}

#[test]
fn form_packets_decode() {
    let mut batch = Vec::new();
    packets::form_request(&mut batch, 7, r#"{"type":"form"}"#);
    packets::form_response(&mut batch, 7, "2\n");
    packets::form_cancel(&mut batch, 7);
    let packets = split_batch(&batch).unwrap();
    assert_eq!(
        packets::read_form_request(&packets[0]),
        Some((7, r#"{"type":"form"}"#.into()))
    );
    assert_eq!(
        packets::read_form_response(&packets[1]),
        Some((7, Some("2\n".into()), None))
    );
    // As the recording shows for a closed menu: no response, reason 0.
    assert_eq!(
        packets::read_form_response(&packets[2]),
        Some((7, None, Some(0)))
    );
}

#[test]
fn command_request_layout() {
    let batch = command("/cs");
    let packets = split_batch(&batch).unwrap();
    let mut body = packets[0].body();
    assert_eq!(body.string().as_deref(), Some("/cs"));
    assert_eq!(body.string().as_deref(), Some("player"));
    let uuid = body.take(16).unwrap();
    assert_eq!(uuid[6] >> 4, 4);
    assert_eq!(body.string().as_deref(), Some(""));
    assert_eq!(body.take(8), Some(&[0u8; 8][..]));
    assert_eq!(body.bool(), Some(false));
    assert_eq!(body.string().as_deref(), Some("latest"));
    assert!(body.rest().is_empty());
}

#[test]
fn header_sub_client_bits_are_ignored() {
    let mut batch = Vec::new();
    push_packet(&mut batch, 100 | (1 << 10) | (2 << 12), &[]);
    assert_eq!(split_batch(&batch).unwrap()[0].id, 100);
}

#[test]
fn truncated_batch_is_unreadable() {
    let mut batch = chat("hi");
    batch.pop();
    assert!(split_batch(&batch).is_none());
}

// ---- recorder -----------------------------------------------------------------------------------

#[test]
fn recorder_logs_forms_and_chat() {
    let mut recorder = Recorder::default();
    let mut batch = chat("gg");
    batch.extend(manage(5));
    recorder.batch(Way::FromServer, &split_batch(&batch).unwrap());
    let lines = recorder.drain();
    assert!(lines.iter().any(|line| line.contains("msg=\"gg\"")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("form=5") && line.contains("Manage Your Server"))
    );
}

#[test]
fn long_lines_split_on_char_boundaries() {
    let mut recorder = Recorder::default();
    recorder.note("é".repeat(1000));
    let lines = recorder.drain();
    assert!(lines.len() > 1);
    assert!(lines[0].starts_with(TAG) && lines[1].starts_with(MORE));
    assert!(lines.iter().all(|line| line.len() < 1024));
    let text: String = lines
        .iter()
        .map(|line| line.split_once(' ').unwrap().1)
        .collect();
    assert_eq!(text, "é".repeat(1000));
}

#[test]
fn drain_respects_the_log_allowance() {
    let mut recorder = Recorder::default();
    for _ in 0..100 {
        recorder.note("x".repeat(800));
    }
    let first = recorder.drain();
    assert!(first.iter().map(|line| line.len() + 1).sum::<usize>() <= 12 * 1024);
    assert!(!recorder.drain().is_empty());
}

#[test]
fn strings_finds_length_prefixed_text() {
    let mut bytes = vec![0xff, 0x02];
    write_string(&mut bytes, "hive:custom_server");
    bytes.push(0);
    assert_eq!(
        packets::strings(&bytes),
        vec!["hive:custom_server".to_string()]
    );
}
