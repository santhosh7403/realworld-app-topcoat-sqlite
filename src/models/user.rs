use ref_cast::RefCast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct User {
    #[serde(default)]
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    // #[serde(default = "default_per_page")]
    pub per_page_amount: i64,
    // #[serde(default = "default_theme")]
    pub theme_mode: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct UserPreview {
    pub id: i64,
    pub username: String,
}

// ---------------------------------------------------------------------------
// Surrogated implementation for UserPreview
// ---------------------------------------------------------------------------
//
// `signal()` requires `T: SignalValue`, which is auto-implemented when
// `T: Surrogated` (owned), `&T: Surrogated` (serialisation), and
// `&mut T: Surrogated` (mutation). The macros from topcoat-runtime handle
// all three; they expect a `#[repr(transparent)]` newtype that implements
// `ref_cast::RefCast` for the borrow variants.

/// Transparent surrogate that carries a [`UserPreview`] across the wire.
#[derive(Debug, Clone, RefCast, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct UserPreviewSurrogate(UserPreview);

impl UserPreviewSurrogate {
    #[inline]
    pub(crate) fn new(v: UserPreview) -> Self {
        Self(v)
    }
}

topcoat::runtime::impl_surrogate!(UserPreview, UserPreviewSurrogate);
topcoat::runtime::impl_surrogate_ref!(UserPreview, UserPreviewSurrogate);
topcoat::runtime::impl_surrogate_mut!(UserPreview, UserPreviewSurrogate);
