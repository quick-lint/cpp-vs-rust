use crate::c_api::web_demo_location::*;
use crate::fe::document::*;

// TODO(port-later): Test on LSPLocator in addition to WebDemoLocator.

#[test]
fn set_text() {
    let mut doc: Document<WebDemoLocator> = Document::new();
    doc.set_text(b"content goes here");
    assert_eq!(doc.string().slice(), b"content goes here");
}

#[test]
fn set_text_multiple_times() {
    let mut doc: Document<WebDemoLocator> = Document::new();
    doc.set_text(b"content goes here");
    doc.set_text(b"newer content goes here");
    assert_eq!(doc.string().slice(), b"newer content goes here");
    doc.set_text(b"finally");
    assert_eq!(doc.string().slice(), b"finally");
}
