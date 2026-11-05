//! Native OS confirm / PIN prompts — UI layer, not part of the framework crate.

use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use signbridge::error::{Error, Result};
use signbridge::ConfirmPrompt;

/// macOS / Windows / Linux message-box confirmations via `rfd`.
pub struct NativeConfirm;

impl ConfirmPrompt for NativeConfirm {
    fn confirm_sign(
        &self,
        origin: &str,
        document_title: Option<&str>,
        certificate_subject: &str,
    ) -> Result<bool> {
        let title = document_title.unwrap_or("(untitled document)");
        let body = format!(
            "A website wants you to sign a document fingerprint with your certificate.\n\n\
             Site: {origin}\n\
             Document: {title}\n\
             Certificate: {certificate_subject}\n\n\
             SignBridge will only sign a hash — never the full file.\n\
             Continue?"
        );

        let result = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("SignBridge — confirm signature")
            .set_description(&body)
            .set_buttons(MessageButtons::OkCancel)
            .show();

        Ok(matches!(result, MessageDialogResult::Ok))
    }

    fn ask_pin(&self, token_label: Option<&str>) -> Result<Option<String>> {
        if let Ok(pin) = std::env::var("SIGNBRIDGE_PIN") {
            return Ok(Some(pin));
        }

        let label = token_label.unwrap_or("USB token");
        let result = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("SignBridge — token PIN")
            .set_description(&format!(
                "Insert your token ({label}) if needed.\n\n\
                 When you click OK, your token driver may ask for the PIN.\n\
                 For automated tests set SIGNBRIDGE_PIN (never in production)."
            ))
            .set_buttons(MessageButtons::OkCancel)
            .show();

        if !matches!(result, MessageDialogResult::Ok) {
            return Err(Error::Cancelled);
        }
        Ok(None)
    }
}
