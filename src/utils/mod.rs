pub(crate) mod filters;
pub(crate) mod markdown;

#[must_use]
pub(crate) fn blob_mime(blob: &git2::Blob<'_>, extension: &str) -> mime::Mime {
    mime_guess::MimeGuess::from_ext(extension).first_or_else(|| {
        if blob.is_binary() {
            mime::APPLICATION_OCTET_STREAM
        } else {
            mime::TEXT_PLAIN_UTF_8
        }
    })
}
